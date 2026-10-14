use crate::support::TestApp;
use crate::support::constants::{
    DEMO_ORDER_COUNT, ORDER_COUNT_QUERY, POSTGRES_PASSWORD, POSTGRES_USER, SAMPLE_DATABASE_SCHEMA,
};
use crate::support::postgres_server::shared_postgres_server;
use reqwest::StatusCode;
use sea_orm::ConnectionTrait;
use sea_orm::sea_query::{Alias, Expr, Query};
use serde_json::{Value, json};
use uuid::Uuid;

fn connection_path(connection: &Value) -> String {
    format!(
        "/connection/{}",
        connection["id"].as_str().expect("connection id")
    )
}

async fn assert_connection_round_trip(app: &TestApp, connection: Value) {
    let path = connection_path(&connection);

    let fetched = app.get(&path).await;
    assert_eq!(fetched.status, StatusCode::OK);
    assert_eq!(fetched.body["id"], connection["id"]);
    assert_eq!(fetched.body["source_type"], connection["source_type"]);

    let listed = app.get("/connection").await;
    assert_eq!(listed.status, StatusCode::OK);
    assert!(
        listed
            .body
            .as_array()
            .expect("connections list")
            .iter()
            .any(|listed_connection| listed_connection["id"] == connection["id"]),
        "the new connection is not listed: {}",
        listed.body
    );

    let mut renamed_connection = fetched.body.clone();
    renamed_connection["name"] = json!("Renamed connection");
    let updated = app.put(&path, renamed_connection).await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.body);
    assert_eq!(app.get(&path).await.body["name"], "Renamed connection");
}

async fn stored_connection_password(app: &TestApp, connection: &Value) -> String {
    let identifier: Uuid = connection["id"]
        .as_str()
        .expect("connection id")
        .parse()
        .expect("connection id is a UUID");
    let query = Query::select()
        .column(Alias::new("password"))
        .from(Alias::new("connection"))
        .and_where(Expr::col(Alias::new("id")).eq(identifier))
        .to_owned();
    let backend = app.app_state.database_connection.get_database_backend();
    app.app_state
        .database_connection
        .query_one(backend.build(&query))
        .await
        .expect("read the stored connection")
        .expect("the connection row exists")
        .try_get::<String>("", "password")
        .expect("read the stored password")
}

#[tokio::test]
async fn sqlite_connection_can_be_created_read_listed_and_updated() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;

    let connection = app.create_sqlite_demo_connection().await;

    assert_eq!(connection["source_type"], "SQLITE");
    assert_eq!(connection["status"], "ACTIVE");
    assert_connection_round_trip(&app, connection).await;
}

#[tokio::test]
async fn postgres_connection_can_be_created_read_listed_and_updated() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;

    let connection = app.create_postgres_sample_connection().await;

    assert_eq!(connection["source_type"], "POSTGRESQL");
    assert_eq!(connection["status"], "ACTIVE");
    assert_connection_round_trip(&app, connection).await;
}

#[tokio::test]
async fn connection_credentials_are_encrypted_at_rest() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;

    let connection = app.create_postgres_sample_connection().await;

    let stored_password = stored_connection_password(&app, &connection).await;
    assert!(!stored_password.is_empty());
    assert_ne!(stored_password, POSTGRES_PASSWORD);
    assert!(!stored_password.contains(POSTGRES_PASSWORD));
}

#[tokio::test]
async fn sqlite_query_returns_columns_and_rows() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;

    let result = app
        .post(
            &format!("{}/execute-query", connection_path(&connection)),
            json!({ "query": ORDER_COUNT_QUERY }),
        )
        .await;

    assert_eq!(result.status, StatusCode::OK, "{}", result.body);
    assert_eq!(result.body["columns"][0]["column_name"], "order_count");
    assert_eq!(
        result.body["data"],
        json!([{ "order_count": DEMO_ORDER_COUNT }])
    );
}

#[tokio::test]
async fn postgres_query_returns_columns_and_rows() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_postgres_sample_connection().await;

    let result = app
        .post(
            &format!("{}/execute-query", connection_path(&connection)),
            json!({ "query": "SELECT name FROM customers ORDER BY id" }),
        )
        .await;

    assert_eq!(result.status, StatusCode::OK, "{}", result.body);
    assert_eq!(result.body["columns"][0]["column_name"], "name");
    assert_eq!(
        result.body["data"],
        json!([
            { "name": "Ada Lovelace" },
            { "name": "Grace Hopper" },
            { "name": "Alan Turing" }
        ])
    );
}

#[tokio::test]
async fn invalid_sql_returns_a_clear_error() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;

    let result = app
        .post(
            &format!("{}/execute-query", connection_path(&connection)),
            json!({ "query": "SELECT * FROM table_that_does_not_exist" }),
        )
        .await;

    assert_eq!(result.status, StatusCode::BAD_REQUEST);
    assert_eq!(result.body["code"], "SYNTAX_ERROR");
    assert!(
        result.body["message"]
            .as_str()
            .expect("error message")
            .contains("table_that_does_not_exist")
    );
}

#[tokio::test]
async fn wrong_postgres_credentials_return_a_clear_error() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let server = shared_postgres_server().await;

    let created = app
        .post(
            "/connection",
            json!({
                "name": "Postgres with a wrong password",
                "source_type": "POSTGRESQL",
                "host": server.host,
                "port": server.port,
                "username": POSTGRES_USER,
                "password": "not-the-postgres-password",
                "database": "postgres",
                "schema": SAMPLE_DATABASE_SCHEMA,
            }),
        )
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.body);
    assert_eq!(created.body["status"], "FAILED");

    let result = app
        .post(
            &format!("{}/execute-query", connection_path(&created.body)),
            json!({ "query": "SELECT 1" }),
        )
        .await;

    assert!(result.status.is_client_error() || result.status.is_server_error());
    assert!(
        result
            .body
            .to_string()
            .contains("password authentication failed"),
        "{}",
        result.body
    );
}
