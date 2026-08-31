use crate::ai_summary::service as ai_summary_service;
use crate::authorization::constants::Action;
use crate::authorization::dto::{TenantContext, TenantId};
use crate::authorization::service as authorization_service;
use crate::canvas::service as canvas_service;
use crate::cell::constants::AggregateFunction;
use crate::cell::constants::CellDisplayOrderMovement;
use crate::cell::constants::CellStatus;
use crate::cell::constants::CellType;
use crate::cell::constants::SortOrder;
use crate::cell::constants::MAX_CELLS_PER_NOTEBOOK;
use crate::cell::constants::MAX_CHART_SERIES;
use crate::cell::constants::MAX_IDENTIFIER_LEN;
use crate::cell::constants::{CELL_DISPLAY_ORDER_OUT_OF_BOUNDS_MESSAGE, CELL_NOT_FOUND_MESSAGE};
use crate::cell::dto;
use crate::cell::dto::CellContent;
use crate::cell::error::CellError;
use crate::cell::repository;
use crate::common::errors::CreateCellError;
use crate::common::errors::ExecuteChartError;
use crate::common::errors::GetDatabaseConnectionError;
use crate::common::errors::SophoError;
use crate::common::time_utils;
use crate::common::AppState;
use crate::connection::service as connection_service;
use crate::dashboard::service as dashboard_service;
use crate::database::constants::QueryResult;
use crate::database::service as database_service;
use crate::entity;
use crate::notebook::does_notebook_exist;
use crate::notebook::service as notebook_service;
use futures_util::stream::{self, StreamExt};
use sea_orm::TransactionTrait;
use std::collections::HashMap;
use std::collections::HashSet;
use uuid::Uuid;

pub async fn does_cell_exist(app_state: &AppState, tenant_id: &TenantId, id: Uuid) -> bool {
    repository::get_cell(&app_state.database_connection, tenant_id, id)
        .await
        .is_ok()
}

pub async fn load_authorization_subject(
    app_state: &AppState,
    tenant_id: &TenantId,
    cell_id: Uuid,
) -> Result<crate::authorization::dto::AuthorizationSubject, sea_orm::DbErr> {
    let cell = repository::get_cell(&app_state.database_connection, tenant_id, cell_id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(CELL_NOT_FOUND_MESSAGE.into()))?;
    let notebook =
        notebook_service::get_notebook_entity(app_state, tenant_id, cell.notebook_id).await?;
    canvas_service::load_authorization_subject(app_state, tenant_id, notebook.canvas_id).await
}

async fn authorize_cell(
    app_state: &AppState,
    tenant_context: &TenantContext,
    cell_id: Uuid,
    action: Action,
) -> Result<entity::cell::Model, CellError> {
    let cell = repository::get_cell(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        cell_id,
    )
    .await
    .map_err(|_| CellError::NotFound)?
    .ok_or(CellError::NotFound)?;

    let subject = load_authorization_subject(app_state, tenant_context.tenant_id(), cell_id)
        .await
        .map_err(|_| CellError::NotFound)?;

    authorization_service::require_access(app_state, tenant_context, &subject, action).await?;

    Ok(cell)
}

pub async fn get_cell(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
) -> Result<dto::CellDto, CellError> {
    let cell = authorize_cell(&app_state, tenant_context, id, Action::Read).await?;
    Ok(dto::CellDto::from(cell))
}

