use crate::support::canvas_builder::{create_canvas, create_sql_cell, string_field};
use crate::support::constants::{
    CONNECTION_HOST_NOT_ALLOWED_CODE, CONNECTION_HOST_NOT_ALLOWED_MESSAGE,
    CONNECTION_TYPE_DISABLED_CODE, INTERNAL_HOSTS, OFFERED_SOURCE_TYPE_PATH, ORDER_COUNT_QUERY,
    ORDER_STATUS_QUESTION, SQLITE_DISABLED_MESSAGE, UNRESOLVABLE_HOST,
};
use crate::support::conversation::start_conversation;
use crate::support::postgres_server::shared_postgres_server;
use crate::support::test_app::{JsonResponse, RedeployedApp};
use crate::support::{DeploymentFlags, TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::{Value, json};

fn sqlite_disabled() -> DeploymentFlags {
    DeploymentFlags {
        disabled_connection_types: Some("SQLITE".to_string()),
        is_default_tenant_off: true,
        ..DeploymentFlags::default()
    }
}

fn internal_address_connections_off() -> DeploymentFlags {
    DeploymentFlags {
        is_internal_address_connections_off: true,
        ..DeploymentFlags::default()
    }
}

fn connection_path(connection: &Value) -> String {
    format!("/connection/{}", string_field(connection, "id"))
}

fn postgres_connection_payload(host: &str) -> Value {
    json!({
        "name": "Warehouse",
        "source_type": "POSTGRESQL",
        "host": host,
        "port": 5432,
        "username": "analyst",
        "password": "secret",
        "database": "sales",
    })
}

fn assert_refused(response: &JsonResponse, error_code: &str, message: &str) {
    assert_eq!(response.status, StatusCode::FORBIDDEN, "{}", response.body);
    assert_eq!(response.body["error_code"], error_code, "{}", response.body);
    assert_eq!(response.body["error"], message, "{}", response.body);
}

async fn offered_source_types(session: &UserSession) -> Value {
    let offered = session.get(OFFERED_SOURCE_TYPE_PATH).await;
    assert_eq!(offered.status, StatusCode::OK, "{}", offered.body);
    offered.body
}

async fn listed_connection(session: &UserSession, connection: &Value) -> Value {
    let listed = session.get("/connection").await;
    assert_eq!(listed.status, StatusCode::OK, "{}", listed.body);
    listed
        .body
        .as_array()
        .expect("connections list")
        .iter()
        .find(|listed_connection| listed_connection["id"] == connection["id"])
        .unwrap_or_else(|| panic!("the connection is not listed: {}", listed.body))
        .clone()
}

async fn run_order_count_query(session: &UserSession, connection: &Value) -> JsonResponse {
    session
        .post(
            &format!("{}/execute-query", connection_path(connection)),
            json!({ "query": ORDER_COUNT_QUERY }),
        )
        .await
}

async fn redeploy(app: &TestApp, deployment_flags: DeploymentFlags) -> RedeployedApp {
    app.redeploy_with_deployment_flags(deployment_flags).await
}

#[tokio::test]
async fn every_driver_backed_type_is_offered_by_default() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    assert_eq!(
        offered_source_types(&app).await,
        json!([
            { "source_type": "POSTGRESQL", "display_name": "PostgreSQL" },
            { "source_type": "SUPABASE", "display_name": "Supabase" },
            { "source_type": "SQLITE", "display_name": "SQLite" },
        ])
    );
}

#[tokio::test]
async fn offered_source_types_require_a_session() {
    let app = TestApp::spawn().await;

    let offered = app.get(OFFERED_SOURCE_TYPE_PATH).await;

    assert_eq!(offered.status, StatusCode::UNAUTHORIZED, "{}", offered.body);
}

#[tokio::test]
async fn a_disabled_connection_type_is_not_offered_and_cannot_be_created() {
    let app = TestApp::spawn().await;
    let redeployed = redeploy(&app, sqlite_disabled()).await;

    assert_eq!(
        offered_source_types(&redeployed.admin).await,
        json!([
            { "source_type": "POSTGRESQL", "display_name": "PostgreSQL" },
            { "source_type": "SUPABASE", "display_name": "Supabase" },
        ])
    );
    let created = redeployed
        .admin
        .post(
            "/connection",
            json!({
                "name": "Local file",
                "source_type": "SQLITE",
                "database": "/tmp/any.sqlite",
                "port": null,
            }),
        )
        .await;
    assert_refused(
        &created,
        CONNECTION_TYPE_DISABLED_CODE,
        SQLITE_DISABLED_MESSAGE,
    );
}

#[tokio::test]
async fn an_enabled_type_still_works_while_another_is_disabled() {
    let app = TestApp::spawn().await;
    let redeployed = redeploy(&app, sqlite_disabled()).await;
    let server = shared_postgres_server().await;

    let created = redeployed
        .admin
        .post(
            "/connection",
            json!({
                "name": "Warehouse",
                "source_type": "POSTGRESQL",
                "host": server.host,
                "port": server.port,
                "username": server.username,
                "password": server.password,
                "database": "postgres",
            }),
        )
        .await;

    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    assert_eq!(created.body["status"], "ACTIVE", "{}", created.body);
    assert_eq!(created.body["is_source_type_available"], true);
}

