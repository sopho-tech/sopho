use crate::support::TestApp;
use crate::support::canvas_builder::{
    column_names, create_canvas, create_chart_cell, create_sql_cell, order_status_chart_content,
    place_chart_on_dashboard, string_field,
};
use crate::support::constants::{AI_PROVIDER_API_KEY, ORDER_STATUS_QUERY, ORDER_STATUS_QUESTION};
use crate::support::conversation::{
    append_canvas_command, event_data, event_names, last_message, run_completion,
    start_conversation, wait_for_persisted_event,
};
use fake_llm::{FailureKind, Provider};
use reqwest::StatusCode;
use sea_orm::ConnectionTrait;
use sea_orm::sea_query::{Alias, Query};
use serde_json::json;
use std::collections::BTreeSet;

async fn stored_api_key(app: &TestApp) -> String {
    let query = Query::select()
        .column(Alias::new("api_key_encrypted"))
        .from(Alias::new("ai_configuration"))
        .to_owned();
    let backend = app.app_state.database_connection.get_database_backend();
    app.app_state
        .database_connection
        .query_one(backend.build(&query))
        .await
        .expect("read the stored AI configuration")
        .expect("the AI configuration row exists")
        .try_get::<String>("", "api_key_encrypted")
        .expect("read the stored API key")
}

fn requests_answered_by(app: &TestApp, fixture_name: &str) -> usize {
    app.fake_llm
        .received_requests()
        .iter()
        .filter(|request| request.fixture_name.as_deref() == Some(fixture_name))
        .count()
}

async fn answer_order_status_question(app: &TestApp, connection_id: &str) -> String {
    let conversation_id = start_conversation(app, connection_id, ORDER_STATUS_QUESTION).await;
    let events = run_completion(app, &conversation_id).await;
    assert_eq!(
        event_names(&events).last().map(String::as_str),
        Some("completed"),
        "{:?}",
        event_names(&events)
    );
    conversation_id
}

async fn assert_ai_configuration_lifecycle(provider_name: &str, provider: Provider) {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let not_configured = json!({ "status": "not_configured" });
    assert_eq!(app.get("/ai_configuration").await.body, not_configured);

    let saved = app.configure_ai_provider(provider_name).await;

    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);
    assert_eq!(saved.body["status"], "live");
    assert_eq!(saved.body["provider"], provider_name);
    assert!(saved.body.get("api_key").is_none(), "{}", saved.body);
    assert!(
        app.fake_llm
            .received_requests()
            .iter()
            .any(|request| request.provider == provider
                && request.fixture_name.as_deref() == Some("liveness_check_agent")),
        "the liveness check did not reach the {provider_name} endpoint"
    );
    let stored_key = stored_api_key(&app).await;
    assert!(!stored_key.is_empty());
    assert!(
        !stored_key.contains(AI_PROVIDER_API_KEY),
        "the API key is stored in plain text"
    );

    let tested = app
        .post(
            "/ai_configuration/test",
            json!({ "provider": provider_name, "api_key": AI_PROVIDER_API_KEY }),
        )
        .await;
    assert_eq!(tested.status, StatusCode::OK, "{}", tested.body);
    assert_eq!(tested.body, json!({ "ok": true }));

    let deleted = app.delete("/ai_configuration").await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(app.get("/ai_configuration").await.body, not_configured);
}

#[tokio::test]
async fn anthropic_configuration_is_encrypted_tested_and_deleted() {
    assert_ai_configuration_lifecycle("anthropic", Provider::Anthropic).await;
}

#[tokio::test]
async fn openai_configuration_is_encrypted_tested_and_deleted() {
    assert_ai_configuration_lifecycle("openai", Provider::Openai).await;
}

#[tokio::test]
async fn failed_liveness_check_marks_the_configuration_failed_and_blocks_conversations() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;
    app.fake_llm
        .fail_next_request("liveness_check_agent", FailureKind::ServerError);

    let saved = app.configure_ai_provider("anthropic").await;

    assert_eq!(saved.status, StatusCode::OK, "{}", saved.body);
    assert_eq!(saved.body["status"], "failed");
    let conversation = app
        .post(
            "/conversational_analytics",
            json!({
                "connection_id": connection["id"],
                "segments": [{ "type": "TEXT", "text": ORDER_STATUS_QUESTION }],
            }),
        )
        .await;
    assert_eq!(conversation.status, StatusCode::FORBIDDEN);
    assert_eq!(conversation.body["code"], "AI_NOT_LIVE");
}