async fn execute_get_cell(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<entity::cell::Model, sea_orm::DbErr> {
    repository::get_cell(&app_state.database_connection, tenant_id, id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(CELL_NOT_FOUND_MESSAGE.into()))
}

async fn generate_cell_name(
    app_state: &AppState,
    tenant_id: &TenantId,
    notebook_id: Uuid,
    cell_type: CellType,
) -> Result<String, SophoError> {
    let total_cells = get_number_of_cells_in_notebook_by_type(
        app_state,
        tenant_id,
        notebook_id,
        cell_type.clone(),
    )
    .await;
    match total_cells {
        Ok(total_cells) => {
            let cell_number = total_cells + 1;
            Ok(format!("{}_{}", cell_type, cell_number))
        }
        Err(e) => Err(e),
    }
}

pub async fn get_cells_by_notebook_id_and_cell_type(
    app_state: &AppState,
    tenant_id: &TenantId,
    notebook_id: Uuid,
    cell_type: Option<CellType>,
) -> Result<Vec<dto::CellDto>, SophoError> {
    let cells = match cell_type {
        Some(cell_type) => {
            repository::get_cells_by_notebook_id_and_cell_type(
                &app_state.database_connection,
                tenant_id,
                notebook_id,
                cell_type,
            )
            .await
        }
        None => {
            repository::get_cells_by_notebook_id(
                &app_state.database_connection,
                tenant_id,
                notebook_id,
            )
            .await
        }
    };
    match cells {
        Ok(cells) => {
            let response_dtos: Vec<dto::CellDto> =
                cells.into_iter().map(dto::CellDto::from).collect();
            Ok(response_dtos)
        }
        Err(e) => Err(SophoError::DatabaseError(e)),
    }
}

async fn get_number_of_cells_in_notebook(
    app_state: &AppState,
    tenant_id: &TenantId,
    notebook_id: Uuid,
) -> Result<i32, SophoError> {
    let cells =
        get_cells_by_notebook_id_and_cell_type(app_state, tenant_id, notebook_id, None).await;
    match cells {
        Ok(cells) => Ok(cells.len() as i32),
        Err(e) => Err(e),
    }
}

async fn get_number_of_cells_in_notebook_by_type(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
    cell_type: CellType,
) -> Result<i32, SophoError> {
    let cells = get_cells_by_notebook_id_and_cell_type(app_state, tenant_id, id, None).await;
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
    tenant_id: &TenantId,
    payload: dto::CreateCellDto,
) -> Result<entity::cell::Model, CreateCellError> {
    if !does_notebook_exist(app_state, tenant_id, payload.notebook_id).await {
        return Err(CreateCellError::NotebookNotFound);
    }

    let cell_count = repository::count_cells_by_notebook_id(
        &app_state.database_connection,
        tenant_id,
        payload.notebook_id,
    )
    .await
    .map_err(CreateCellError::Repository)?;
    if cell_count as usize >= MAX_CELLS_PER_NOTEBOOK {
        return Err(CreateCellError::NotebookFull(MAX_CELLS_PER_NOTEBOOK));
    }

    let display_order = match payload.display_order {
        Some(order) => order,
        None => {
            let number_of_cells =
                get_number_of_cells_in_notebook(app_state, tenant_id, payload.notebook_id)
                    .await
                    .map_err(CreateCellError::Sopho)?;
            number_of_cells + 1
        }
    };

    let cell_name = match payload.name {
        Some(name) => name,
        None => generate_cell_name(
            app_state,
            tenant_id,
            payload.notebook_id,
            payload.cell_type.clone(),
        )
        .await
        .map_err(CreateCellError::Sopho)?,
    };

    let cell_entity = entity::cell::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
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

    let cell = repository::save_cell(&app_state.database_connection, tenant_id, cell_entity)
        .await
        .map_err(CreateCellError::Repository)?;

    Ok(cell)
}

pub async fn execute_create_cell_transaction(
    txn: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    payload: dto::CreateCellDto,
    display_order: i32,
) -> Result<entity::cell::Model, sea_orm::DbErr> {
    let cell_count =
        repository::count_cells_by_notebook_id(txn, tenant_id, payload.notebook_id).await?;
    if cell_count as usize >= MAX_CELLS_PER_NOTEBOOK {
        return Err(sea_orm::DbErr::Custom(format!(
            "notebook already holds the maximum of {MAX_CELLS_PER_NOTEBOOK} cells"
        )));
    }

    let cell_entity = entity::cell::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
        name: payload.name,
        content: payload.content,
        cell_type: payload.cell_type.to_string(),
        status: CellStatus::Active.to_string(),
        notebook_id: payload.notebook_id,
        connection_id: payload.connection_id,
        display_order,
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };
    repository::save_cell_transaction(txn, tenant_id, cell_entity).await
}

pub async fn get_cells_by_notebook_id_entities(
    app_state: &AppState,
    tenant_id: &TenantId,
    notebook_id: Uuid,
) -> Result<Vec<entity::cell::Model>, sea_orm::DbErr> {
    repository::get_cells_by_notebook_id(&app_state.database_connection, tenant_id, notebook_id)
        .await
}

