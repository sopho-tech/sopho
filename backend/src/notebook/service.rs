use crate::authorization::constants::Action;
use crate::authorization::dto::Actor;
use crate::authorization::dto::{TenantContext, TenantId};
use crate::authorization::errors::AuthorizationError;
use crate::cell::constants::CellType;
use crate::cell::service as cell_service;
use crate::common::time_utils;
use crate::common::{AppState, PaginatedResponse};
use crate::entity;
use crate::notebook::constants::{NOTEBOOK_NOT_FOUND_MESSAGE, NotebookStatus};
use crate::notebook::dto;
use crate::notebook::error::NotebookError;
use crate::notebook::repository;
use sea_orm::{ConnectionTrait, DatabaseTransaction};
use std::collections::HashMap;
use uuid::Uuid;

pub async fn does_notebook_exist(app_state: &AppState, tenant_id: &TenantId, id: Uuid) -> bool {
    matches!(
        repository::get_notebook(&app_state.database_connection, tenant_id, id).await,
        Ok(Some(_))
    )
}

pub async fn get_notebook(
    app_state: AppState,
    tenant_context: &TenantContext,
    id: Uuid,
) -> Result<dto::NotebookDto, NotebookError> {
    let notebook = authorize_notebook(&app_state, tenant_context, id, Action::Read).await?;
    let cells = cell_service::get_cells_by_notebook_id_and_cell_type(
        &app_state,
        tenant_context.tenant_id(),
        id,
        None,
    )
    .await
    .map_err(|error| NotebookError::Internal(error.to_string()))?;

    let mut response_dto = dto::NotebookDto::from(notebook);
    response_dto.cells = cells;
    Ok(response_dto)
}

pub async fn get_all_notebooks(
    app_state: AppState,
    tenant_context: &TenantContext,
    page: u64,
    page_size: u64,
) -> Result<PaginatedResponse<dto::NotebookDto>, NotebookError> {
    crate::authorization::service::require_capability(
        tenant_context,
        crate::authorization::constants::ResourceType::Canvas.permission_for(Action::Read),
    )?;

    let (notebooks, total_items, total_pages) = repository::get_paginated_visible_notebooks(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        crate::canvas::service::visible_canvas_ids_subquery(tenant_context),
        page,
        page_size,
    )
    .await?;

    Ok(PaginatedResponse {
        data: notebooks.into_iter().map(dto::NotebookDto::from).collect(),
        total_pages,
        page,
        page_size,
        total_items,
    })
}

pub async fn get_cells_by_notebook_id(
    app_state: AppState,
    tenant_context: &TenantContext,
    notebook_id: Uuid,
    cell_type: Option<CellType>,
) -> Result<dto::NotebookDto, NotebookError> {
    let notebook =
        authorize_notebook(&app_state, tenant_context, notebook_id, Action::Read).await?;

    let cells = cell_service::get_cells_by_notebook_id_and_cell_type(
        &app_state,
        tenant_context.tenant_id(),
        notebook_id,
        cell_type,
    )
    .await
    .map_err(|error| NotebookError::Internal(error.to_string()))?;

    let mut response_dto = dto::NotebookDto::from(notebook);
    response_dto.cells = cells;
    Ok(response_dto)
}

pub async fn get_notebooks_by_canvas_id(
    app_state: AppState,
    tenant_context: &TenantContext,
    canvas_id: Uuid,
) -> Result<Vec<dto::NotebookDto>, NotebookError> {
    authorize_canvas_for_children(&app_state, tenant_context, canvas_id).await?;

    let notebooks = repository::get_notebooks_by_canvas_id_connection(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        canvas_id,
    )
    .await?;

    Ok(notebooks.into_iter().map(dto::NotebookDto::from).collect())
}

pub async fn create_notebook(
    app_state: AppState,
    tenant_context: &TenantContext,
    payload: dto::CreateNotebookDto,
) -> Result<dto::NotebookDto, NotebookError> {
    crate::authorization::service::require_capability(
        tenant_context,
        crate::authorization::constants::Permission::CanvasWrite,
    )?;

    let tenant_id = tenant_context.tenant_id();
    let actor = tenant_context.actor();
    let notebook = entity::notebook::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
        canvas_id: Uuid::new_v4(),
        name: payload.name,
        description: payload.description,
        status: NotebookStatus::Active.to_string(),
        created_by_user_id: actor.user_id(),
        created_by_actor: actor.kind_name(),
        modified_by_user_id: actor.user_id(),
        modified_by_actor: actor.kind_name(),
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };
    let notebook =
        repository::save_notebook_connection(&app_state.database_connection, tenant_id, notebook)
            .await?;

    Ok(dto::NotebookDto::from(notebook))
}

