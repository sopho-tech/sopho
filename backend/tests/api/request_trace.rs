use crate::support::TestApp;
use crate::support::constants::{
    REQUEST_ID_HEADER, SECRET_CALLBACK_CODE, SINGLE_SIGN_ON_PROVIDER, TERMINAL_COLOUR_CODE_END,
    TERMINAL_ESCAPE_CHARACTER,
};
use crate::support::database::select_uuid;
use reqwest::{Response, StatusCode};
use serde_json::Value;
use sopho::LogsFormat;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tracing::subscriber::DefaultGuard;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

#[derive(Clone, Default)]
struct CapturedLogs {
    written_bytes: Arc<Mutex<Vec<u8>>>,
}

impl CapturedLogs {
    fn text(&self) -> String {
        String::from_utf8(self.written_bytes.lock().expect("lock the logs").clone())
            .expect("logs are text")
    }

    fn text_without_terminal_colours(&self) -> String {
        let mut plain_text = String::new();
        let mut is_inside_colour_code = false;
        for character in self.text().chars() {
            match (is_inside_colour_code, character) {
                (false, TERMINAL_ESCAPE_CHARACTER) => is_inside_colour_code = true,
                (false, _) => plain_text.push(character),
                (true, TERMINAL_COLOUR_CODE_END) => is_inside_colour_code = false,
                (true, _) => {}
            }
        }
        plain_text
    }

    fn json_lines(&self) -> Vec<Value> {
        self.text()
            .lines()
            .map(|line| {
                serde_json::from_str(line)
                    .unwrap_or_else(|error| panic!("log line is not JSON ({error}): {line}"))
            })
            .collect()
    }

    fn json_lines_of_request(&self, request_id: &str) -> Vec<Value> {
        self.json_lines()
            .into_iter()
            .filter(|line| line["request_id"] == request_id)
            .collect()
    }
}

impl Write for CapturedLogs {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.written_bytes
            .lock()
            .expect("lock the logs")
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'writer> MakeWriter<'writer> for CapturedLogs {
    type Writer = CapturedLogs;

    fn make_writer(&'writer self) -> Self::Writer {
        self.clone()
    }
}

fn capture_logs(logs_format: LogsFormat) -> (CapturedLogs, DefaultGuard) {
    let captured_logs = CapturedLogs::default();
    let subscriber =
        tracing_subscriber::registry().with(sopho::log_layer(logs_format, captured_logs.clone()));
    (captured_logs, tracing::subscriber::set_default(subscriber))
}

fn request_id_of(response: &Response) -> String {
    response.headers()[REQUEST_ID_HEADER]
        .to_str()
        .expect("the request id is text")
        .to_string()
}

async fn admin_user_id(app: &TestApp) -> String {
    select_uuid(
        app,
        "user",
        "id",
        &[("email", crate::support::constants::ADMIN_EMAIL.into())],
    )
    .await
    .to_string()
}

#[tokio::test]
async fn every_response_carries_its_own_request_id() {
    let app = TestApp::spawn().await;
    let health_url = format!("{}/api/v1/monitor/health", app.base_url());

    let first = app.client.get(&health_url).send().await.expect("first");
    let second = app
        .client
        .get(&health_url)
        .header(REQUEST_ID_HEADER, "chosen-by-the-client")
        .send()
        .await
        .expect("second");
    let page = app
        .client
        .get(format!("{}/signin", app.base_url()))
        .send()
        .await
        .expect("page");

    let request_ids = [
        request_id_of(&first),
        request_id_of(&second),
        request_id_of(&page),
    ];
    for request_id in &request_ids {
        assert!(uuid::Uuid::parse_str(request_id).is_ok(), "{request_id}");
    }
    assert_ne!(request_ids[0], request_ids[1]);
    assert_ne!(request_ids[1], request_ids[2]);
    assert_ne!(request_ids[1], "chosen-by-the-client");
}

#[tokio::test]
async fn an_error_response_body_carries_the_request_id_of_its_header() {
    let app = TestApp::spawn().await;

    let signed_out = app
        .client
        .get(format!("{}/api/v1/auth/me", app.base_url()))
        .send()
        .await
        .expect("request the profile signed out");
    assert_eq!(signed_out.status(), StatusCode::UNAUTHORIZED);
    let signed_out_request_id = request_id_of(&signed_out);
    let signed_out_body: Value = signed_out.json().await.expect("the refusal is JSON");
    assert_eq!(signed_out_body["request_id"], signed_out_request_id);
    assert!(signed_out_body["message"].is_string(), "{signed_out_body}");

    app.login_as_admin().await;
    let refused_connection = app
        .client
        .post(app.api_url("/connection"))
        .json(&serde_json::json!({ "name": "No type" }))
        .send()
        .await
        .expect("post an invalid connection");
    assert!(refused_connection.status().is_client_error());
    let succeeded = app
        .client
        .get(app.api_url("/connection"))
        .send()
        .await
        .expect("list connections");
    assert_eq!(succeeded.status(), StatusCode::OK);
    let connections: Value = succeeded.json().await.expect("connections are JSON");
    assert!(connections.is_array(), "a successful body is left as it is");
}

#[tokio::test]
async fn json_log_lines_of_a_tenant_request_carry_request_user_and_tenant_ids() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let expected_user_id = admin_user_id(&app).await;
    let (captured_logs, _capture_guard) = capture_logs(LogsFormat::Json);