#[tokio::test]
async fn conversation_completion_streams_every_stage_and_answers_with_a_chart() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    let connection = app.create_sqlite_demo_connection().await;
    let conversation_id = start_conversation(
        &app,
        &string_field(&connection, "id"),
        ORDER_STATUS_QUESTION,
    )
    .await;

    let events = run_completion(&app, &conversation_id).await;

    let names = event_names(&events);
    assert_eq!(names.len(), 25, "{names:?}");
    assert_eq!(
        names[..19],
        [
            "starting",
            "routing",
            "routed",
            "generating_candidate_hypothesis",
            "generated_candidate_hypothesis",
            "integrating_candidate_plans",
            "integrated_candidate_plans",
            "executing_search_space_reduction",
            "executed_search_space_reduction",
            "executing_functional_role_analysis",
            "executed_functional_role_analysis",
            "executing_data_profiling",
            "executed_data_profiling",
            "executing_schema_linking_synthesis",
            "executed_schema_linking_synthesis",
            "generating_sql",
            "generated_sql",
            "executing_query",
            "executed_query",
        ]
    );
    assert_eq!(
        names[19..23]
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "narrated",
            "narrating",
            "recommended_visualization",
            "recommending_visualization",
        ])
    );
    assert_eq!(names[23..], ["suggested_followups", "completed"]);
    assert_eq!(
        event_data(&events, "generated_sql")["sql"],
        ORDER_STATUS_QUERY
    );
    let query_result = event_data(&events, "executed_query");
    assert_eq!(column_names(&query_result), ["order_status", "order_count"]);
    let rows = query_result["data"].as_array().expect("result rows");
    assert_eq!(rows.len(), 7);
    assert_eq!(
        rows[0],
        json!({ "order_status": "delivered", "order_count": 9620 })
    );
    let visualization = &event_data(&events, "recommended_visualization")["visualization"];
    assert_eq!(visualization["chart_type"], "BAR");
    assert_eq!(visualization["x_axis"], "order_status");
    assert_eq!(visualization["series"][0]["data_key"], "order_count");
    assert_eq!(
        event_data(&events, "narrated")["narration"],
        "Delivered orders far outnumber every other status."
    );
    assert_eq!(
        event_data(&events, "suggested_followups")["questions"]
            .as_array()
            .expect("follow-up questions")
            .len(),
        2
    );
    assert_eq!(
        last_message(&app, &conversation_id).await["status"],
        "PROCESSED"
    );
}

#[tokio::test]
async fn canvas_command_turns_the_answered_question_into_a_canvas_with_a_chart() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    let connection = app.create_sqlite_demo_connection().await;
    let conversation_id =
        answer_order_status_question(&app, &string_field(&connection, "id")).await;
    wait_for_persisted_event(&app, &conversation_id, "completed").await;
    append_canvas_command(&app, &conversation_id).await;

    let events = run_completion(&app, &conversation_id).await;

    assert_eq!(
        event_names(&events),
        [
            "starting",
            "routing",
            "routed",
            "generating_canvas",
            "canvas_generated",
            "completed",
        ]
    );
    assert_eq!(
        event_data(&events, "routed")["decision"]["code"],
        "generate_canvas"
    );
    let generated = event_data(&events, "canvas_generated");
    assert_eq!(generated["name"], "Order statuses");
    assert_eq!(generated["reused"], false);
    assert_eq!(generated["sql_cell_count"], 1);
    assert_eq!(generated["chart_cell_count"], 1);
    assert_eq!(generated["dashboard_charts_count"], 1);
    let canvas_id = string_field(&generated, "canvas_id");
    let notebooks = app.get(&format!("/notebook/canvas/{canvas_id}")).await;
    assert_eq!(notebooks.status, StatusCode::OK, "{}", notebooks.body);
    let notebook_id = string_field(&notebooks.body[0], "id");
    let cells = app.get(&format!("/notebook/{notebook_id}/cells")).await;
    assert_eq!(cells.status, StatusCode::OK, "{}", cells.body);
    let cells = cells.body["cells"]
        .as_array()
        .expect("notebook cells")
        .clone();
    let cell_types: Vec<String> = cells
        .iter()
        .map(|cell| string_field(cell, "cell_type"))
        .collect();
    assert_eq!(cell_types, ["SQL", "CHART"]);
    assert_eq!(cells[0]["content"], ORDER_STATUS_QUERY);
    let dashboard = app.get(&format!("/dashboard/canvas/{canvas_id}")).await;
    assert_eq!(dashboard.status, StatusCode::OK, "{}", dashboard.body);
    assert_eq!(dashboard.body["layout"][0]["cell_id"], cells[1]["id"]);
}

#[tokio::test]
async fn conversation_name_is_suggested_by_the_model() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    let connection = app.create_sqlite_demo_connection().await;
    let conversation_id = start_conversation(
        &app,
        &string_field(&connection, "id"),
        ORDER_STATUS_QUESTION,
    )
    .await;

    let named = app
        .post(
            &format!("/conversational_analytics/{conversation_id}/suggest-name"),
            json!({}),
        )
        .await;

    assert_eq!(named.status, StatusCode::OK, "{}", named.body);
    assert_eq!(named.body["name"], "Orders by status");
    let fetched = app
        .get(&format!("/conversational_analytics/{conversation_id}"))
        .await;
    assert_eq!(fetched.body["conversation"]["name"], "Orders by status");
}

