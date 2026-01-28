use crate::authentication::service::{validate_access_token, AccessTokenValidationError};
use crate::authentication::CookieName;
use crate::common::error_codes::codes;
use crate::common::AppState;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{Response, StatusCode};
use axum::middleware::Next;
use serde_json::json;

/// Creates an error response with the given status code, message, and error code
fn create_error_response(status: StatusCode, message: &str, error_code: &str) -> Response<Body> {
    let body = json!({
        "message": message,
        "error_code": error_code
    });

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap_or_default()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()
        })
}

/// Axum middleware function that validates access tokens
/// This function can be used with `axum::middleware::from_fn_with_state`
pub async fn auth_middleware_fn(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    cookies: tower_cookies::Cookies,
    req: Request,
    next: Next,
) -> Response<Body> {
    // Extract access token from cookies
    let access_token = cookies
        .get(&CookieName::AccessToken.to_string())
        .map(|cookie| cookie.value().to_string());

    match access_token {
        Some(token) => {
            // Validate the access token
            match validate_access_token(&token, &app_state).await {
                Ok(_) => {
                    // Token is valid, proceed with the request
                    next.run(req).await
                }
                Err(AccessTokenValidationError::Expired) => {
                    // Access token expired
                    create_error_response(
                        StatusCode::UNAUTHORIZED,
                        "Access token expired",
                        codes::ACCESS_TOKEN_EXPIRED.as_str(),
                    )
                }
                Err(AccessTokenValidationError::Invalid) => {
                    // Invalid access token
                    create_error_response(
                        StatusCode::UNAUTHORIZED,
                        "Invalid access token",
                        codes::INVALID_ACCESS_TOKEN.as_str(),
                    )
                }
                Err(AccessTokenValidationError::Missing) => {
                    // This shouldn't happen here, but handle it anyway
                    create_error_response(
                        StatusCode::UNAUTHORIZED,
                        "Access token required",
                        codes::INVALID_ACCESS_TOKEN.as_str(),
                    )
                }
            }
        }
        None => {
            // No access token found
            create_error_response(
                StatusCode::UNAUTHORIZED,
                "Access token required",
                codes::INVALID_ACCESS_TOKEN.as_str(),
            )
        }
    }
}
