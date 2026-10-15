use crate::authorization::dto::{AuthenticatedUser, TenantContext};
use crate::authorization::errors::TenantContextError;
use crate::authorization::service::resolve_tenant_context;
use crate::common::AppState;
use axum::Json;
use axum::extract::{FromRequestParts, Path};
use axum::http::StatusCode;
use axum::http::request::Parts;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct TenantPath {
    tenant_id: Uuid,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        _app_state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        parts.extensions.get::<AuthenticatedUser>().cloned().ok_or((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "message": "Authentication required" })),
        ))
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
                    Json(serde_json::json!({ "message": "Membership suspended" })),
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

fn tenant_not_found() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "message": "Tenant not found" })),
    )
}