pub async fn get_cells_by_notebook_id_transaction(
    txn: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    notebook_id: Uuid,
) -> Result<Vec<entity::cell::Model>, sea_orm::DbErr> {
    repository::get_cells_by_notebook_id_transaction(txn, tenant_id, notebook_id).await
}

pub async fn update_cell_content_transaction(
    txn: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    cell_id: Uuid,
    name: Option<String>,
    content: Option<String>,
) -> Result<entity::cell::Model, sea_orm::DbErr> {
    repository::update_cell_content_transaction(txn, tenant_id, cell_id, name, content)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Cell not found".into()))
}

pub async fn delete_cell_transaction(
    txn: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    cell_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    repository::delete_cell_transaction(txn, tenant_id, cell_id).await?;
    Ok(())
}

pub async fn create_cell(
    app_state: AppState,
    tenant_context: &TenantContext,
    payload: dto::CreateCellDto,
) -> Result<dto::CellDto, CellError> {
    crate::authorization::service::require_capability(
        tenant_context,
        crate::authorization::constants::Permission::CanvasWrite,
    )?;

    let cell = execute_create_cell(&app_state, tenant_context.tenant_id(), payload).await?;
    Ok(dto::CellDto::from(cell))
}

pub async fn delete_cell(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
) -> Result<(), CellError> {
    authorize_cell(&app_state, tenant_context, id, Action::Delete).await?;

    let transaction = app_state.database_connection.begin().await?;
    match delete_cell_in_transaction(&transaction, tenant_context.tenant_id(), id).await {
        Ok(()) => {
            transaction.commit().await?;
            Ok(())
        }
        Err(error) => {
            let _ = transaction.rollback().await;
            Err(error)
        }
    }
}

async fn delete_cell_in_transaction(
    transaction: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<(), CellError> {
    let cell = repository::get_cell_transaction(transaction, tenant_id, id)
        .await?
        .ok_or(CellError::NotFound)?;

    let notebook =
        notebook_service::get_notebook_transaction(transaction, tenant_id, cell.notebook_id)
            .await?;

    dashboard_service::remove_cell_from_layout_transaction(
        transaction,
        tenant_id,
        notebook.canvas_id,
        id,
    )
    .await?;
    repository::delete_cell_transaction(transaction, tenant_id, id).await?;
    ai_summary_service::delete_chart_summary(transaction, tenant_id, id).await?;

    Ok(())
}

pub async fn reorder_cell(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    payload: dto::ReorderCellDto,
) -> Result<dto::CellDto, CellError> {
    authorize_cell(&app_state, tenant_context, id, Action::Write).await?;

    let tenant_id = tenant_context.tenant_id();
    let cell = execute_get_cell(&app_state, tenant_id, id).await?;

    let mut ordered_cells = repository::get_cells_by_notebook_id(
        &app_state.database_connection,
        tenant_id,
        cell.notebook_id,
    )
    .await?;

    let cell_count = ordered_cells.len() as i32;
    let current_index = cell.display_order;
    if current_index < 1 || current_index > cell_count {
        return Err(CellError::Internal(
            CELL_DISPLAY_ORDER_OUT_OF_BOUNDS_MESSAGE.to_string(),
        ));
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
            if current_index < cell_count {
                current_index + 1
            } else {
                current_index
            }
        }
        CellDisplayOrderMovement::Top => 1,
        CellDisplayOrderMovement::Bottom => cell_count,
    };
    if new_index != current_index {
        let cell_model = ordered_cells.remove((current_index - 1) as usize);
        ordered_cells.insert((new_index - 1) as usize, cell_model);
    }

    let transaction = app_state.database_connection.begin().await?;
    match persist_cell_order(&transaction, tenant_id, &ordered_cells, id, cell.clone()).await {
        Ok(reordered_cell) => {
            transaction.commit().await?;
            Ok(dto::CellDto::from(reordered_cell))
        }
        Err(error) => {
            let _ = transaction.rollback().await;
            Err(error)
        }
    }
}

