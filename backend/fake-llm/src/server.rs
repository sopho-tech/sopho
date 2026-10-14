use crate::constants::{
    ANTHROPIC_MESSAGES_PATH, INJECTED_SERVER_ERROR_MESSAGE, MALFORMED_JSON_BODY, OPENAI_BASE_PATH,
    OPENAI_CHAT_COMPLETIONS_PATH, UNMATCHED_ERROR_TYPE,
};
use crate::failure_kind::FailureKind;
use crate::fixture::{Fixture, load_fixtures};
use crate::provider::Provider;
use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use serde_json::json;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
pub struct ReceivedRequest {
    pub provider: Provider,
    pub fixture_name: Option<String>,
    pub body: String,
}

struct SharedState {
    fixtures: Vec<Fixture>,
    received_requests: Mutex<Vec<ReceivedRequest>>,
    pending_failures: Mutex<HashMap<String, FailureKind>>,
}

pub struct FakeLlmServer {
    base_url: String,
    shared_state: Arc<SharedState>,
    server_task: JoinHandle<()>,
}

impl FakeLlmServer {
    pub async fn start(fixture_directory: &Path) -> anyhow::Result<Self> {
        Self::start_on(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            fixture_directory,
        )
        .await
    }

    pub async fn start_on(address: SocketAddr, fixture_directory: &Path) -> anyhow::Result<Self> {
        let shared_state = Arc::new(SharedState {
            fixtures: load_fixtures(fixture_directory)?,
            received_requests: Mutex::new(Vec::new()),
            pending_failures: Mutex::new(HashMap::new()),
        });
        let listener = TcpListener::bind(address).await?;
        let local_address = listener.local_addr()?;
        let router = Router::new()
            .route(ANTHROPIC_MESSAGES_PATH, post(handle_anthropic_request))
            .route(OPENAI_CHAT_COMPLETIONS_PATH, post(handle_openai_request))
            .with_state(Arc::clone(&shared_state));
        let server_task = tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("fake LLM server stopped unexpectedly");
        });
        Ok(Self {
            base_url: format!("http://{local_address}"),
            shared_state,
            server_task,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn anthropic_base_url(&self) -> String {
        self.base_url.clone()
    }

    pub fn openai_base_url(&self) -> String {
        format!("{}{OPENAI_BASE_PATH}", self.base_url)
    }

    pub fn fail_next_request(&self, fixture_name: &str, failure_kind: FailureKind) {
        assert!(
            self.shared_state
                .fixtures
                .iter()
                .any(|fixture| fixture.name == fixture_name),
            "no fixture named {fixture_name}"
        );
        self.shared_state
            .pending_failures
            .lock()
            .expect("pending failures lock poisoned")
            .insert(fixture_name.to_string(), failure_kind);
    }

    pub fn received_requests(&self) -> Vec<ReceivedRequest> {
        self.shared_state
            .received_requests
            .lock()
            .expect("received requests lock poisoned")
            .clone()
    }

    pub fn unmatched_requests(&self) -> Vec<ReceivedRequest> {
        self.received_requests()
            .into_iter()
            .filter(|received_request| received_request.fixture_name.is_none())
            .collect()
    }
}

impl Drop for FakeLlmServer {
    fn drop(&mut self) {
        self.server_task.abort();
    }
}

async fn handle_anthropic_request(
    State(shared_state): State<Arc<SharedState>>,
    body: String,
) -> Response {
    respond(Provider::Anthropic, &shared_state, body)
}

async fn handle_openai_request(
    State(shared_state): State<Arc<SharedState>>,
    body: String,
) -> Response {
    respond(Provider::Openai, &shared_state, body)
}

fn respond(provider: Provider, shared_state: &SharedState, body: String) -> Response {
    let matching_fixtures: Vec<&Fixture> = shared_state
        .fixtures
        .iter()
        .filter(|fixture| fixture.provider == provider && body.contains(&fixture.match_text))
        .collect();
    let matched_fixture = match matching_fixtures.as_slice() {
        [fixture] => Some(*fixture),
        _ => None,
    };
    shared_state
        .received_requests
        .lock()
        .expect("received requests lock poisoned")
        .push(ReceivedRequest {
            provider,
            fixture_name: matched_fixture.map(|fixture| fixture.name.clone()),
            body,
        });
    let Some(fixture) = matched_fixture else {
        let fixture_names: Vec<&str> = matching_fixtures
            .iter()
            .map(|fixture| fixture.name.as_str())
            .collect();
        let message = if fixture_names.is_empty() {
            format!("no {provider} fixture matched the request")
        } else {
            format!(
                "{} {provider} fixtures matched the request: {}",
                fixture_names.len(),
                fixture_names.join(", ")
            )
        };
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({ "error": { "type": UNMATCHED_ERROR_TYPE, "message": message } })),
        )
            .into_response();
    };
    let pending_failure = shared_state
        .pending_failures
        .lock()
        .expect("pending failures lock poisoned")
        .remove(&fixture.name);
    match pending_failure {
        Some(FailureKind::ServerError) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(server_error_body(provider)),
        )
            .into_response(),
        Some(FailureKind::MalformedJson) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            MALFORMED_JSON_BODY,
        )
            .into_response(),
        None => axum::Json(fixture.response.clone()).into_response(),
    }
}

fn server_error_body(provider: Provider) -> serde_json::Value {
    match provider {
        Provider::Anthropic => json!({
            "type": "error",
            "error": { "type": "api_error", "message": INJECTED_SERVER_ERROR_MESSAGE },
        }),
        Provider::Openai => json!({
            "error": { "type": "server_error", "message": INJECTED_SERVER_ERROR_MESSAGE },
        }),
    }
}
