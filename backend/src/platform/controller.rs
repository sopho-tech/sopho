use crate::ai_usage::dto::{UsageListQuery, UsageRangeQuery};
use crate::authentication::service::PasswordFlowError;
use crate::authorization::dto::AuthenticatedUser;
use crate::common::AppState;
use crate::email_delivery::error::EmailDeliveryError;
use crate::platform::dto::{EmailLogQuery, UserListQuery};
use crate::platform::error::PlatformError;
use crate::platform::service;
use axum::Router;
use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Deserialize)]
struct UserPath {
    user_id: Uuid,
}

#[derive(Deserialize)]
struct OwnerPath {
    tenant_id: Uuid,
    user_id: Uuid,
}

#[derive(Deserialize)]
struct TenantPath {
    tenant_id: Uuid,
}

#[derive(Deserialize)]
struct EmailPath {
    email_id: Uuid,
}

#[derive(Deserialize)]
struct UsageConversationPath {
    tenant_id: Uuid,
    conversation_id: Uuid,
}

pub fn routes(app_state: AppState) -> Router {
    Router::new()
        .route("/user", get(list_users))
        .route("/user/{user_id}/deactivate", post(deactivate_user))
        .route("/user/{user_id}/reactivate", post(reactivate_user))
        .route("/user/{user_id}/password_reset", post(send_password_reset))
        .route(
            "/user/{user_id}/make_administrator",
            post(make_administrator),
        )
        .route(
            "/user/{user_id}/remove_administrator",
            post(remove_administrator),
        )
        .route("/tenant", get(list_tenants))
        .route("/ai_usage/summary", get(get_platform_ai_usage_summary))
        .route("/ai_usage/over_time", get(get_platform_ai_usage_over_time))
        .route("/ai_usage/by_kind", get(get_platform_ai_usage_by_kind))
        .route("/ai_usage/by_tenant", get(list_platform_ai_usage_by_tenant))
        .route("/ai_usage/by_user", get(list_platform_ai_usage_by_user))
        .route(
            "/ai_usage/by_conversation",
            get(list_platform_ai_usage_by_conversation),
        )
        .route(
            "/ai_usage/tenant/{tenant_id}/conversation/{conversation_id}/questions",
            get(get_platform_ai_usage_of_conversation_questions),
        )
        .route("/tenant/{tenant_id}/owner/{user_id}", put(appoint_owner))
        .route("/tenant/{tenant_id}/suspend", post(suspend_tenant))
        .route("/tenant/{tenant_id}/restore", post(restore_tenant))
        .route("/email", get(list_email_log))
        .route("/email_template", get(list_email_template_previews))
        .route("/email/{email_id}/retry", post(retry_email))
        .with_state(app_state)
}