async fn persist_cell_order(
    transaction: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    ordered_cells: &[entity::cell::Model],
    reordered_cell_id: Uuid,
    original_cell: entity::cell::Model,
) -> Result<entity::cell::Model, CellError> {
    let mut reordered_cell = original_cell;
    for (position, cell) in ordered_cells.iter().enumerate() {
        let new_display_order = (position + 1) as i32;
        if cell.display_order == new_display_order {
            continue;
        }
        let updated = repository::update_cell_display_order_transaction(
            transaction,
            tenant_id,
            cell.id,
            new_display_order,
        )
        .await?
        .ok_or(CellError::NotFound)?;
        if cell.id == reordered_cell_id {
            reordered_cell = updated;
        }
    }
    Ok(reordered_cell)
}

pub async fn update_cell(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    payload: dto::CellDto,
) -> Result<dto::CellDto, CellError> {
    authorize_cell(&app_state, tenant_context, id, Action::Write).await?;

    let cell = repository::update_cell(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        id,
        payload,
    )
    .await?
    .ok_or(CellError::NotFound)?;

    Ok(dto::CellDto::from(cell))
}

pub async fn execute_cell(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
) -> Result<QueryResult, CellError> {
    authorize_cell(&app_state, tenant_context, id, Action::Read).await?;

    let tenant_id = tenant_context.tenant_id();
    let cell = execute_get_cell(&app_state, tenant_id, id).await?;
    let cell_type = CellType::from_str(&cell.cell_type).map_err(CellError::Internal)?;

    match cell_type {
        CellType::Sql => execute_sql_cell(&app_state, tenant_id, cell).await,
        CellType::Chart => execute_chart_cell(&app_state, tenant_id, cell).await,
        _ => Err(CellError::UnsupportedCellType),
    }
}

async fn fetch_connection(
    app_state: &AppState,
    tenant_id: &TenantId,
    connection_id: Uuid,
) -> Result<entity::connection::Model, GetDatabaseConnectionError> {
    connection_service::execute_get_connection(app_state, tenant_id, connection_id)
        .await
        .map_err(|e| match &e {
            sea_orm::DbErr::RecordNotFound(_) => GetDatabaseConnectionError::ConnectionNotFound,
            _ => GetDatabaseConnectionError::Connection(e),
        })
}

fn strip_trailing_semicolons(query: &str) -> String {
    let mut trimmed = query.trim_end();
    while let Some(stripped) = trimmed.strip_suffix(';') {
        trimmed = stripped.trim_end();
    }
    trimmed.to_string()
}

fn validate_identifier(value: &str) -> Result<&str, ExecuteChartError> {
    let invalid = value.is_empty()
        || value.len() > MAX_IDENTIFIER_LEN
        || value.chars().any(|c| c == '"' || c.is_control());
    if invalid {
        return Err(ExecuteChartError::InvalidIdentifier(value.to_string()));
    }
    Ok(value)
}

fn build_aggregated_query(
    source_query: &str,
    x_axis: &str,
    x_alias: &str,
    series: &[dto::ChartSeries],
    sort_by: &str,
    y_axis_sort_order: &str,
) -> Result<String, ExecuteChartError> {
    validate_identifier(x_axis)?;
    validate_identifier(x_alias)?;
    let x_projection = format!("\"{}\" AS \"{}\"", x_axis, x_alias);
    let mut y_projections = Vec::with_capacity(series.len());
    for entry in series {
        let aggregate_fn = entry
            .aggregate_function
            .as_ref()
            .ok_or(ExecuteChartError::MissingChartSetting("aggregate function"))?;
        validate_identifier(&entry.column)?;
        let alias = validate_identifier(entry.alias.as_str())?;
        y_projections.push(format!(
            "{}(\"{}\") AS \"{}\"",
            aggregate_fn.as_str(),
            entry.column,
            alias
        ));
    }
    let projection = format!("{}, {}", x_projection, y_projections.join(", "));
    if y_axis_sort_order != SortOrder::None.as_str() {
        return Ok(format!(
            "SELECT {} FROM ({}) AS subquery GROUP BY \"{}\" ORDER BY \"{}\" {}",
            projection, source_query, x_axis, sort_by, y_axis_sort_order
        ));
    }
    Ok(format!(
        "SELECT {} FROM ({}) AS subquery GROUP BY \"{}\"",
        projection, source_query, x_axis
    ))
}

fn build_pie_chart_aggregated_query(
    source_query: &str,
    category: &str,
    value: &str,
    aggregate_fn: &AggregateFunction,
) -> Result<String, ExecuteChartError> {
    validate_identifier(category)?;
    validate_identifier(value)?;
    Ok(format!(
        "SELECT \"{}\", {}(\"{}\") AS \"{}\" FROM ({}) AS subquery GROUP BY \"{}\"",
        category,
        aggregate_fn.as_str(),
        value,
        value,
        source_query,
        category
    ))
}

