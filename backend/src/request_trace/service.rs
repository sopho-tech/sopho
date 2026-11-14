use crate::request_trace::constants::{
    BACKGROUND_JOB_SPAN_NAME, JSON_CONTENT_TYPE_PREFIX, REQUEST_ID_BODY_FIELD, REQUEST_ID_HEADER,
    REQUEST_SPAN_NAME, TENANT_ID_SPAN_FIELD, USER_ID_SPAN_FIELD,
};
use crate::request_trace::dto::RequestId;
use axum::body::Body;
use axum::extract::Request;
use axum::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use tracing::Span;
use tracing::field::Empty;
use uuid::Uuid;

pub async fn assign_request_id(mut request: Request, next: Next) -> Response {
    let request_id = RequestId {
        value: Uuid::new_v4(),
    };
    let request_id_header_name = HeaderName::from_static(REQUEST_ID_HEADER);
    let request_id_header_value = HeaderValue::from_str(&request_id.value.to_string())
        .expect("a UUID is a valid header value");
    request.headers_mut().insert(
        request_id_header_name.clone(),
        request_id_header_value.clone(),
    );
    request.extensions_mut().insert(request_id);

    let response = next.run(request).await;
    let mut response = with_request_id_in_error_body(response, request_id).await;
    response
        .headers_mut()
        .insert(request_id_header_name, request_id_header_value);
    response
}

fn is_json_error(response: &Response) -> bool {
    let is_error = response.status().is_client_error() || response.status().is_server_error();
    let is_json = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|content_type| content_type.to_str().ok())
        .is_some_and(|content_type| content_type.starts_with(JSON_CONTENT_TYPE_PREFIX));
    is_error && is_json
}

async fn with_request_id_in_error_body(response: Response, request_id: RequestId) -> Response {
    if !is_json_error(&response) {
        return response;
    }
    let (mut response_parts, response_body) = response.into_parts();
    let body_bytes = match axum::body::to_bytes(response_body, usize::MAX).await {
        Ok(body_bytes) => body_bytes,
        Err(read_error) => {
            tracing::error!("could not read an error response body: {read_error}");
            response_parts.status = StatusCode::INTERNAL_SERVER_ERROR;
            return Response::from_parts(response_parts, Body::empty());
        }
    };
    let Ok(serde_json::Value::Object(mut error_fields)) =
        serde_json::from_slice::<serde_json::Value>(&body_bytes)
    else {
        return Response::from_parts(response_parts, Body::from(body_bytes));
    };
    error_fields.insert(
        REQUEST_ID_BODY_FIELD.to_string(),
        serde_json::Value::String(request_id.value.to_string()),
    );
    let rewritten_body = serde_json::Value::Object(error_fields).to_string();
    response_parts.headers.remove(CONTENT_LENGTH);
    Response::from_parts(response_parts, Body::from(rewritten_body))
}

pub fn request_span<RequestBody>(request: &axum::http::Request<RequestBody>) -> Span {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|request_id| request_id.value.to_string());
    tracing::error_span!(
        REQUEST_SPAN_NAME,
        request_id = request_id,
        method = %request.method(),
        path = request.uri().path(),
        user_id = Empty,
        tenant_id = Empty,
    )
}

pub fn record_user_id(user_id: Uuid) {
    Span::current().record(USER_ID_SPAN_FIELD, tracing::field::display(user_id));
}

pub fn record_tenant_id(tenant_id: Uuid) {
    Span::current().record(TENANT_ID_SPAN_FIELD, tracing::field::display(tenant_id));
}

pub fn background_job_span(job_name: &'static str) -> Span {
    tracing::error_span!(BACKGROUND_JOB_SPAN_NAME, job = job_name)
}

pub fn background_job_span_for_tenant(job_name: &'static str, tenant_id: Uuid) -> Span {
    tracing::error_span!(BACKGROUND_JOB_SPAN_NAME, job = job_name, tenant_id = %tenant_id)
}

#[cfg(test)]
mod tests {
    use super::assign_request_id;
    use crate::request_trace::constants::{REQUEST_ID_BODY_FIELD, REQUEST_ID_HEADER};
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::middleware;
    use axum::response::IntoResponse;
    use axum::routing::get;
    use tower::ServiceExt;

    async fn json_error() -> impl IntoResponse {
        (
            StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({ "error": "Refused" })),
        )
    }

    async fn plain_text_error() -> impl IntoResponse {
        (StatusCode::BAD_REQUEST, "not json")
    }

    async fn json_array_error() -> impl IntoResponse {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!(["a", "b"])),
        )
    }

    async fn json_success() -> impl IntoResponse {
        axum::Json(serde_json::json!({ "name": "kept as it is" }))
    }

    fn traced_router() -> Router {
        Router::new()
            .route("/json_error", get(json_error))
            .route("/plain_text_error", get(plain_text_error))
            .route("/json_array_error", get(json_array_error))
            .route("/json_success", get(json_success))
            .layer(middleware::from_fn(assign_request_id))
    }

    struct TracedResponse {
        request_id: String,
        body: String,
    }

    async fn send_traced_request(path: &str, client_request_id: Option<&str>) -> TracedResponse {
        let mut request_builder = Request::builder().uri(path);
        if let Some(client_request_id) = client_request_id {
            request_builder = request_builder.header(REQUEST_ID_HEADER, client_request_id);
        }
        let response = traced_router()
            .oneshot(request_builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let request_id = response.headers()[REQUEST_ID_HEADER]
            .to_str()
            .unwrap()
            .to_string();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        TracedResponse {
            request_id,
            body: String::from_utf8(body_bytes.to_vec()).unwrap(),
        }
    }

    #[tokio::test]
    async fn a_json_error_body_gains_the_request_id_of_its_header() {
        let response = send_traced_request("/json_error", None).await;
        let body: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(body["error"], "Refused");
        assert_eq!(body[REQUEST_ID_BODY_FIELD], response.request_id);
        assert!(uuid::Uuid::parse_str(&response.request_id).is_ok());
    }

    #[tokio::test]
    async fn every_request_gets_its_own_request_id() {
        let first = send_traced_request("/json_success", None).await;
        let second = send_traced_request("/json_success", None).await;
        assert_ne!(first.request_id, second.request_id);
    }

    #[tokio::test]
    async fn a_request_id_sent_by_the_client_is_replaced() {
        for client_request_id in [
            "chosen-by-the-client",
            "00000000-0000-4000-8000-000000000000",
        ] {
            let response = send_traced_request("/json_error", Some(client_request_id)).await;
            assert_ne!(response.request_id, client_request_id);
            assert!(!response.body.contains(client_request_id));
        }
    }

    #[tokio::test]
    async fn bodies_that_are_not_json_object_errors_are_left_as_they_are() {
        assert_eq!(
            send_traced_request("/plain_text_error", None).await.body,
            "not json"
        );
        assert_eq!(
            send_traced_request("/json_array_error", None).await.body,
            r#"["a","b"]"#
        );
        assert_eq!(
            send_traced_request("/json_success", None).await.body,
            r#"{"name":"kept as it is"}"#
        );
    }
}
