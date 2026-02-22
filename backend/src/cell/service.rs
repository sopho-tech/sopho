use crate::cell::constants::CellDisplayOrderMovement;
use crate::cell::constants::CellStatus;
use crate::cell::constants::CellType;
use crate::cell::constants::SortOrder;
use crate::cell::dto;
use crate::cell::dto::CellContent;
use crate::cell::repository;
use crate::common::error_codes::codes;
use crate::common::errors::CreateCellError;
use crate::common::errors::ExecuteQueryError;
use crate::common::errors::SophoError;
use crate::common::time_utils;
use crate::common::AppState;
use crate::connection::constants::SourceType;
use crate::connection::service as connection_service;
use crate::dashboard::service as dashboard_service;
use crate::database::constants::DatabaseConnection;
use crate::database::{postgres, sqlite};
use crate::entity;
use crate::notebook::does_notebook_exist;
use crate::notebook::service as notebook_service;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use sea_orm::TransactionTrait;
use sqlx::types::JsonValue;
use tracing::info;
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

pub async fn execute_create_cell(
    app_state: &AppState,
    payload: dto::CreateCellDto,
) -> Result<entity::cell::Model, CreateCellError> {
    if !does_notebook_exist(app_state, payload.notebook_id).await {
        return Err(CreateCellError::NotebookNotFound);
    }

    let display_order = match payload.display_order {
        Some(order) => order,
        None => {
            let number_of_cells = get_number_of_cells_in_notebook(app_state, payload.notebook_id)
                .await
                .map_err(CreateCellError::Sopho)?;
            number_of_cells + 1
        }
    };

    let cell_name = match payload.name {
        Some(name) => name,
        None => generate_cell_name(app_state, payload.notebook_id, payload.cell_type.clone())
            .await
            .map_err(CreateCellError::Sopho)?,
    };

    let cell_entity = entity::cell::Model {
        id: Uuid::new_v4(),
        name: Some(cell_name),
        content: payload.content,
        cell_type: payload.cell_type.to_string(),
        status: CellStatus::Active.to_string(),
        notebook_id: payload.notebook_id,
        connection_id: payload.connection_id,
        display_order,
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };

    let cell = repository::save_cell(&app_state.database_connection, cell_entity)
        .await
        .map_err(CreateCellError::Repository)?;

    Ok(cell)
}

pub async fn create_cell(app_state: AppState, payload: dto::CreateCellDto) -> impl IntoResponse {
    match execute_create_cell(&app_state, payload).await {
        Ok(cell) => {
            let response_dto = dto::CellDto::from(cell);
            (
                StatusCode::CREATED,
                axum::Json(serde_json::json!(response_dto)),
            )
        }
        Err(CreateCellError::NotebookNotFound) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Notebook not found" })),
        ),
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
) -> Result<DatabaseConnection, (http::StatusCode, axum::Json<JsonValue>)> {
    let connection = connection_service::execute_get_connection(&app_state, connection_id).await;
    let connection = match connection {
        Ok(connection) => connection,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            ));
        }
    };

    let database_connection = match SourceType::from_str(&connection.source_type).unwrap() {
        SourceType::Postgresql | SourceType::Supabase => postgres::get_database_connection(&connection).await,
        SourceType::Sqlite => sqlite::get_database_connection(&connection).await,
        _ => panic!("Not implemented"),
    };

    match database_connection {
        Ok(database_connection) => Ok(database_connection),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )),
    }
}

async fn execute_query_and_format_results(
    mut database_connection: DatabaseConnection,
    query: &str,
) -> (http::StatusCode, axum::Json<JsonValue>) {
    let result = match &mut database_connection {
        DatabaseConnection::Postgres(conn) => postgres::execute_query(conn, query).await,
        DatabaseConnection::Sqlite(conn) => sqlite::execute_query(conn, query).await,
    };
    match result {
        Ok(result) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "columns": result.columns,
                "data": result.data,
            })),
        ),
        Err(ExecuteQueryError::Database(sqlx::Error::Database(e))) => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "status": StatusCode::BAD_REQUEST.as_u16(),
                "code": codes::SYNTAX_ERROR.as_str(),
                "message": e.to_string()
            })),
        ),
        Err(ExecuteQueryError::Database(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        ),
        Err(ExecuteQueryError::UnhandledDataType(type_name)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "error": format!("data type '{}' not handled", type_name)
            })),
        ),
    }
}

