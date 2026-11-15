use crate::ai_usage::constants::USAGE_NOT_FOUND_MESSAGE;
use crate::ai_usage::dto::{ConversationPageQuery, UsageRangeQuery};
use crate::ai_usage::error::AiUsageError;
use crate::ai_usage::service;
use crate::authorization::dto::TenantContext;
use crate::authorization::service::authorization_error_response;
use crate::common::AppState;
use axum::Router;
use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Deserialize)]
struct UsageConversationPath {
    conversation_id: Uuid,
}

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/summary", get(get_usage_summary))
        .route("/over_time", get(get_usage_over_time))
        .route("/by_user", get(get_usage_by_user))
        .route("/by_kind", get(get_usage_by_kind))
        .route("/by_conversation", get(list_usage_by_conversation))
        .route(
            "/conversation/{conversation_id}/questions",
            get(get_usage_of_conversation_questions),
        )
        .with_state(app_state)
}

async fn get_usage_summary(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Query(usage_range_query): Query<UsageRangeQuery>,
) -> Response {
    usage_response(
        service::get_usage_summary(&app_state, &tenant_context, &usage_range_query).await,
    )
}

async fn get_usage_over_time(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Query(usage_range_query): Query<UsageRangeQuery>,
) -> Response {
    usage_response(
        service::get_usage_over_time(&app_state, &tenant_context, &usage_range_query).await,
    )
}

async fn get_usage_by_user(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Query(usage_range_query): Query<UsageRangeQuery>,
) -> Response {
    usage_response(
        service::get_usage_by_user(&app_state, &tenant_context, &usage_range_query).await,
    )
}

async fn get_usage_by_kind(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Query(usage_range_query): Query<UsageRangeQuery>,
) -> Response {
    usage_response(
        service::get_usage_by_kind(&app_state, &tenant_context, &usage_range_query).await,
    )
}

async fn list_usage_by_conversation(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Query(usage_range_query): Query<UsageRangeQuery>,
    Query(conversation_page_query): Query<ConversationPageQuery>,
) -> Response {
    usage_response(
        service::list_usage_by_conversation(
            &app_state,
            &tenant_context,
            &usage_range_query,
            &conversation_page_query,
        )
        .await,
    )
}

async fn get_usage_of_conversation_questions(
    State(app_state): State<AppState>,
    tenant_context: TenantContext,
    Path(UsageConversationPath { conversation_id }): Path<UsageConversationPath>,
    Query(usage_range_query): Query<UsageRangeQuery>,
) -> Response {
    usage_response(
        service::get_usage_of_conversation_questions(
            &app_state,
            &tenant_context,
            &usage_range_query,
            conversation_id,
        )
        .await,
    )
}

fn usage_response<UsageBody: Serialize>(usage_result: Result<UsageBody, AiUsageError>) -> Response {
    match usage_result {
        Ok(usage_body) => (StatusCode::OK, Json(usage_body)).into_response(),
        Err(AiUsageError::Unauthorized(authorization_error)) => {
            authorization_error_response(authorization_error, USAGE_NOT_FOUND_MESSAGE)
                .into_response()
        }
        Err(
            usage_error @ (AiUsageError::RangeReversed
            | AiUsageError::RangeTooLong { .. }
            | AiUsageError::RangeAmbiguous
            | AiUsageError::PageOutOfRange
            | AiUsageError::RangeWithoutDays
            | AiUsageError::RangeOutOfCalendar),
        ) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": usage_error.to_string() })),
        )
            .into_response(),
        Err(
            usage_error @ (AiUsageError::Database(_)
            | AiUsageError::Metadata(_)
            | AiUsageError::TokenCountOutOfRange { .. }
            | AiUsageError::PlatformCallWithoutTokenCounts
            | AiUsageError::UnknownAgent { .. }
            | AiUsageError::UnreadableStoredValue(_)
            | AiUsageError::Conversation(_)),
        ) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": usage_error.to_string() })),
        )
            .into_response(),
    }
}