    let tenant_request = app
        .client
        .get(app.api_url("/connection"))
        .send()
        .await
        .expect("list connections");
    assert_eq!(tenant_request.status(), StatusCode::OK);
    let tenant_request_id = request_id_of(&tenant_request);
    let signed_out_request = app
        .session(None)
        .client
        .get(format!(
            "{}/api/v1/auth/sign_in_page_option",
            app.base_url()
        ))
        .send()
        .await
        .expect("read the sign-in options");
    let signed_out_request_id = request_id_of(&signed_out_request);

    let tenant_request_lines = captured_logs.json_lines_of_request(&tenant_request_id);
    assert!(
        !tenant_request_lines.is_empty(),
        "no log line carries the request id: {}",
        captured_logs.text()
    );
    let finished_line = tenant_request_lines
        .last()
        .expect("the request logged at least one line");
    assert_eq!(finished_line["user_id"], expected_user_id);
    assert_eq!(finished_line["tenant_id"], app.tenant_id.to_string());
    assert_eq!(finished_line["method"], "GET");
    assert_eq!(
        finished_line["path"],
        format!("/api/v1/tenant/{}/connection", app.tenant_id)
    );

    let signed_out_lines = captured_logs.json_lines_of_request(&signed_out_request_id);
    assert!(!signed_out_lines.is_empty());
    for signed_out_line in &signed_out_lines {
        assert_eq!(signed_out_line.get("user_id"), None);
        assert_eq!(signed_out_line.get("tenant_id"), None);
        assert_eq!(signed_out_line.get("span"), None);
        assert_eq!(signed_out_line.get("spans"), None);
    }
}

#[tokio::test]
async fn plain_log_lines_carry_the_ids_too() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let expected_user_id = admin_user_id(&app).await;
    let (captured_logs, _capture_guard) = capture_logs(LogsFormat::Text);

    let tenant_request = app
        .client
        .get(app.api_url("/connection"))
        .send()
        .await
        .expect("list connections");
    let tenant_request_id = request_id_of(&tenant_request);

    let logs = captured_logs.text_without_terminal_colours();
    assert!(
        logs.lines().any(|line| {
            line.contains(&format!("request_id=\"{tenant_request_id}\""))
                && line.contains(&format!("user_id={expected_user_id}"))
                && line.contains(&format!("tenant_id={}", app.tenant_id))
        }),
        "no plain log line carries all three ids: {logs}"
    );
    assert!(serde_json::from_str::<Value>(logs.lines().next().unwrap()).is_err());
}

#[tokio::test]
async fn the_query_string_of_a_sign_in_callback_is_not_logged() {
    let app = TestApp::spawn_with_single_sign_on().await;
    let (captured_logs, _capture_guard) = capture_logs(LogsFormat::Json);

    let callback = app
        .client_without_redirects()
        .get(format!(
            "{}/api/v1/auth/oidc/{SINGLE_SIGN_ON_PROVIDER}/callback?code={SECRET_CALLBACK_CODE}&state=any",
            app.base_url()
        ))
        .send()
        .await
        .expect("request the callback");
    let callback_request_id = request_id_of(&callback);

    let callback_lines = captured_logs.json_lines_of_request(&callback_request_id);
    assert!(!callback_lines.is_empty());
    assert_eq!(
        callback_lines[0]["path"],
        format!("/api/v1/auth/oidc/{SINGLE_SIGN_ON_PROVIDER}/callback")
    );
    assert!(
        !captured_logs.text().contains(SECRET_CALLBACK_CODE),
        "the callback code was logged"
    );
}
