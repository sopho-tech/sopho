use crate::canvas::service as canvas_service;
use crate::cell::constants::CellType;
use crate::cell::service as cell_service;
use crate::common::AppState;
use crate::search::constants::{EntityType, SEARCH_LIMIT_PER_ENTITY};
use crate::search::dto::{SearchRequestDto, SearchResultItemDto};
use axum::http::StatusCode;
use axum::response::IntoResponse;

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

    let mut results = Vec::new();
    for canvas in canvases {
        results.push(SearchResultItemDto {
            id: canvas.id,
            name: Some(canvas.name),
            entity_type: EntityType::Canvas.as_str().to_string(),
            updated_at: canvas.updated_at,
        });
    }

    Ok(results)
}

async fn search_sql_cells(
    app_state: &AppState,
    search_query: &str,
    limit: u64,
) -> Result<Vec<SearchResultItemDto>, sea_orm::DbErr> {
    let cells = if search_query.is_empty() {
        cell_service::get_last_modified_cells_by_type(app_state, CellType::Sql, limit).await?
    } else {
        cell_service::search_cells_by_name_and_type(app_state, search_query, CellType::Sql, limit)
            .await?
    };
    let mut results = Vec::new();

    for cell in cells {
        results.push(SearchResultItemDto {
            id: cell.id,
            name: cell.name.clone(),
            entity_type: EntityType::SqlCell.as_str().to_string(),
            updated_at: cell.updated_at,
        });
    }

    Ok(results)
}

async fn search_chart_cells(
    app_state: &AppState,
    search_query: &str,
    limit: u64,
) -> Result<Vec<SearchResultItemDto>, sea_orm::DbErr> {
    let cells = if search_query.is_empty() {
        cell_service::get_last_modified_cells_by_type(app_state, CellType::Chart, limit).await?
    } else {
        cell_service::search_cells_by_name_and_type(app_state, search_query, CellType::Chart, limit)
            .await?
    };
    let mut results = Vec::new();

    for cell in cells {
        results.push(SearchResultItemDto {
            id: cell.id,
            name: cell.name.clone(),
            entity_type: EntityType::ChartCell.as_str().to_string(),
            updated_at: cell.updated_at,
        });
    }

    Ok(results)
}

pub async fn search(app_state: AppState, request: SearchRequestDto) -> impl IntoResponse {
    let search_query = request.query.as_deref().unwrap_or("");
    let limit = SEARCH_LIMIT_PER_ENTITY;

    let filters = match &request.filters {
        Some(f) if !f.is_empty() => f.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(
                    serde_json::json!({ "error": "At least one filter type must be enabled" }),
                ),
            );
        }
    };

    let include_canvas = filters.contains(&EntityType::Canvas.as_str());
    let include_sql_cells = filters.contains(&EntityType::SqlCell.as_str());
    let include_chart_cells = filters.contains(&EntityType::ChartCell.as_str());

    if !include_canvas && !include_sql_cells && !include_chart_cells {
        return (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "At least one filter type must be enabled" })),
        );
    }

    let mut all_results: Vec<SearchResultItemDto> = Vec::new();

    if include_canvas {
        match search_canvases(&app_state, search_query, limit).await {
            Ok(mut results) => all_results.append(&mut results),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                );
            }
        }
    }

    if include_sql_cells {
        match search_sql_cells(&app_state, search_query, limit).await {
            Ok(mut results) => all_results.append(&mut results),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                );
            }
        }
    }

    if include_chart_cells {
        match search_chart_cells(&app_state, search_query, limit).await {
            Ok(mut results) => all_results.append(&mut results),
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                );
            }
        }
    }

    all_results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    (
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "data": all_results
        })),
    )
}