fn build_aggregated_query(
    source_query: &str,
    x_axis: &str,
    y_axis: &str,
    aggregate_fn: &str,
    y_axis_sort_order: &str,
) -> String {
    if y_axis_sort_order != SortOrder::None.as_str() {
        return format!(
            "SELECT \"{}\", {}(\"{}\") AS \"{}\" FROM ({}) AS subquery GROUP BY \"{}\" ORDER BY \"{}\" {}",
            x_axis, aggregate_fn, y_axis, y_axis, source_query, x_axis, y_axis, y_axis_sort_order
        );
    }
    format!(
        "SELECT \"{}\", {}(\"{}\") AS \"{}\" FROM ({}) AS subquery GROUP BY \"{}\"",
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

fn get_sql_query_from_content_inner(
    content: &str,
    invalid_format_status: StatusCode,
    invalid_format_msg: &str,
) -> Result<String, (http::StatusCode, axum::Json<JsonValue>)> {
    match CellContent::parse(content, &CellType::Sql) {
        Ok(CellContent::Sql(sql_content)) => Ok(sql_content.query),
        Ok(_) => Err((
            invalid_format_status,
            axum::Json(serde_json::json!({ "error": invalid_format_msg })),
        )),
        Err(_) => Ok(content.to_string()),
    }
}

fn get_sql_query_from_cell(
    cell: &entity::cell::Model,
) -> Result<String, (http::StatusCode, axum::Json<JsonValue>)> {
    let content = match &cell.content {
        Some(c) => c,
        None => {
            return Err((
                StatusCode::PRECONDITION_FAILED,
                axum::Json(serde_json::json!({ "error": "Cell has no content" })),
            ));
        }
    };
    get_sql_query_from_content_inner(
        content,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Cell content is not SQL",
    )
}

fn get_sql_query_from_content(
    content: &str,
) -> Result<String, (http::StatusCode, axum::Json<JsonValue>)> {
    get_sql_query_from_content_inner(
        content,
        StatusCode::BAD_REQUEST,
        "Content is not valid SQL format",
    )
}

fn parse_chart_content_inner(
    content: &str,
    parse_error_status: StatusCode,
    parse_error_msg: &str,
) -> Result<dto::ChartContent, (http::StatusCode, axum::Json<JsonValue>)> {
    let cell_content = match CellContent::parse(content, &CellType::Chart) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Error when parsing chart content: {}", e);
            return Err((
                parse_error_status,
                axum::Json(serde_json::json!({ "error": parse_error_msg })),
            ));
        }
    };
    match cell_content {
        CellContent::Chart(c) => Ok(c),
        _ => Err((
            parse_error_status,
            axum::Json(serde_json::json!({ "error": parse_error_msg })),
        )),
    }
}

fn parse_chart_content_from_string(
    content: &str,
) -> Result<dto::ChartContent, (http::StatusCode, axum::Json<JsonValue>)> {
    parse_chart_content_inner(
        content,
        StatusCode::BAD_REQUEST,
        "Invalid chart content format",
    )
}

