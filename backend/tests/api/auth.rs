use crate::support::TestApp;
use crate::support::constants::{
    ACCESS_TOKEN_COOKIE, ADMIN_EMAIL, ADMIN_FULL_NAME, ADMIN_PASSWORD, ADMIN_USERNAME,
    REFRESH_TOKEN_COOKIE,
};
use reqwest::StatusCode;
use reqwest::header::SET_COOKIE;
use sea_orm::{ConnectionTrait, Statement};
use serde_json::json;
use uuid::Uuid;

fn cookie_names(response: &reqwest::Response) -> Vec<String> {
    response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .map(|header_value| {
            header_value
                .to_str()
                .expect("Set-Cookie is ASCII")
                .split('=')
                .next()
                .expect("Set-Cookie has a name")
                .to_string()
        })
        .collect()
}

#[tokio::test]
async fn login_sets_the_session_cookies() {
    let app = TestApp::spawn().await;

    let response = app
        .client
        .post(app.api_url("/auth/session"))
        .json(&json!({ "email": ADMIN_EMAIL, "password": ADMIN_PASSWORD }))
        .send()
        .await
        .expect("send the login request");

    assert_eq!(response.status(), StatusCode::OK);
    let cookie_names = cookie_names(&response);
    assert!(
        cookie_names.contains(&ACCESS_TOKEN_COOKIE.to_string()),
        "{cookie_names:?}"
    );
    assert!(
        cookie_names.contains(&REFRESH_TOKEN_COOKIE.to_string()),
        "{cookie_names:?}"
    );
}

#[tokio::test]
async fn login_with_a_wrong_password_is_rejected() {
    let app = TestApp::spawn().await;

    let response = app
        .post(
            "/auth/session",
            json!({ "email": ADMIN_EMAIL, "password": "not-the-admin-password" }),
        )
        .await;

    assert_eq!(
        response.status,
        StatusCode::UNAUTHORIZED,
        "{}",
        response.body
    );
    assert_eq!(app.get("/auth/me").await.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn current_user_requires_a_session() {
    let app = TestApp::spawn().await;
    assert_eq!(app.get("/auth/me").await.status, StatusCode::UNAUTHORIZED);

    app.login_as_admin().await;
    let current_user = app.get("/auth/me").await;

    assert_eq!(current_user.status, StatusCode::OK);
    assert_eq!(current_user.body["email"], ADMIN_EMAIL);
    assert_eq!(current_user.body["username"], ADMIN_USERNAME);
    assert_eq!(current_user.body["full_name"], ADMIN_FULL_NAME);
}

#[tokio::test]
async fn token_refresh_rotates_the_session() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    let response = app
        .client
        .post(app.api_url("/auth/refresh"))
        .send()
        .await
        .expect("send the refresh request");

    assert_eq!(response.status(), StatusCode::OK);
    let cookie_names = cookie_names(&response);
    assert!(
        cookie_names.contains(&ACCESS_TOKEN_COOKIE.to_string()),
        "{cookie_names:?}"
    );
    assert_eq!(app.get("/auth/me").await.status, StatusCode::OK);
}

#[tokio::test]
async fn token_refresh_without_a_session_is_rejected() {
    let app = TestApp::spawn().await;

    let response = app.post("/auth/refresh", json!({})).await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_ends_the_session() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    let response = app.delete("/auth/session").await;

    assert_eq!(response.status, StatusCode::OK, "{}", response.body);
    assert_eq!(app.get("/auth/me").await.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn every_protected_route_group_requires_a_session() {
    let app = TestApp::spawn().await;
    let unknown_identifier = Uuid::new_v4();
    let protected_paths = [
        format!("/dashboard/{unknown_identifier}"),
        "/connection".to_string(),
        "/notebook".to_string(),
        format!("/cell/{unknown_identifier}"),
        "/canvas".to_string(),
        "/search?query=orders".to_string(),
        "/conversational_analytics".to_string(),
        "/ai_configuration".to_string(),
        format!("/suggested_question/connection/{unknown_identifier}"),
        format!("/ai_summary/dashboard/{unknown_identifier}"),
    ];

    for protected_path in protected_paths {
        let response = app.get(&protected_path).await;
        assert_eq!(
            response.status,
            StatusCode::UNAUTHORIZED,
            "{protected_path} answered {} without a session",
            response.status
        );
    }
}

#[tokio::test]
async fn admin_seeding_is_idempotent_across_restarts() {
    let app = TestApp::spawn().await;

    app.restart_initialization().await;
    app.restart_initialization().await;

    let backend = app.app_state.database_connection.get_database_backend();
    let user_count = app
        .app_state
        .database_connection
        .query_one(Statement::from_string(
            backend,
            "SELECT COUNT(*) AS user_count FROM \"user\"",
        ))
        .await
        .expect("count users")
        .expect("the count query returns a row")
        .try_get::<i64>("", "user_count")
        .expect("read the user count");
    assert_eq!(user_count, 1);
    app.login_as_admin().await;
}