fn get_sql_query_from_content(content: &str) -> Result<String, CellError> {
    match CellContent::parse(content, &CellType::Sql) {
        Ok(CellContent::Sql(sql_content)) => Ok(sql_content.query),
        Ok(_) => Err(CellError::InvalidSqlContent),
        Err(_) => Ok(content.to_string()),
    }
}

fn get_sql_query_from_cell_data(cell: &entity::cell::Model) -> Result<String, ExecuteChartError> {
    let content = cell
        .content
        .as_ref()
        .ok_or(ExecuteChartError::MissingContent)?;
    match CellContent::parse(content, &CellType::Sql) {
        Ok(CellContent::Sql(sql_content)) => Ok(sql_content.query),
        Ok(_) => Err(ExecuteChartError::SourceNotSql),
        Err(_) => Ok(content.to_string()),
    }
}

fn get_sql_query_from_cell(cell: &entity::cell::Model) -> Result<String, CellError> {
    get_sql_query_from_cell_data(cell).map_err(CellError::ExecuteChart)
}

fn parse_chart_content(content: &str) -> Result<dto::ChartContent, ExecuteChartError> {
    match CellContent::parse(content, &CellType::Chart) {
        Ok(CellContent::Chart(c)) => Ok(c),
        Ok(_) => Err(ExecuteChartError::InvalidChartContent),
        Err(e) => {
            tracing::error!("Error when parsing chart content: {}", e);
            Err(ExecuteChartError::InvalidChartContent)
        }
    }
}

pub async fn execute_cell_preview(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    payload: dto::ExecuteCellPreviewDto,
) -> Result<QueryResult, CellError> {
    authorize_cell(&app_state, tenant_context, id, Action::Write).await?;

    let tenant_id = tenant_context.tenant_id();
    let cell = execute_get_cell(&app_state, tenant_id, id).await?;

    match payload.cell_type {
        CellType::Sql => {
            let connection_id = cell.connection_id.ok_or(CellError::MissingConnection)?;
            let connection = fetch_connection(&app_state, tenant_id, connection_id).await?;
            let query = get_sql_query_from_content(&payload.content)?;
            execute_sql_with_query(&app_state, &connection, &query).await
        }
        CellType::Chart => {
            let chart_content = parse_chart_content(&payload.content)?;
            Ok(execute_chart_content_data(&app_state, tenant_id, &chart_content).await?)
        }
        _ => Err(CellError::UnsupportedPreviewCellType),
    }
}

async fn execute_sql_with_query(
    app_state: &AppState,
    connection: &entity::connection::Model,
    query: &str,
) -> Result<QueryResult, CellError> {
    Ok(database_service::execute_sql_query(app_state, connection, query).await?)
}

async fn execute_sql_cell(
    app_state: &AppState,
    tenant_id: &TenantId,
    cell: entity::cell::Model,
) -> Result<QueryResult, CellError> {
    let connection_id = cell.connection_id.ok_or(CellError::MissingConnection)?;
    let connection = fetch_connection(app_state, tenant_id, connection_id).await?;
    let query = get_sql_query_from_cell(&cell)?;
    execute_sql_with_query(app_state, &connection, &query).await
}

fn map_cell_lookup_error(err: sea_orm::DbErr) -> ExecuteChartError {
    match err {
        sea_orm::DbErr::RecordNotFound(_) => ExecuteChartError::CellNotFound,
        other => ExecuteChartError::Repository(other),
    }
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
    tenant_id: &TenantId,
    source_cell_id: Uuid,
) -> Result<(entity::connection::Model, String), ExecuteChartError> {
    let source_cell = execute_get_cell(app_state, tenant_id, source_cell_id)
        .await
        .map_err(map_cell_lookup_error)?;

    let connection_id = source_cell
        .connection_id
        .ok_or(ExecuteChartError::MissingConnection)?;

    let connection = fetch_connection(app_state, tenant_id, connection_id)
        .await
        .map_err(|e| ExecuteChartError::ExecuteSql(e.into()))?;

    let source_query = strip_trailing_semicolons(&get_sql_query_from_cell_data(&source_cell)?);

    Ok((connection, source_query))
}