#[tokio::test]
async fn an_existing_connection_of_a_disabled_type_stops_working() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    let canvas = create_canvas(&app, "Orders").await;
    let cell = create_sql_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Order count",
        ORDER_COUNT_QUERY,
    )
    .await;
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    assert_eq!(
        listed_connection(&app, &connection).await["is_source_type_available"],
        true
    );
    assert_eq!(
        run_order_count_query(&app, &connection).await.status,
        StatusCode::OK
    );

    let redeployed = redeploy(&app, sqlite_disabled()).await;
    let session = &redeployed.admin;

    assert_eq!(
        listed_connection(session, &connection).await["is_source_type_available"],
        false
    );
    let fetched = session.get(&connection_path(&connection)).await;
    assert_eq!(fetched.status, StatusCode::OK, "{}", fetched.body);
    assert_eq!(fetched.body["is_source_type_available"], false);

    let updated = session
        .put(&connection_path(&connection), fetched.body.clone())
        .await;
    assert_refused(
        &updated,
        CONNECTION_TYPE_DISABLED_CODE,
        SQLITE_DISABLED_MESSAGE,
    );
    assert_refused(
        &run_order_count_query(session, &connection).await,
        CONNECTION_TYPE_DISABLED_CODE,
        SQLITE_DISABLED_MESSAGE,
    );
    let executed_cell = session
        .post(
            &format!("/cell/{}/execute", string_field(&cell, "id")),
            json!({}),
        )
        .await;
    assert_refused(
        &executed_cell,
        CONNECTION_TYPE_DISABLED_CODE,
        SQLITE_DISABLED_MESSAGE,
    );

    let llm_requests_before_completion = app.fake_llm.received_requests().len();
    let completion = session
        .post(
            &format!("/conversational_analytics/{conversation_id}/completion"),
            json!({}),
        )
        .await;
    assert_eq!(
        completion.status,
        StatusCode::FORBIDDEN,
        "{}",
        completion.body
    );
    assert_eq!(completion.body["code"], CONNECTION_TYPE_DISABLED_CODE);
    assert_eq!(completion.body["error"], SQLITE_DISABLED_MESSAGE);
    assert_eq!(
        app.fake_llm.received_requests().len(),
        llm_requests_before_completion,
        "a refused completion must not call the LLM"
    );
}

#[tokio::test]
async fn internal_hosts_are_refused_when_internal_address_connections_are_off() {
    let app = TestApp::spawn_with_deployment_flags(internal_address_connections_off()).await;
    app.login_as_admin().await;

    for internal_host in INTERNAL_HOSTS.into_iter().chain([UNRESOLVABLE_HOST]) {
        let created = app
            .post("/connection", postgres_connection_payload(internal_host))
            .await;
        assert_eq!(
            created.status,
            StatusCode::FORBIDDEN,
            "{internal_host} was not refused: {}",
            created.body
        );
        assert_eq!(
            created.body["error_code"], CONNECTION_HOST_NOT_ALLOWED_CODE,
            "{internal_host}: {}",
            created.body
        );
        assert_eq!(
            created.body["error"], CONNECTION_HOST_NOT_ALLOWED_MESSAGE,
            "{internal_host}: {}",
            created.body
        );
    }
    let listed = app.get("/connection").await;
    assert!(
        listed
            .body
            .as_array()
            .expect("connections list")
            .iter()
            .all(|connection| connection["source_type"] != "POSTGRESQL"),
        "a refused connection was saved: {}",
        listed.body
    );
}

#[tokio::test]
async fn a_supabase_connection_to_an_internal_host_is_refused_too() {
    let app = TestApp::spawn_with_deployment_flags(internal_address_connections_off()).await;
    app.login_as_admin().await;
    let mut payload = postgres_connection_payload("127.0.0.1");
    payload["source_type"] = json!("SUPABASE");

    let created = app.post("/connection", payload).await;

    assert_refused(
        &created,
        CONNECTION_HOST_NOT_ALLOWED_CODE,
        CONNECTION_HOST_NOT_ALLOWED_MESSAGE,
    );
}

#[tokio::test]
async fn sqlite_connections_are_unaffected_by_the_internal_address_check() {
    let mut app = TestApp::spawn_with_deployment_flags(internal_address_connections_off()).await;
    app.login_as_admin().await;

    let connection = app.create_sqlite_demo_connection().await;

    assert_eq!(
        run_order_count_query(&app, &connection).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn an_existing_connection_to_an_internal_host_stops_working() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_postgres_sample_connection().await;
    assert_eq!(connection["status"], "ACTIVE", "{connection}");

    let redeployed = redeploy(&app, internal_address_connections_off()).await;
    let session = &redeployed.admin;

    let fetched = session.get(&connection_path(&connection)).await;
    assert_eq!(fetched.status, StatusCode::OK, "{}", fetched.body);
    assert_eq!(fetched.body["is_source_type_available"], true);
    let updated = session
        .put(&connection_path(&connection), fetched.body.clone())
        .await;
    assert_refused(
        &updated,
        CONNECTION_HOST_NOT_ALLOWED_CODE,
        CONNECTION_HOST_NOT_ALLOWED_MESSAGE,
    );
    let queried = session
        .post(
            &format!("{}/execute-query", connection_path(&connection)),
            json!({ "query": "SELECT 1" }),
        )
        .await;
    assert_refused(
        &queried,
        CONNECTION_HOST_NOT_ALLOWED_CODE,
        CONNECTION_HOST_NOT_ALLOWED_MESSAGE,
    );
}