async fn assert_chart_and_dashboard_summaries(provider: &str) {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider(provider).await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    let canvas = create_canvas(&app, "Summaries").await;
    let sql_cell = create_sql_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Orders by status",
        ORDER_STATUS_QUERY,
    )
    .await;
    let chart = create_chart_cell(
        &app,
        &canvas.notebook_id,
        &connection_id,
        "Orders by status chart",
        order_status_chart_content(&string_field(&sql_cell, "id")),
    )
    .await;
    let chart_id = string_field(&chart, "id");
    place_chart_on_dashboard(&app, &canvas, &chart_id).await;

    let chart_summary = app
        .post(&format!("/ai_summary/chart/{chart_id}"), json!({}))
        .await;
    let dashboard_summary = app
        .post(
            &format!(
                "/ai_summary/dashboard/{}",
                string_field(&canvas.dashboard, "id")
            ),
            json!({}),
        )
        .await;

    assert_eq!(
        chart_summary.status,
        StatusCode::OK,
        "{}",
        chart_summary.body
    );
    assert_eq!(
        chart_summary.body["status"], "READY",
        "{}",
        chart_summary.body
    );
    assert_eq!(
        chart_summary.body["summary_text"],
        "Delivered orders dominate the chart."
    );
    assert_eq!(
        dashboard_summary.status,
        StatusCode::OK,
        "{}",
        dashboard_summary.body
    );
    assert_eq!(
        dashboard_summary.body["status"], "READY",
        "{}",
        dashboard_summary.body
    );
    assert_eq!(
        dashboard_summary.body["summary_text"],
        "Most orders on this dashboard were delivered."
    );
    assert_eq!(dashboard_summary.body["summarized_entity_count"], 1);
}

#[tokio::test]
async fn anthropic_writes_chart_and_dashboard_summaries() {
    assert_chart_and_dashboard_summaries("anthropic").await;
}

#[tokio::test]
async fn openai_writes_chart_and_dashboard_summaries() {
    assert_chart_and_dashboard_summaries("openai").await;
}

#[tokio::test]
async fn suggested_questions_are_generated_once_for_every_connection() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    app.create_sqlite_demo_connection().await;

    sopho::refresh_stale_suggested_questions(&app.app_state).await;

    let connections = app.get("/connection").await;
    let connections = connections.body.as_array().expect("connections").clone();
    assert_eq!(connections.len(), 2);
    for connection in &connections {
        let suggested = app
            .get(&format!(
                "/suggested_question/connection/{}",
                string_field(connection, "id")
            ))
            .await;
        assert_eq!(suggested.status, StatusCode::OK, "{}", suggested.body);
        let questions: BTreeSet<String> = suggested
            .body
            .as_array()
            .expect("suggested questions")
            .iter()
            .map(|question| string_field(question, "question_text"))
            .collect();
        assert_eq!(
            questions,
            BTreeSet::from([
                "Which product categories sell the most?".to_string(),
                "Which sellers ship the most orders?".to_string(),
            ])
        );
    }
    assert_eq!(
        requests_answered_by(&app, "suggested_questions_agent"),
        connections.len()
    );

    sopho::refresh_stale_suggested_questions(&app.app_state).await;

    assert_eq!(
        requests_answered_by(&app, "suggested_questions_agent"),
        connections.len()
    );
}

#[tokio::test]
async fn llm_server_error_ends_the_answer_with_an_error_and_the_server_keeps_serving() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    app.fake_llm
        .fail_next_request("router_agent", FailureKind::ServerError);

    let events = run_completion(&app, &conversation_id).await;

    assert_eq!(event_names(&events), ["starting", "routing", "error"]);
    let error = event_data(&events, "error");
    assert!(
        error
            .as_str()
            .expect("error message")
            .contains("fake LLM injected server error"),
        "{error}"
    );
    assert_eq!(
        last_message(&app, &conversation_id).await["status"],
        "FAILED"
    );
    answer_order_status_question(&app, &connection_id).await;
}

#[tokio::test]
async fn malformed_llm_response_fails_the_answer_after_the_query_ran() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    let connection = app.create_sqlite_demo_connection().await;
    let conversation_id = start_conversation(
        &app,
        &string_field(&connection, "id"),
        ORDER_STATUS_QUESTION,
    )
    .await;
    app.fake_llm.fail_next_request(
        "visualization_recommendation_agent",
        FailureKind::MalformedJson,
    );

    let events = run_completion(&app, &conversation_id).await;

    let names = event_names(&events);
    assert!(names.contains(&"executed_query".to_string()), "{names:?}");
    assert!(!names.contains(&"completed".to_string()), "{names:?}");
    assert_eq!(names.last().map(String::as_str), Some("error"), "{names:?}");
    assert_eq!(
        last_message(&app, &conversation_id).await["status"],
        "FAILED"
    );
}