fn validated_output_names<'a>(
    x_alias: &'a str,
    series: &'a [dto::ChartSeries],
) -> Result<HashSet<&'a str>, ExecuteChartError> {
    if series.is_empty() {
        return Err(ExecuteChartError::MissingChartSetting("series"));
    }
    if series.len() > MAX_CHART_SERIES {
        return Err(ExecuteChartError::TooManySeries {
            count: series.len(),
            max: MAX_CHART_SERIES,
        });
    }

    let mut names = HashSet::with_capacity(series.len() + 1);
    names.insert(x_alias);
    for entry in series {
        if !names.insert(entry.alias.as_str()) {
            return Err(ExecuteChartError::DuplicateSeriesAlias(
                entry.alias.as_str().to_string(),
            ));
        }
    }
    Ok(names)
}

fn build_axis_chart_query(
    axis_content: &dto::AxisChartContent,
    source_query: &str,
) -> Result<String, ExecuteChartError> {
    let x_alias = axis_content.x_axis_alias.as_str();
    let names = validated_output_names(x_alias, &axis_content.series)?;

    let y_axis_sort_order = axis_content
        .y_axis_sort_order
        .as_ref()
        .ok_or(ExecuteChartError::MissingChartSetting("y-axis sort order"))?
        .as_str();

    let sort_by = axis_content
        .y_axis_sort_by
        .as_deref()
        .filter(|name| names.contains(name))
        .unwrap_or_else(|| axis_content.series[0].alias.as_str());

    build_aggregated_query(
        source_query,
        &axis_content.x_axis,
        x_alias,
        &axis_content.series,
        sort_by,
        y_axis_sort_order,
    )
}

fn build_pie_query(
    pie_content: &dto::PieChartContent,
    source_query: &str,
) -> Result<String, ExecuteChartError> {
    let aggregate_fn = pie_content
        .aggregate_function
        .as_deref()
        .ok_or(ExecuteChartError::MissingChartSetting("aggregate function"))?;
    let aggregate_fn = AggregateFunction::from_str(aggregate_fn)
        .map_err(|_| ExecuteChartError::InvalidIdentifier(aggregate_fn.to_string()))?;

    build_pie_chart_aggregated_query(
        source_query,
        &pie_content.category,
        &pie_content.value,
        &aggregate_fn,
    )
}

fn build_chart_aggregated_query(
    chart_content: &dto::ChartContent,
    source_query: &str,
) -> Result<String, ExecuteChartError> {
    match chart_content {
        dto::ChartContent::Bar(axis_content) | dto::ChartContent::Line(axis_content) => {
            build_axis_chart_query(axis_content, source_query)
        }
        dto::ChartContent::Pie(pie_content) => build_pie_query(pie_content, source_query),
        dto::ChartContent::Metric(_) => Ok(source_query.to_string()),
    }
}

async fn execute_chart_content_data(
    app_state: &AppState,
    tenant_id: &TenantId,
    chart_content: &dto::ChartContent,
) -> Result<QueryResult, ExecuteChartError> {
    let source_cell_id = get_source_cell_id(chart_content);
    let (connection, source_query) =
        get_source_cell_for_chart(app_state, tenant_id, source_cell_id).await?;
    let aggregated_query = build_chart_aggregated_query(chart_content, &source_query)?;
    Ok(database_service::execute_sql_query(app_state, &connection, &aggregated_query).await?)
}

async fn execute_chart_for_cell(
    app_state: &AppState,
    tenant_id: &TenantId,
    cell: &entity::cell::Model,
) -> Result<dto::ChartExecution, ExecuteChartError> {
    let content = cell
        .content
        .as_ref()
        .ok_or(ExecuteChartError::MissingContent)?;
    let chart_content = parse_chart_content(content)?;
    let result = execute_chart_content_data(app_state, tenant_id, &chart_content).await?;
    Ok(dto::ChartExecution {
        cell_id: cell.id,
        chart_name: cell.name.clone().unwrap_or_default(),
        chart_type: chart_content.chart_type().as_str().to_string(),
        field_names: chart_content.field_names(),
        result,
    })
}