pub async fn execute_cell_preview(
    app_state: AppState,
    id: Uuid,
    payload: dto::ExecuteCellPreviewDto,
) -> impl IntoResponse {
    let cell = execute_get_cell(&app_state, id).await;
    let cell = match cell {
        Ok(cell) => cell,
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

    match payload.cell_type {
        CellType::Sql => {
            let connection_id = match cell.connection_id {
                Some(id) => id,
                None => {
                    return (
                        StatusCode::PRECONDITION_FAILED,
                        axum::Json(
                            serde_json::json!({ "error": "Cell has no connection assigned" }),
                        ),
                    );
                }
            };
            let query = match get_sql_query_from_content(&payload.content) {
                Ok(q) => q,
                Err(err) => return err,
            };
            execute_sql_with_query(app_state, connection_id, &query).await
        }
        CellType::Chart => {
            let chart_content = match parse_chart_content_from_string(&payload.content) {
                Ok(c) => c,
                Err(err) => return err,
            };
            execute_chart_with_content(app_state, chart_content).await
        }
        _ => (
            StatusCode::BAD_REQUEST,
            axum::Json(
                serde_json::json!({ "error": "Only SQL and Chart cell types are supported for preview execution" }),
            ),
        ),
    }
}

async fn execute_sql_with_query(
    app_state: AppState,
    connection_id: Uuid,
    query: &str,
) -> (http::StatusCode, axum::Json<JsonValue>) {
    let database_connection = match get_database_connection(app_state, connection_id).await {
        Ok(conn) => conn,
        Err(err) => return err,
    };
    execute_query_and_format_results(database_connection, query).await
}

async fn execute_sql_cell(
    app_state: AppState,
    cell: entity::cell::Model,
) -> (http::StatusCode, axum::Json<JsonValue>) {
    let connection_id = match cell.connection_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::PRECONDITION_FAILED,
                axum::Json(serde_json::json!({
                    "status": StatusCode::PRECONDITION_FAILED.as_u16(),
                    "code": codes::MISSING_PREREQUISITES.as_str(),
                    "message": "Choose a connection"
                })),
            );
        }
    };
    let query = match get_sql_query_from_cell(&cell) {
        Ok(q) => q,
        Err(err) => return err,
    };
    execute_sql_with_query(app_state, connection_id, &query).await
}

fn parse_chart_content_from_cell(
    cell: &entity::cell::Model,
) -> Result<dto::ChartContent, (http::StatusCode, axum::Json<JsonValue>)> {
    let content = match &cell.content {
        Some(c) => c,
        None => {
            return Err((
                StatusCode::PRECONDITION_FAILED,
                axum::Json(serde_json::json!({ "error": "Cell has no content" })),
            ));
        }
    };
    parse_chart_content_inner(
        content,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Cell has non serializable content",
    )
}

fn get_source_cell_id(chart_content: &dto::ChartContent) -> Uuid {
    match chart_content {
        dto::ChartContent::Bar(axis_content) | dto::ChartContent::Line(axis_content) => {
            axis_content.cell_id
        }
        dto::ChartContent::Pie(pie_content) => pie_content.cell_id,
        dto::ChartContent::Metric(metric_content) => metric_content.cell_id,
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
                            "message": "ChartCell has no aggregate function specified"
                        })),
                    ));
                }
            };

            let y_axis_sort_order = match &axis_content.y_axis_sort_order {
                Some(y_axis_sort_order) => y_axis_sort_order.as_str(),
                None => {
                    return Err((
                        StatusCode::PRECONDITION_FAILED,
                        axum::Json(serde_json::json!({
                            "message": "ChartCell has no y-axis sort order specified"
                        })),
                    ));
                }
            };

            Ok(build_aggregated_query(
                source_query,
                &axis_content.x_axis,
                &axis_content.y_axis,
                aggregate_fn,
                y_axis_sort_order,
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
        dto::ChartContent::Metric(_) => Ok(source_query.to_string()),
    }
}

async fn execute_chart_with_content(
    app_state: AppState,
    chart_content: dto::ChartContent,
) -> (http::StatusCode, axum::Json<JsonValue>) {
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
    execute_sql_with_query(app_state, connection_id, &aggregated_query).await
}

async fn execute_chart_cell(
    app_state: AppState,
    cell: entity::cell::Model,
) -> (http::StatusCode, axum::Json<JsonValue>) {
    let chart_content = match parse_chart_content_from_cell(&cell) {
        Ok(c) => c,
        Err(err) => return err,
    };
    execute_chart_with_content(app_state, chart_content).await
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
