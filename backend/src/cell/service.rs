use crate::cell::constants::CellDisplayOrderMovement;
use crate::cell::constants::CellStatus;
use crate::cell::constants::CellType;
use crate::cell::dto;
use crate::cell::dto::CellContent;
use crate::cell::repository;
use crate::common::database_utils;
use crate::common::errors::SophoError;
use crate::common::time_utils;
use crate::common::AppState;
use crate::connection::service as connection_service;
use crate::dashboard::service as dashboard_service;
use crate::entity;
use crate::notebook::does_notebook_exist;
use crate::notebook::service as notebook_service;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use sea_orm::TransactionTrait;
use sqlx::postgres::PgConnection;
use sqlx::types::JsonValue;
use sqlx::Column;
use sqlx::Connection;
use sqlx::Row;
use uuid::Uuid;

pub async fn does_cell_exist(app_state: &AppState, id: Uuid) -> bool {
    repository::get_cell(&app_state.database_connection, id)
        .await
        .is_ok()
}

pub async fn get_cell(app_state: AppState, id: Uuid) -> impl IntoResponse {
    let cell = execute_get_cell(&app_state, id).await;
    match cell {
        Ok(cell) => {
            let response_dto = dto::CellDto::from(cell);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(e) => match e {
            sea_orm::DbErr::RecordNotFound(_) => (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Cell not found" })),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ),
        },
    }
}

async fn execute_get_cell(
    app_state: &AppState,
    id: Uuid,
) -> Result<entity::cell::Model, sea_orm::DbErr> {
    return repository::get_cell(&app_state.database_connection, id).await;
}

async fn generate_cell_name(
    app_state: &AppState,
    notebook_id: Uuid,
    cell_type: CellType,
) -> Result<String, SophoError> {
    let total_cells =
        get_number_of_cells_in_notebook_by_type(app_state, notebook_id, cell_type.clone()).await;
    match total_cells {
        Ok(total_cells) => {
            let cell_number = total_cells + 1;
            return Ok(format!("{}_{}", cell_type, cell_number));
        }
        Err(e) => return Err(e),
    }
}

pub async fn get_cells_by_notebook_id_and_cell_type(
    app_state: &AppState,
    notebook_id: Uuid,
    cell_type: Option<CellType>,
) -> Result<Vec<dto::CellDto>, SophoError> {
    let cells = match cell_type {
        Some(cell_type) => {
            repository::get_cells_by_notebook_id_and_cell_type(
                &app_state.database_connection,
                notebook_id,
                cell_type,
            )
            .await
        }
        None => {
            repository::get_cells_by_notebook_id(&app_state.database_connection, notebook_id).await
        }
    };
    match cells {
        Ok(cells) => {
            let response_dtos: Vec<dto::CellDto> = cells
                .into_iter()
                .map(|cell| dto::CellDto::from(cell))
                .collect();
            Ok(response_dtos)
        }
        Err(e) => Err(SophoError::DatabaseError(e)),
    }
}

async fn get_number_of_cells_in_notebook(
    app_state: &AppState,
    notebook_id: Uuid,
) -> Result<i32, SophoError> {
    let cells = get_cells_by_notebook_id_and_cell_type(&app_state, notebook_id, None).await;
    match cells {
        Ok(cells) => Ok(cells.len() as i32),
        Err(e) => Err(e),
    }
}

async fn get_number_of_cells_in_notebook_by_type(
    app_state: &AppState,
    id: Uuid,
    cell_type: CellType,
) -> Result<i32, SophoError> {
    let cells = get_cells_by_notebook_id_and_cell_type(&app_state, id, None).await;
    match cells {
        Ok(cells) => {
            let count = cells
                .into_iter()
                .filter(|cell| cell.cell_type == cell_type)
                .count();
            Ok(count as i32)
        }
        Err(e) => Err(e),
    }
}

pub async fn create_cell(app_state: AppState, payload: dto::CreateCellDto) -> impl IntoResponse {
    let display_order;
    let number_of_cells = get_number_of_cells_in_notebook(&app_state, payload.notebook_id).await;
    match number_of_cells {
        Ok(number_of_cells) => {
            display_order = number_of_cells + 1;
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    }

    if !does_notebook_exist(&app_state, payload.notebook_id).await {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Notebook not found" })),
        );
    }

    let cell_name =
        generate_cell_name(&app_state, payload.notebook_id, payload.cell_type.clone()).await;
    let cell_name = match cell_name {
        Ok(cell_name) => cell_name,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };

    let cell_entity = entity::cell::Model {
        id: Uuid::new_v4(),
        name: Some(cell_name),
        content: payload.content,
        cell_type: payload.cell_type.to_string(),
        status: CellStatus::Active.to_string(),
        notebook_id: payload.notebook_id,
        connection_id: payload.connection_id,
        display_order: display_order,
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };

    match repository::save_cell(&app_state.database_connection, cell_entity).await {
        Ok(cell) => {
            let response_dto = dto::CellDto::from(cell);
            (
                StatusCode::CREATED,
                axum::Json(serde_json::json!(response_dto)),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn delete_cell(app_state: AppState, id: Uuid) -> impl IntoResponse {
    let txn = match app_state.database_connection.begin().await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };
    let cell = match repository::get_cell_transaction(&txn, id).await {
        Ok(c) => c,
        Err(sea_orm::DbErr::RecordNotFound(_)) => {
            let _ = txn.rollback().await;
            return (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Cell not found" })),
            );
        }
        Err(e) => {
            let _ = txn.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };
    let notebook = match notebook_service::get_notebook_transaction(&txn, cell.notebook_id).await {
        Ok(n) => n,
        Err(e) => {
            let _ = txn.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };
    if let Err(e) =
        dashboard_service::remove_cell_from_layout_transaction(&txn, notebook.canvas_id, id).await
    {
        let _ = txn.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        );
    }
    if let Err(e) = repository::delete_cell_transaction(&txn, id).await {
        let _ = txn.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        );
    }
    match txn.commit().await {
        Ok(_) => (StatusCode::NO_CONTENT, axum::Json(serde_json::Value::Null)),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn reorder_cell(
    app_state: AppState,
    id: Uuid,
    payload: dto::ReorderCellDto,
) -> impl IntoResponse {
    let cell = match execute_get_cell(&app_state, id).await {
        Ok(c) => c,
        Err(sea_orm::DbErr::RecordNotFound(_)) => {
            return (
                StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Cell not found" })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };
    let cells = match repository::get_cells_by_notebook_id(
        &app_state.database_connection,
        cell.notebook_id,
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };
    let mut ordered_cells: Vec<entity::cell::Model> = cells;
    let len = ordered_cells.len() as i32;
    let current_index = cell.display_order;
    if current_index < 1 || current_index > len {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": "Cell display_order out of bounds" })),
        );
    }
    let new_index = match payload.movement_type {
        CellDisplayOrderMovement::Up => {
            if current_index > 1 {
                current_index - 1
            } else {
                current_index
            }
        }
        CellDisplayOrderMovement::Down => {
            if current_index < len {
                current_index + 1
            } else {
                current_index
            }
        }
        CellDisplayOrderMovement::Top => 1,
        CellDisplayOrderMovement::Bottom => len,
    };
    if new_index != current_index {
        let from_pos = (current_index - 1) as usize;
        let to_pos = (new_index - 1) as usize;
        let cell_model = ordered_cells.remove(from_pos);
        ordered_cells.insert(to_pos, cell_model);
    }
    let txn = match app_state.database_connection.begin().await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };
    let mut response_cell = cell.clone();
    for (i, c) in ordered_cells.iter().enumerate() {
        let new_order = (i + 1) as i32;
        if c.display_order != new_order {
            match repository::update_cell_display_order_transaction(&txn, c.id, new_order).await {
                Ok(updated) => {
                    if c.id == id {
                        response_cell = updated;
                    }
                }
                Err(e) => {
                    let _ = txn.rollback().await;
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(serde_json::json!({ "error": e.to_string() })),
                    );
                }
            }
        }
    }
    match txn.commit().await {
        Ok(_) => {
            let response_dto = dto::CellDto::from(response_cell);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn update_cell(
    app_state: AppState,
    id: Uuid,
    payload: dto::CellDto,
) -> impl IntoResponse {
    let cell = repository::update_cell(&app_state.database_connection, id, payload).await;
    match cell {
        Ok(cell) => {
            let response_dto = dto::CellDto::from(cell);
            (StatusCode::OK, axum::Json(serde_json::json!(response_dto)))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

pub async fn execute_cell(app_state: AppState, id: Uuid) -> impl IntoResponse {
    let cell = execute_get_cell(&app_state, id).await;
    let cell = match cell {
        Ok(cell) => cell,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };
    let cell_type = match CellType::from_str(&cell.cell_type) {
        Ok(cell_type) => cell_type,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
        }
    };
    match cell_type {
        CellType::Sql => return execute_sql_cell(app_state, cell).await,
        CellType::Chart => return execute_chart_cell(app_state, cell).await,
        _ => {
            return (
                StatusCode::EXPECTATION_FAILED,
                axum::Json(serde_json::json!({ "error": "cell type not supported" })),
            );
        }
    }
}

async fn get_database_connection(
    app_state: AppState,
    connection_id: Uuid,
) -> Result<PgConnection, (http::StatusCode, axum::Json<JsonValue>)> {
    let connection = connection_service::execute_get_connection(app_state, connection_id).await;
    let connection = match connection {
        Ok(connection) => connection,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };

    let database_url = database_utils::get_database_url(&connection);
    let database_connection = PgConnection::connect(&database_url).await;
    match database_connection {
        Ok(database_connection) => Ok(database_connection),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

async fn execute_query_and_format_results(
    mut database_connection: PgConnection,
    query: &str,
) -> (http::StatusCode, axum::Json<JsonValue>) {
    let result = sqlx::query(query).fetch_all(&mut database_connection).await;
    let rows = match result {
        Ok(rows) => rows,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            );
        }
    };

    let mut columns: Vec<serde_json::Value> = Vec::new();
    if let Some(first_row) = rows.get(0) {
        columns = first_row
            .columns()
            .iter()
            .map(|col| {
                serde_json::json!({
                    "column_name": col.name().to_string(),
                    "data_type": col.type_info().to_string()
                })
            })
            .collect();
    }

    let mut json_rows: Vec<serde_json::Value> = Vec::new();
    for row in rows {
        let mut map = serde_json::Map::new();
        for (i, col) in row.columns().iter().enumerate() {
            let value: Result<serde_json::Value, sqlx::Error> = match col
                .type_info()
                .to_string()
                .as_str()
            {
                "BOOL" => {
                    let value = row.try_get::<bool, _>(i);
                    value.map(serde_json::Value::Bool)
                }
                "UUID" => row.try_get::<Uuid, _>(i).map(|v| serde_json::json!(v)),
                "TEXT" => {
                    let value = row.try_get::<String, _>(i);
                    value.map(serde_json::Value::String)
                }
                "JSONB" => row
                    .try_get::<sqlx::types::Json<serde_json::Value>, _>(i)
                    .map(|j| {
                        serde_json::Value::String(serde_json::to_string(&j.0).unwrap_or_default())
                    }),
                "VARCHAR" => {
                    let value = row.try_get::<String, _>(i);
                    value.map(serde_json::Value::String)
                }
                "DATE" => row.try_get::<NaiveDate, _>(i).map(|v| serde_json::json!(v)),
                "TIMESTAMP" => row
                    .try_get::<NaiveDateTime, _>(i)
                    .map(|v| serde_json::json!(v)),
                "TIMESTAMPTZ" => row
                    .try_get::<DateTime<FixedOffset>, _>(i)
                    .map(|v| serde_json::json!(v)),
                "INT4" => row.try_get::<i32, _>(i).map(|v| serde_json::json!(v)),
                "INT8" => row.try_get::<i64, _>(i).map(|v| serde_json::json!(v)),
                "NUMERIC" => row.try_get::<Decimal, _>(i).map(|v| {
                    let f: f64 = v.try_into().unwrap_or(0.0);
                    serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null)
                }),
                _ => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        axum::Json(serde_json::json!({
                            "error": format!("data type '{}' not handled", col.type_info().to_string())
                        })),
                    );
                }
            };
            match value {
                Ok(value) => {
                    map.insert(col.name().to_string(), value);
                }
                Err(err) => {
                    if let sqlx::Error::ColumnDecode {
                        index: _index,
                        source: _source,
                    } = &err
                    {
                        map.insert(col.name().to_string(), serde_json::Value::Null);
                    } else {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            axum::Json(serde_json::json!({ "error": err.to_string() })),
                        );
                    }
                }
            }
        }
        json_rows.push(serde_json::Value::Object(map));
    }
    (
        StatusCode::OK,
        axum::Json(serde_json::json!(
            {
                "columns": columns,
                "data": json_rows,
            }
        )),
    )
}

fn build_aggregated_query(
    source_query: &str,
    x_axis: &str,
    y_axis: &str,
    aggregate_fn: &str,
) -> String {
    format!(
        "SELECT {}, {}({}) AS {} FROM ({}) AS subquery GROUP BY {}",
        x_axis, aggregate_fn, y_axis, y_axis, source_query, x_axis
    )
}

fn build_pie_chart_aggregated_query(
    source_query: &str,
    category: &str,
    value: &str,
    aggregate_fn: &str,
) -> String {
    format!(
        "SELECT {}, {}({}) AS {} FROM ({}) AS subquery GROUP BY {}",
        category, aggregate_fn, value, value, source_query, category
    )
}

fn get_sql_query_from_cell(
    cell: &entity::cell::Model,
) -> Result<String, (http::StatusCode, axum::Json<JsonValue>)> {
    let cell_content = match &cell.content {
        Some(cell_content) => cell_content,
        None => {
            return Err((
                StatusCode::PRECONDITION_FAILED,
                axum::Json(serde_json::json!({ "error": "Cell has no content" })),
            ));
        }
    };

    match CellContent::parse(cell_content, &CellType::Sql) {
        Ok(CellContent::Sql(sql_content)) => Ok(sql_content.query),
        Ok(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": "Cell content is not SQL" })),
        )),
        Err(_) => Ok(cell_content.clone()),
    }
}

async fn execute_sql_cell(
    app_state: AppState,
    cell: entity::cell::Model,
) -> (http::StatusCode, axum::Json<JsonValue>) {
    let connection_id = match cell.connection_id {
        Some(connection_id) => connection_id,
        None => {
            return (
                StatusCode::PRECONDITION_FAILED,
                axum::Json(serde_json::json!({ "error": "Cell has no connection assigned" })),
            );
        }
    };

    let query = match get_sql_query_from_cell(&cell) {
        Ok(query) => query,
        Err(err) => return err,
    };

    let database_connection = match get_database_connection(app_state, connection_id).await {
        Ok(conn) => conn,
        Err(err) => return err,
    };

    execute_query_and_format_results(database_connection, &query).await
}

fn parse_chart_content_from_cell(
    cell: &entity::cell::Model,
) -> Result<dto::ChartContent, (http::StatusCode, axum::Json<JsonValue>)> {
    let content = match &cell.content {
        Some(content) => content,
        None => {
            return Err((
                StatusCode::PRECONDITION_FAILED,
                axum::Json(serde_json::json!({ "error": "Cell has no content" })),
            ));
        }
    };

    let cell_content = match CellContent::parse(content, &CellType::Chart) {
        Ok(cell_content) => cell_content,
        Err(e) => {
            tracing::error!("Error when parsing cell content: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": "Cell has non serializable content" })),
            ));
        }
    };

    match cell_content {
        CellContent::Chart(chart_content) => Ok(chart_content),
        _ => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": "Cell has non serializable content" })),
        )),
    }
}

fn get_source_cell_id(chart_content: &dto::ChartContent) -> Uuid {
    match chart_content {
        dto::ChartContent::Bar(axis_content) | dto::ChartContent::Line(axis_content) => {
            axis_content.cell_id
        }
        dto::ChartContent::Pie(pie_content) => pie_content.cell_id,
    }
}

async fn get_source_cell_for_chart(
    app_state: &AppState,
    source_cell_id: Uuid,
) -> Result<(entity::cell::Model, Uuid, String), (http::StatusCode, axum::Json<JsonValue>)> {
    let source_cell = execute_get_cell(app_state, source_cell_id).await;
    let source_cell = match source_cell {
        Ok(source_cell) => source_cell,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };

    let connection_id = match source_cell.connection_id {
        Some(connection_id) => connection_id,
        None => {
            return Err((
                StatusCode::PRECONDITION_FAILED,
                axum::Json(
                    serde_json::json!({ "error": "Source cell has no connection assigned" }),
                ),
            ));
        }
    };

    let source_query = match get_sql_query_from_cell(&source_cell) {
        Ok(query) => query,
        Err(err) => return Err(err),
    };

    Ok((source_cell, connection_id, source_query))
}

fn build_chart_aggregated_query(
    chart_content: dto::ChartContent,
    source_query: &str,
) -> Result<String, (http::StatusCode, axum::Json<JsonValue>)> {
    match chart_content {
        dto::ChartContent::Bar(axis_content) | dto::ChartContent::Line(axis_content) => {
            let aggregate_fn = match &axis_content.y_axis_aggregate_function {
                Some(fn_name) => fn_name.as_str(),
                None => {
                    return Err((
                        StatusCode::PRECONDITION_FAILED,
                        axum::Json(serde_json::json!({
                            "error": "ChartCell has no aggregate function specified"
                        })),
                    ));
                }
            };

            Ok(build_aggregated_query(
                source_query,
                &axis_content.x_axis,
                &axis_content.y_axis,
                aggregate_fn,
            ))
        }
        dto::ChartContent::Pie(pie_content) => {
            let aggregate_fn = match &pie_content.aggregate_function {
                Some(fn_name) => fn_name.as_str(),
                None => {
                    return Err((
                        StatusCode::PRECONDITION_FAILED,
                        axum::Json(serde_json::json!({
                            "error": "ChartCell has no aggregate function specified"
                        })),
                    ));
                }
            };

            Ok(build_pie_chart_aggregated_query(
                source_query,
                &pie_content.category,
                &pie_content.value,
                aggregate_fn,
            ))
        }
    }
}

async fn execute_chart_cell(
    app_state: AppState,
    cell: entity::cell::Model,
) -> (http::StatusCode, axum::Json<JsonValue>) {
    let chart_content = match parse_chart_content_from_cell(&cell) {
        Ok(content) => content,
        Err(err) => return err,
    };

    let source_cell_id = get_source_cell_id(&chart_content);
    let (_, connection_id, source_query) =
        match get_source_cell_for_chart(&app_state, source_cell_id).await {
            Ok(result) => result,
            Err(err) => return err,
        };

    let aggregated_query = match build_chart_aggregated_query(chart_content, &source_query) {
        Ok(query) => query,
        Err(err) => return err,
    };

    let database_connection = match get_database_connection(app_state, connection_id).await {
        Ok(conn) => conn,
        Err(err) => return err,
    };

    execute_query_and_format_results(database_connection, &aggregated_query).await
}

pub async fn get_last_modified_cells_by_type(
    app_state: &AppState,
    cell_type: CellType,
    limit: u64,
) -> Result<Vec<entity::cell::Model>, sea_orm::DbErr> {
    repository::get_latest_cells_by_type(&app_state.database_connection, cell_type, limit).await
}

pub async fn search_cells_by_name_and_type(
    app_state: &AppState,
    search_query: &str,
    cell_type: CellType,
    limit: u64,
) -> Result<Vec<entity::cell::Model>, sea_orm::DbErr> {
    repository::search_cells_by_name_and_type(
        &app_state.database_connection,
        search_query,
        cell_type,
        limit,
    )
    .await
}

pub async fn delete_cells_by_notebook_id_transaction(
    txn: &sea_orm::DatabaseTransaction,
    notebook_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    repository::delete_cells_by_notebook_id_transaction(txn, notebook_id).await
}

pub async fn get_cell_counts_by_canvas_id(
    app_state: &AppState,
    canvas_id: Uuid,
) -> Result<(i32, i32), SophoError> {
    let notebook = notebook_service::get_notebook_by_canvas_id(app_state, canvas_id)
        .await
        .map_err(|e| SophoError::DatabaseError(e))?;

    let sql_count = repository::count_cells_by_notebook_id_and_cell_type(
        &app_state.database_connection,
        notebook.id,
        CellType::Sql,
    )
    .await
    .map_err(|e| SophoError::DatabaseError(e))?;

    let chart_count = repository::count_cells_by_notebook_id_and_cell_type(
        &app_state.database_connection,
        notebook.id,
        CellType::Chart,
    )
    .await
    .map_err(|e| SophoError::DatabaseError(e))?;

    Ok((sql_count as i32, chart_count as i32))
}