pub async fn execute_chart_cell_data(
    app_state: &AppState,
    tenant_id: &TenantId,
    cell_id: Uuid,
) -> Result<dto::ChartExecution, ExecuteChartError> {
    let cell = execute_get_cell(app_state, tenant_id, cell_id)
        .await
        .map_err(map_cell_lookup_error)?;
    execute_chart_for_cell(app_state, tenant_id, &cell).await
}

pub async fn execute_chart_cells_data(
    app_state: &AppState,
    tenant_id: &TenantId,
    cell_ids: &[Uuid],
    concurrency: usize,
) -> Result<Vec<dto::ChartExecutionOutcome>, sea_orm::DbErr> {
    let cells =
        repository::get_cells_by_ids(&app_state.database_connection, tenant_id, cell_ids).await?;
    let cells_by_id: HashMap<Uuid, entity::cell::Model> =
        cells.into_iter().map(|cell| (cell.id, cell)).collect();

    let results = stream::iter(cell_ids.iter().copied().map(|cell_id| {
        let cells_by_id = &cells_by_id;
        async move {
            let cell = cells_by_id.get(&cell_id);
            dto::ChartExecutionOutcome {
                cell_id,
                chart_name: cell.and_then(|cell| cell.name.clone()).unwrap_or_default(),
                result: match cell {
                    Some(cell) => execute_chart_for_cell(app_state, tenant_id, cell).await,
                    None => Err(ExecuteChartError::CellNotFound),
                },
            }
        }
    }))
    .buffered(concurrency.max(1))
    .collect::<Vec<_>>()
    .await;

    Ok(results)
}

async fn execute_chart_cell(
    app_state: &AppState,
    tenant_id: &TenantId,
    cell: entity::cell::Model,
) -> Result<QueryResult, CellError> {
    match execute_chart_for_cell(app_state, tenant_id, &cell).await {
        Ok(execution) => Ok(execution.result),
        Err(ExecuteChartError::InvalidChartContent) => Err(CellError::NonSerializableContent),
        Err(other) => Err(CellError::ExecuteChart(other)),
    }
}

pub async fn get_last_modified_cells_by_type(
    app_state: &AppState,
    tenant_context: &TenantContext,
    cell_type: CellType,
    limit: u64,
) -> Result<Vec<entity::cell::Model>, sea_orm::DbErr> {
    repository::get_latest_cells_by_type(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        notebook_service::visible_notebook_ids_subquery(tenant_context),
        cell_type,
        limit,
    )
    .await
}

pub async fn search_cells_by_name_and_type(
    app_state: &AppState,
    tenant_context: &TenantContext,
    search_query: &str,
    cell_type: CellType,
    limit: u64,
) -> Result<Vec<entity::cell::Model>, sea_orm::DbErr> {
    repository::search_cells_by_name_and_type(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        notebook_service::visible_notebook_ids_subquery(tenant_context),
        search_query,
        cell_type,
        limit,
    )
    .await
}

pub async fn delete_cells_by_notebook_id_transaction(
    txn: &sea_orm::DatabaseTransaction,
    tenant_id: &TenantId,
    notebook_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    let cell_ids: Vec<Uuid> =
        repository::get_cells_by_notebook_id_transaction(txn, tenant_id, notebook_id)
            .await?
            .into_iter()
            .map(|cell| cell.id)
            .collect();
    repository::delete_cells_by_notebook_id_transaction(txn, tenant_id, notebook_id).await?;
    ai_summary_service::delete_chart_summaries(txn, tenant_id, &cell_ids).await
}

pub async fn get_cell_counts_by_canvas_id(
    app_state: &AppState,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<(i32, i32), SophoError> {
    let notebook = notebook_service::get_notebook_by_canvas_id(app_state, tenant_id, canvas_id)
        .await
        .map_err(SophoError::DatabaseError)?;

    let sql_count = repository::count_cells_by_notebook_id_and_cell_type(
        &app_state.database_connection,
        tenant_id,
        notebook.id,
        CellType::Sql,
    )
    .await
    .map_err(SophoError::DatabaseError)?;

    let chart_count = repository::count_cells_by_notebook_id_and_cell_type(
        &app_state.database_connection,
        tenant_id,
        notebook.id,
        CellType::Chart,
    )
    .await
    .map_err(SophoError::DatabaseError)?;

    Ok((sql_count as i32, chart_count as i32))
}