pub async fn create_notebook_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    canvas_id: Uuid,
    name: String,
    description: Option<String>,
    actor: &Actor,
) -> Result<entity::notebook::Model, sea_orm::DbErr> {
    let notebook = entity::notebook::Model {
        id: Uuid::new_v4(),
        tenant_id: tenant_id.get_uuid(),
        canvas_id,
        name,
        description,
        status: NotebookStatus::Active.to_string(),
        created_by_user_id: actor.user_id(),
        created_by_actor: actor.kind_name(),
        modified_by_user_id: actor.user_id(),
        modified_by_actor: actor.kind_name(),
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };
    repository::save_notebook_transaction(txn, tenant_id, notebook).await
}

pub async fn get_notebook_by_canvas_id(
    app_state: &AppState,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<entity::notebook::Model, sea_orm::DbErr> {
    repository::get_notebook_by_canvas_id(&app_state.database_connection, tenant_id, canvas_id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Notebook not found".into()))
}

pub async fn get_canvas_ids_by_notebook_ids(
    app_state: &AppState,
    tenant_id: &TenantId,
    notebook_ids: Vec<Uuid>,
) -> Result<HashMap<Uuid, Uuid>, sea_orm::DbErr> {
    let notebooks =
        repository::get_notebooks_by_ids(&app_state.database_connection, tenant_id, notebook_ids)
            .await?;
    Ok(notebooks
        .into_iter()
        .map(|notebook| (notebook.id, notebook.canvas_id))
        .collect())
}

pub async fn get_notebook_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<entity::notebook::Model, sea_orm::DbErr> {
    repository::get_notebook_transaction(txn, tenant_id, id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Notebook not found".into()))
}

pub async fn get_notebook_by_canvas_id_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    canvas_id: Uuid,
) -> Result<entity::notebook::Model, sea_orm::DbErr> {
    repository::get_notebook_by_canvas_id_transaction(txn, tenant_id, canvas_id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Notebook not found".into()))
}

pub async fn delete_notebook_transaction(
    txn: &DatabaseTransaction,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    repository::delete_notebook_transaction(txn, tenant_id, id).await
}

pub async fn get_notebook_entity(
    app_state: &AppState,
    tenant_id: &TenantId,
    id: Uuid,
) -> Result<entity::notebook::Model, sea_orm::DbErr> {
    repository::get_notebook(&app_state.database_connection, tenant_id, id)
        .await?
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound("Notebook not found".into()))
}

pub fn visible_notebook_ids_subquery(
    tenant_context: &TenantContext,
) -> sea_orm::sea_query::SelectStatement {
    repository::visible_notebook_ids_subquery(
        tenant_context.tenant_id(),
        crate::canvas::service::visible_canvas_ids_subquery(tenant_context),
    )
}

pub async fn authorize_notebook(
    app_state: &AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    action: Action,
) -> Result<entity::notebook::Model, NotebookError> {
    let record = repository::get_notebook(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        id,
    )
    .await
    .map_err(|_| NotebookError::NotFound)?
    .ok_or(NotebookError::NotFound)?;

    let subject = crate::canvas::service::load_authorization_subject(
        app_state,
        tenant_context.tenant_id(),
        record.canvas_id,
    )
    .await
    .map_err(|_| NotebookError::NotFound)?;

    crate::authorization::service::require_access(app_state, tenant_context, &subject, action)
        .await?;

    Ok(record)
}

pub async fn require_notebook_access(
    app_state: &AppState,
    tenant_context: &TenantContext,
    id: Uuid,
    action: Action,
) -> Result<(), AuthorizationError> {
    match authorize_notebook(app_state, tenant_context, id, action).await {
        Ok(_) => Ok(()),
        Err(NotebookError::Unauthorized(authorization_error)) => Err(authorization_error),
        Err(NotebookError::Database(database_error)) => {
            Err(AuthorizationError::Database(database_error))
        }
        Err(_) => Err(AuthorizationError::NotReachable),
    }
}

async fn authorize_canvas_for_children(
    app_state: &AppState,
    tenant_context: &TenantContext,
    canvas_id: Uuid,
) -> Result<(), NotebookError> {
    let subject = crate::canvas::service::load_authorization_subject(
        app_state,
        tenant_context.tenant_id(),
        canvas_id,
    )
    .await
    .map_err(|_| NotebookError::CanvasNotFound)?;

    crate::authorization::service::require_access(
        app_state,
        tenant_context,
        &subject,
        Action::Read,
    )
    .await?;

    Ok(())
}

pub async fn canvas_id_for_notebook(
    connection: &impl ConnectionTrait,
    tenant_id: &TenantId,
    notebook_id: Uuid,
) -> Result<Uuid, sea_orm::DbErr> {
    repository::get_notebook(connection, tenant_id, notebook_id)
        .await?
        .map(|notebook| notebook.canvas_id)
        .ok_or_else(|| sea_orm::DbErr::RecordNotFound(NOTEBOOK_NOT_FOUND_MESSAGE.into()))
}
