use crate::authentication::service::{AccessTokenValidationError, authenticate_from_cookies};
use crate::common::AppState;
use crate::common::error_codes::codes;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{Response, StatusCode};
use axum::middleware::Next;
use serde_json::json;

pub async fn session_middleware_fn(
    axum::extract::State(app_state): axum::extract::State<AppState>,
    cookies: tower_cookies::Cookies,
    request: Request,
    next: Next,
) -> Response<Body> {
    match authenticate_from_cookies(&cookies, &app_state).await {
        Ok(authenticated_user) => {
            let mut request = request;
            request.extensions_mut().insert(authenticated_user);
            next.run(request).await
        }
        Err(error) => access_token_error_response(error),
    }
}

fn access_token_error_response(error: AccessTokenValidationError) -> Response<Body> {
    let (message, error_code) = match error {
        AccessTokenValidationError::Missing => (
            "Access token required",
            codes::INVALID_ACCESS_TOKEN.as_str(),
        ),
        AccessTokenValidationError::Invalid => {
            ("Invalid access token", codes::INVALID_ACCESS_TOKEN.as_str())
        }
        AccessTokenValidationError::Expired => {
            ("Access token expired", codes::ACCESS_TOKEN_EXPIRED.as_str())
        }
    };

    let body = json!({
        "message": message,
        "error_code": error_code
    });

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap_or_default()))
        .unwrap_or_else(|_| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap()
        })
}