async fn list_users(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(query): Query<UserListQuery>,
) -> Response {
    match service::list_users(&app_state, authenticated_user.user_id(), query).await {
        Ok(user_page) => Json(user_page).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn deactivate_user(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(UserPath { user_id }): Path<UserPath>,
) -> Response {
    match service::deactivate_user(&app_state, authenticated_user.user_id(), user_id).await {
        Ok(deactivated) => Json(deactivated).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn reactivate_user(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(UserPath { user_id }): Path<UserPath>,
) -> Response {
    match service::reactivate_user(&app_state, authenticated_user.user_id(), user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn send_password_reset(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(UserPath { user_id }): Path<UserPath>,
) -> Response {
    match service::send_password_reset(&app_state, authenticated_user.user_id(), user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn make_administrator(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(UserPath { user_id }): Path<UserPath>,
) -> Response {
    match service::make_administrator(&app_state, authenticated_user.user_id(), user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn remove_administrator(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(UserPath { user_id }): Path<UserPath>,
) -> Response {
    match service::remove_administrator(&app_state, authenticated_user.user_id(), user_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn list_tenants(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Response {
    match service::list_tenants(&app_state, authenticated_user.user_id()).await {
        Ok(tenants) => Json(tenants).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn get_platform_ai_usage_summary(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(usage_range_query): Query<UsageRangeQuery>,
) -> Response {
    match service::get_platform_ai_usage_summary(
        &app_state,
        authenticated_user.user_id(),
        &usage_range_query,
    )
    .await
    {
        Ok(usage_summary) => Json(usage_summary).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn get_platform_ai_usage_over_time(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(usage_range_query): Query<UsageRangeQuery>,
) -> Response {
    match service::get_platform_ai_usage_over_time(
        &app_state,
        authenticated_user.user_id(),
        &usage_range_query,
    )
    .await
    {
        Ok(usage_over_time) => Json(usage_over_time).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn get_platform_ai_usage_by_kind(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(usage_range_query): Query<UsageRangeQuery>,
) -> Response {
    match service::get_platform_ai_usage_by_kind(
        &app_state,
        authenticated_user.user_id(),
        &usage_range_query,
    )
    .await
    {
        Ok(usage_by_kind) => Json(usage_by_kind).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn list_platform_ai_usage_by_tenant(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(usage_range_query): Query<UsageRangeQuery>,
    Query(usage_list_query): Query<UsageListQuery>,
) -> Response {
    match service::list_platform_ai_usage_by_tenant(
        &app_state,
        authenticated_user.user_id(),
        &usage_range_query,
        &usage_list_query,
    )
    .await
    {
        Ok(usage_by_tenant) => Json(usage_by_tenant).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn list_platform_ai_usage_by_user(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(usage_range_query): Query<UsageRangeQuery>,
    Query(usage_list_query): Query<UsageListQuery>,
) -> Response {
    match service::list_platform_ai_usage_by_user(
        &app_state,
        authenticated_user.user_id(),
        &usage_range_query,
        &usage_list_query,
    )
    .await
    {
        Ok(usage_by_user) => Json(usage_by_user).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn list_platform_ai_usage_by_conversation(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(usage_range_query): Query<UsageRangeQuery>,
    Query(usage_list_query): Query<UsageListQuery>,
) -> Response {
    match service::list_platform_ai_usage_by_conversation(
        &app_state,
        authenticated_user.user_id(),
        &usage_range_query,
        &usage_list_query,
    )
    .await
    {
        Ok(usage_by_conversation) => Json(usage_by_conversation).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn get_platform_ai_usage_of_conversation_questions(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(UsageConversationPath {
        tenant_id,
        conversation_id,
    }): Path<UsageConversationPath>,
    Query(usage_range_query): Query<UsageRangeQuery>,
) -> Response {
    match service::get_platform_ai_usage_of_conversation_questions(
        &app_state,
        authenticated_user.user_id(),
        &usage_range_query,
        tenant_id,
        conversation_id,
    )
    .await
    {
        Ok(conversation_questions) => Json(conversation_questions).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn appoint_owner(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(OwnerPath { tenant_id, user_id }): Path<OwnerPath>,
) -> Response {
    match service::appoint_owner(&app_state, authenticated_user.user_id(), tenant_id, user_id).await
    {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn suspend_tenant(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(TenantPath { tenant_id }): Path<TenantPath>,
) -> Response {
    match service::suspend_tenant(&app_state, authenticated_user.user_id(), tenant_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn restore_tenant(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(TenantPath { tenant_id }): Path<TenantPath>,
) -> Response {
    match service::restore_tenant(&app_state, authenticated_user.user_id(), tenant_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn list_email_log(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Query(query): Query<EmailLogQuery>,
) -> Response {
    match service::list_email_log(&app_state, authenticated_user.user_id(), query).await {
        Ok(email_page) => Json(email_page).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn list_email_template_previews(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
) -> Response {
    match service::list_email_template_previews(&app_state, authenticated_user.user_id()).await {
        Ok(email_templates) => Json(email_templates).into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

async fn retry_email(
    State(app_state): State<AppState>,
    authenticated_user: AuthenticatedUser,
    Path(EmailPath { email_id }): Path<EmailPath>,
) -> Response {
    match service::retry_email(&app_state, authenticated_user.user_id(), email_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error) => platform_error_response(error).into_response(),
    }
}

fn platform_error_response(error: PlatformError) -> (StatusCode, Json<Value>) {
    let status = match &error {
        PlatformError::Unauthorized(_)
        | PlatformError::PasswordReset(PasswordFlowError::PasswordSignInOff) => {
            StatusCode::FORBIDDEN
        }
        PlatformError::UserNotFound
        | PlatformError::TenantNotFound
        | PlatformError::EmailDelivery(EmailDeliveryError::NotFound) => StatusCode::NOT_FOUND,
        PlatformError::AiUsage(ai_usage_error) if ai_usage_error.is_invalid_request() => {
            StatusCode::BAD_REQUEST
        }
        PlatformError::CannotDeactivateYourself
        | PlatformError::CannotRemoveYourOwnAdministratorRole
        | PlatformError::LastPlatformAdministrator
        | PlatformError::AppointeeDeactivated
        | PlatformError::AccountDeactivated
        | PlatformError::EmailDelivery(EmailDeliveryError::NotRetryable)
        | PlatformError::PasswordReset(PasswordFlowError::ResetUnavailable)
        | PlatformError::PasswordReset(PasswordFlowError::NoPassword) => StatusCode::CONFLICT,
        PlatformError::PasswordReset(PasswordFlowError::TooManyResetEmails) => {
            StatusCode::TOO_MANY_REQUESTS
        }
        PlatformError::PasswordReset(_)
        | PlatformError::EmailDelivery(_)
        | PlatformError::EmailTemplate(_)
        | PlatformError::Membership(_)
        | PlatformError::Tenant(_)
        | PlatformError::AiUsage(_)
        | PlatformError::Database(_)
        | PlatformError::Internal(_) => {
            tracing::error!("Platform request failed: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    (
        status,
        Json(serde_json::json!({
            "message": error.to_string(),
            "error_code": error.error_code().map(|error_code| error_code.as_str()),
        })),
    )
}
