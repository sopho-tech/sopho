use crate::canvas::service as canvas_service;
use crate::cell::constants::CellType;
use crate::cell::service as cell_service;
use crate::common::AppState;
use crate::conversational_analytics::conversation::service as conversation_service;
use crate::notebook::service as notebook_service;
use crate::search::constants::{EntityType, SEARCH_LIMIT_PER_ENTITY};
use crate::search::dto::{SearchRequestDto, SearchResultItemDto};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

const SUPPORTED_ENTITY_TYPES: [EntityType; 4] = [
    EntityType::Canvas,
    EntityType::SqlCell,
    EntityType::ChartCell,
    EntityType::Conversation,
];

async fn search_canvases(
    app_state: &AppState,
    search_query: &str,
    limit: u64,
) -> Result<Vec<SearchResultItemDto>, sea_orm::DbErr> {
    let canvases = if search_query.is_empty() {
        canvas_service::get_last_modified_canvases(app_state, limit).await?
    } else {
        canvas_service::search_canvases_by_name(app_state, search_query, limit).await?
    };

    Ok(canvases
        .into_iter()
        .map(|canvas| SearchResultItemDto {
            id: canvas.id,
            name: Some(canvas.name),
            entity_type: EntityType::Canvas.as_str().to_string(),
            updated_at: canvas.updated_at,
            canvas_id: None,
        })
        .collect())
}

async fn search_conversations(
    app_state: &AppState,
    search_query: &str,
    limit: u64,
) -> Result<Vec<SearchResultItemDto>, sea_orm::DbErr> {
    let search_term = (!search_query.is_empty()).then_some(search_query);
    let conversations =
        conversation_service::search_conversations_by_name(app_state, search_term, limit).await?;

    Ok(conversations
        .into_iter()
        .map(|conversation| SearchResultItemDto {
            id: conversation.id,
            name: Some(conversation.name),
            entity_type: EntityType::Conversation.as_str().to_string(),
            updated_at: conversation.updated_at,
            canvas_id: None,
        })
        .collect())
}

async fn search_cells(
    app_state: &AppState,
    search_query: &str,
    cell_type: CellType,
    entity_type: EntityType,
    limit: u64,
) -> Result<Vec<SearchResultItemDto>, sea_orm::DbErr> {
    let cells = if search_query.is_empty() {
        cell_service::get_last_modified_cells_by_type(app_state, cell_type, limit).await?
    } else {
        cell_service::search_cells_by_name_and_type(app_state, search_query, cell_type, limit)
            .await?
    };

    let mut notebook_ids: Vec<Uuid> = cells.iter().map(|cell| cell.notebook_id).collect();
    notebook_ids.sort();
    notebook_ids.dedup();
    let canvas_id_by_notebook_id =
        notebook_service::get_canvas_ids_by_notebook_ids(app_state, notebook_ids).await?;

    Ok(cells
        .into_iter()
        .map(|cell| SearchResultItemDto {
            id: cell.id,
            name: cell.name,
            entity_type: entity_type.as_str().to_string(),
            updated_at: cell.updated_at,
            canvas_id: canvas_id_by_notebook_id.get(&cell.notebook_id).copied(),
        })
        .collect())
}

async fn search_by_entity_type(
    app_state: &AppState,
    search_query: &str,
    entity_type: EntityType,
    limit: u64,
) -> Result<Vec<SearchResultItemDto>, sea_orm::DbErr> {
    match entity_type {
        EntityType::Canvas => search_canvases(app_state, search_query, limit).await,
        EntityType::Conversation => search_conversations(app_state, search_query, limit).await,
        EntityType::SqlCell => {
            search_cells(app_state, search_query, CellType::Sql, entity_type, limit).await
        }
        EntityType::ChartCell => {
            search_cells(app_state, search_query, CellType::Chart, entity_type, limit).await
        }
    }
}

async fn run_search(
    app_state: &AppState,
    search_query: &str,
    filters: &[&str],
) -> Result<Vec<SearchResultItemDto>, sea_orm::DbErr> {
    let mut results: Vec<SearchResultItemDto> = Vec::new();

    for entity_type in SUPPORTED_ENTITY_TYPES {
        if !filters.contains(&entity_type.as_str()) {
            continue;
        }
        let mut entity_results = search_by_entity_type(
            app_state,
            search_query,
            entity_type,
            SEARCH_LIMIT_PER_ENTITY,
        )
        .await?;
        results.append(&mut entity_results);
    }

    results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(results)
}

pub async fn search(app_state: AppState, request: SearchRequestDto) -> impl IntoResponse {
    let search_query = request.query.as_deref().unwrap_or("");
    let requested_filters = request.filters.unwrap_or_default();
    let filters = requested_filters
        .iter()
        .map(|filter| filter.as_str())
        .collect::<Vec<_>>();

    let has_supported_filter = SUPPORTED_ENTITY_TYPES
        .iter()
        .any(|entity_type| filters.contains(&entity_type.as_str()));

    if !has_supported_filter {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "At least one filter type must be enabled" })),
        );
    }

    match run_search(&app_state, search_query, &filters).await {
        Ok(results) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "data": results })),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": error.to_string() })),
        ),
    }
}
