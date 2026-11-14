use crate::authentication::service::{AccessTokenValidationError, authenticate_from_cookies};
use crate::authorization::constants::{
    ACCESS_TOKEN_EXPIRED_MESSAGE, ACCESS_TOKEN_REQUIRED_MESSAGE, INVALID_ACCESS_TOKEN_MESSAGE,
    MEMBERSHIP_SUSPENDED_MESSAGE, TENANT_NOT_FOUND_MESSAGE, TENANT_SUSPENDED_MESSAGE,
};
use crate::authorization::dto::{AuthenticatedUser, TenantContext};
use crate::authorization::errors::TenantContextError;
use crate::authorization::service::resolve_tenant_context;
use crate::common::AppState;
use crate::common::error_codes::{ErrorCode, codes};
use axum::Json;
use axum::extract::{FromRequestParts, Path};
use axum::http::StatusCode;
use axum::http::request::Parts;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct TenantPath {
    tenant_id: Uuid,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(authenticated_user) = parts.extensions.get::<AuthenticatedUser>() {
            return Ok(authenticated_user.clone());
        }
        let cookies = Cookies::from_request_parts(parts, app_state)
            .await
            .map_err(|(status, message)| (status, Json(serde_json::json!({ "error": message }))))?;
        let authenticated_user = authenticate_from_cookies(&cookies, app_state)
            .await
            .map_err(access_token_error_response)?;
        parts.extensions.insert(authenticated_user.clone());
        Ok(authenticated_user)
    }
}

impl FromRequestParts<AppState> for TenantContext {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let authenticated_user = AuthenticatedUser::from_request_parts(parts, app_state).await?;

        let Path(TenantPath { tenant_id }) =
            Path::<TenantPath>::from_request_parts(parts, app_state)
                .await
                .map_err(|_| tenant_not_found())?;

        resolve_tenant_context(app_state, authenticated_user.user_id(), tenant_id)
            .await
            .map_err(|error| match error {
                TenantContextError::NotAMember => tenant_not_found(),
                TenantContextError::MembershipSuspended => (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({ "message": MEMBERSHIP_SUSPENDED_MESSAGE })),
                ),
                TenantContextError::TenantSuspended => (
                    StatusCode::FORBIDDEN,
                    Json(serde_json::json!({
                        "message": TENANT_SUSPENDED_MESSAGE,
                        "error_code": ErrorCode::TenantSuspended.as_str(),
                    })),
                ),
                TenantContextError::Database(database_error) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": database_error.to_string() })),
                ),
                TenantContextError::UnknownPermission(permission) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": format!("Unknown permission in role: {}", permission)
                    })),
                ),
            })
    }
}

fn access_token_error_response(
    error: AccessTokenValidationError,
) -> (StatusCode, Json<serde_json::Value>) {
    let (message, error_code) = match error {
        AccessTokenValidationError::Missing => {
            (ACCESS_TOKEN_REQUIRED_MESSAGE, codes::INVALID_ACCESS_TOKEN)
        }
        AccessTokenValidationError::Invalid => {
            (INVALID_ACCESS_TOKEN_MESSAGE, codes::INVALID_ACCESS_TOKEN)
        }
        AccessTokenValidationError::Expired => {
            (ACCESS_TOKEN_EXPIRED_MESSAGE, codes::ACCESS_TOKEN_EXPIRED)
        }
    };
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({ "message": message, "error_code": error_code.as_str() })),
    )
}

fn tenant_not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "message": TENANT_NOT_FOUND_MESSAGE })),
    )
}
