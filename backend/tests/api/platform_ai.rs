use crate::support::canvas_builder::{
    create_canvas, create_chart_cell, create_sql_cell, order_status_chart_content, string_field,
};
use crate::support::constants::{
    AI_PROVIDER_API_KEY, ORDER_STATUS_QUERY, ORDER_STATUS_QUESTION, PLATFORM_AI_API_KEY,
};
use crate::support::conversation::{
    answer_order_status_question, event_names, run_completion, start_conversation,
};
use crate::support::database::select_rows;
use crate::support::{DeploymentFlags, TestApp};
use fake_llm::FailureKind;
use reqwest::StatusCode;
use serde_json::{Value, json};

const GENEROUS_TOKEN_ALLOWANCE: u64 = 1_000_000;
const TOKEN_ALLOWANCE_BELOW_ONE_ANSWER: u64 = 100;
const ALLOWANCE_USED_CODE: &str = "PLATFORM_AI_ALLOWANCE_USED";
const ALLOWANCE_USED_MESSAGE: &str = "This tenant has used its platform AI allowance. Add your own AI key in Settings to keep using AI.";
const AI_NOT_LIVE_CODE: &str = "AI_NOT_LIVE";
const PLATFORM_KEY_SOURCE: &str = "PLATFORM";
const TENANT_KEY_SOURCE: &str = "TENANT";
const CHART_SUMMARY_AGENT: &str = "chart_summary_agent";

struct UsageTotals {
    call_count: usize,
    platform_tokens: i64,
    tenant_call_count: usize,
}

async fn usage_totals(app: &TestApp) -> UsageTotals {
    let usage_rows = select_rows(
        app,
        "ai_usage",
        &["key_source", "input_tokens", "output_tokens"],
        &[],
    )
    .await;
    let mut usage_totals = UsageTotals {
        call_count: usage_rows.len(),
        platform_tokens: 0,
        tenant_call_count: 0,
    };
    for usage_row in usage_rows {
        let key_source: String = usage_row.try_get("", "key_source").expect("key_source");
        let input_tokens: i64 = usage_row.try_get("", "input_tokens").expect("input_tokens");
        let output_tokens: i64 = usage_row
            .try_get("", "output_tokens")
            .expect("output_tokens");
        match key_source.as_str() {
            PLATFORM_KEY_SOURCE => usage_totals.platform_tokens += input_tokens + output_tokens,
            TENANT_KEY_SOURCE => usage_totals.tenant_call_count += 1,
            other_key_source => panic!("unexpected key source {other_key_source}"),
        }
    }
    usage_totals
}

async fn platform_ai_used_tokens(app: &TestApp) -> i64 {
    let platform_ai_usage_rows = select_rows(
        app,
        "platform_ai_usage",
        &["used_tokens"],
        &[("tenant_id", app.tenant_id.into())],
    )
    .await;
    match platform_ai_usage_rows.as_slice() {
        [] => 0,
        [platform_ai_usage_row] => platform_ai_usage_row
            .try_get("", "used_tokens")
            .expect("used_tokens"),
        _ => panic!("more than one platform AI usage row for the tenant"),
    }
}

async fn app_with_platform_ai(provider: &'static str, token_allowance: u64) -> (TestApp, String) {
    let mut app = TestApp::spawn_with_deployment_flags(DeploymentFlags::with_platform_ai(
        provider,
        token_allowance,
    ))
    .await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    (app, connection_id)
}

async fn ai_status(app: &TestApp) -> Value {
    let status = app.get("/ai_configuration").await;
    assert_eq!(status.status, StatusCode::OK, "{}", status.body);
    status.body
}

async fn create_order_status_chart(app: &TestApp, connection_id: &str) -> String {
    let canvas = create_canvas(app, "Platform AI").await;
    let sql_cell = create_sql_cell(
        app,
        &canvas.notebook_id,
        connection_id,
        "Orders by status",
        ORDER_STATUS_QUERY,
    )
    .await;
    let chart = create_chart_cell(
        app,
        &canvas.notebook_id,
        connection_id,
        "Orders by status chart",
        order_status_chart_content(&string_field(&sql_cell, "id")),
    )
    .await;
    string_field(&chart, "id")
}

fn new_conversation_body(connection_id: &str) -> Value {
    json!({
        "connection_id": connection_id,
        "segments": [{ "type": "TEXT", "text": ORDER_STATUS_QUESTION }],
    })
}

#[tokio::test]
async fn a_tenant_with_no_key_answers_on_the_platform_key_and_its_total_matches_its_usage() {
    let (app, connection_id) = app_with_platform_ai("anthropic", GENEROUS_TOKEN_ALLOWANCE).await;
    let status_before = ai_status(&app).await;
    assert_eq!(
        status_before,
        json!({
            "status": "not_configured",
            "is_ai_available": true,
            "key_source": PLATFORM_KEY_SOURCE,
            "unavailable_message": null,
            "platform_ai": {
                "token_allowance": GENEROUS_TOKEN_ALLOWANCE,
                "used_tokens": 0,
                "remaining_tokens": GENEROUS_TOKEN_ALLOWANCE,
                "is_allowance_used": false,
            },
        })
    );

    answer_order_status_question(&app, &connection_id).await;

    let usage_totals = usage_totals(&app).await;
    assert!(usage_totals.call_count > 12, "{}", usage_totals.call_count);
    assert_eq!(usage_totals.tenant_call_count, 0);
    assert!(usage_totals.platform_tokens > 0);
    assert_eq!(
        platform_ai_used_tokens(&app).await,
        usage_totals.platform_tokens
    );
    let status_after = ai_status(&app).await;
    assert_eq!(
        status_after["platform_ai"]["used_tokens"],
        usage_totals.platform_tokens
    );
    assert_eq!(
        status_after["platform_ai"]["remaining_tokens"],
        GENEROUS_TOKEN_ALLOWANCE as i64 - usage_totals.platform_tokens
    );
    assert!(
        !status_after.to_string().contains(PLATFORM_AI_API_KEY),
        "{status_after}"
    );
}

#[tokio::test]
async fn an_answer_that_crosses_the_allowance_finishes_and_later_work_is_refused() {
    let (app, connection_id) =
        app_with_platform_ai("anthropic", TOKEN_ALLOWANCE_BELOW_ONE_ANSWER).await;
    let chart_id = create_order_status_chart(&app, &connection_id).await;

    let conversation_id = answer_order_status_question(&app, &connection_id).await;

    assert!(platform_ai_used_tokens(&app).await > TOKEN_ALLOWANCE_BELOW_ONE_ANSWER as i64);
    let new_conversation = app
        .post(
            "/conversational_analytics",
            new_conversation_body(&connection_id),
        )
        .await;
    let follow_up = app
        .post(
            &format!("/conversational_analytics/{conversation_id}/messages"),
            json!({ "segments": [{ "type": "TEXT", "text": ORDER_STATUS_QUESTION }] }),
        )
        .await;
    let name_suggestion = app
        .post(
            &format!("/conversational_analytics/{conversation_id}/suggest-name"),
            json!({}),
        )
        .await;
    let chart_summary = app
        .post(&format!("/ai_summary/chart/{chart_id}"), json!({}))
        .await;
    for refused in [
        &new_conversation,
        &follow_up,
        &name_suggestion,
        &chart_summary,
    ] {
        assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.body);
        assert_eq!(
            refused.body["code"], ALLOWANCE_USED_CODE,
            "{}",
            refused.body
        );
        assert_eq!(refused.body["error"], ALLOWANCE_USED_MESSAGE);
    }
    let status = ai_status(&app).await;
    assert_eq!(status["is_ai_available"], false);
    assert_eq!(status["unavailable_message"], ALLOWANCE_USED_MESSAGE);
    assert_eq!(status["platform_ai"]["is_allowance_used"], true);
    assert_eq!(status["platform_ai"]["remaining_tokens"], 0);
    let canvases = app.get("/canvas?page=0&page_size=10").await;
    assert_eq!(canvases.status, StatusCode::OK, "{}", canvases.body);
    let members = app.get("/member").await;
    assert_eq!(members.status, StatusCode::OK, "{}", members.body);
    let fetched_conversation = app
        .get(&format!("/conversational_analytics/{conversation_id}"))
        .await;
    assert_eq!(fetched_conversation.status, StatusCode::OK);
}

#[tokio::test]
async fn adding_a_key_moves_the_tenant_to_its_own_key_and_removing_it_moves_it_back() {
    let (app, connection_id) =
        app_with_platform_ai("anthropic", TOKEN_ALLOWANCE_BELOW_ONE_ANSWER).await;
    answer_order_status_question(&app, &connection_id).await;
    let used_tokens_on_the_platform_key = platform_ai_used_tokens(&app).await;

    app.configure_live_ai_provider("anthropic").await;

    let status_on_own_key = ai_status(&app).await;
    assert_eq!(status_on_own_key["is_ai_available"], true);
    assert_eq!(status_on_own_key["key_source"], TENANT_KEY_SOURCE);
    assert_eq!(status_on_own_key["platform_ai"], Value::Null);
    answer_order_status_question(&app, &connection_id).await;
    assert_eq!(
        platform_ai_used_tokens(&app).await,
        used_tokens_on_the_platform_key
    );
    assert!(usage_totals(&app).await.tenant_call_count > 12);

    let removed = app.delete("/ai_configuration").await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);

    let status_back_on_the_platform_key = ai_status(&app).await;
    assert_eq!(
        status_back_on_the_platform_key["key_source"],
        PLATFORM_KEY_SOURCE
    );
    assert_eq!(status_back_on_the_platform_key["is_ai_available"], false);
    assert_eq!(
        status_back_on_the_platform_key["platform_ai"]["used_tokens"],
        used_tokens_on_the_platform_key
    );
}

#[tokio::test]
async fn a_tenant_whose_own_key_fails_is_refused_and_never_uses_the_platform_key() {
    let (app, connection_id) = app_with_platform_ai("anthropic", GENEROUS_TOKEN_ALLOWANCE).await;
    let chart_id = create_order_status_chart(&app, &connection_id).await;
    app.fake_llm
        .fail_next_request("liveness_check_agent", FailureKind::ServerError);
    let saved = app.configure_ai_provider("anthropic").await;
    assert_eq!(saved.body["status"], "failed", "{}", saved.body);

    let new_conversation = app
        .post(
            "/conversational_analytics",
            new_conversation_body(&connection_id),
        )
        .await;
    let chart_summary = app
        .post(&format!("/ai_summary/chart/{chart_id}"), json!({}))
        .await;

    assert_eq!(new_conversation.status, StatusCode::FORBIDDEN);
    assert_eq!(new_conversation.body["code"], AI_NOT_LIVE_CODE);
    assert_eq!(
        chart_summary.status,
        StatusCode::PRECONDITION_FAILED,
        "{}",
        chart_summary.body
    );
    let status = ai_status(&app).await;
    assert_eq!(status["is_ai_available"], false);
    assert_eq!(status["key_source"], TENANT_KEY_SOURCE);
    assert_eq!(status["platform_ai"], Value::Null);
    assert_eq!(platform_ai_used_tokens(&app).await, 0);
    assert_eq!(usage_totals(&app).await.platform_tokens, 0);
}

#[tokio::test]
async fn a_platform_key_call_that_reports_no_token_counts_fails_and_adds_nothing() {
    let (app, connection_id) = app_with_platform_ai("openai", GENEROUS_TOKEN_ALLOWANCE).await;
    let chart_id = create_order_status_chart(&app, &connection_id).await;
    app.fake_llm
        .fail_next_request(CHART_SUMMARY_AGENT, FailureKind::MissingUsage);

    let chart_summary = app
        .post(&format!("/ai_summary/chart/{chart_id}"), json!({}))
        .await;

    assert_eq!(
        chart_summary.status,
        StatusCode::OK,
        "{}",
        chart_summary.body
    );
    assert_eq!(
        chart_summary.body["status"], "FAILED",
        "{}",
        chart_summary.body
    );
    assert_eq!(usage_totals(&app).await.call_count, 0);
    assert_eq!(platform_ai_used_tokens(&app).await, 0);
}

#[tokio::test]
async fn a_tenant_with_no_key_is_refused_as_before_when_platform_ai_is_off() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    let connection = app.create_sqlite_demo_connection().await;

    let new_conversation = app
        .post(
            "/conversational_analytics",
            new_conversation_body(&string_field(&connection, "id")),
        )
        .await;

    assert_eq!(new_conversation.status, StatusCode::FORBIDDEN);
    assert_eq!(new_conversation.body["code"], AI_NOT_LIVE_CODE);
    assert_eq!(
        ai_status(&app).await,
        json!({
            "status": "not_configured",
            "is_ai_available": false,
            "key_source": null,
            "unavailable_message": "AI is not configured for this tenant. Add an AI key in Settings to use AI.",
            "platform_ai": null,
        })
    );
}

#[tokio::test]
async fn a_question_stored_just_before_the_allowance_ran_out_is_refused_when_its_answer_starts() {
    let (app, connection_id) =
        app_with_platform_ai("anthropic", TOKEN_ALLOWANCE_BELOW_ONE_ANSWER).await;
    let waiting_conversation_id =
        start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    answer_order_status_question(&app, &connection_id).await;

    let completion = app
        .post(
            &format!("/conversational_analytics/{waiting_conversation_id}/completion"),
            json!({}),
        )
        .await;

    assert_eq!(
        completion.status,
        StatusCode::FORBIDDEN,
        "{}",
        completion.body
    );
    assert_eq!(completion.body["code"], ALLOWANCE_USED_CODE);
}

#[tokio::test]
async fn platform_administrators_see_each_tenants_key_and_platform_ai_usage() {
    let (app, connection_id) = app_with_platform_ai("anthropic", GENEROUS_TOKEN_ALLOWANCE).await;
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    let events = run_completion(&app, &conversation_id).await;
    assert_eq!(
        event_names(&events).last().map(String::as_str),
        Some("completed")
    );
    let used_tokens = platform_ai_used_tokens(&app).await;

    let tenants_on_the_platform_key = app.get("/platform/tenant").await;
    app.configure_live_ai_provider("anthropic").await;
    let tenants_on_own_key = app.get("/platform/tenant").await;

    assert_eq!(tenants_on_the_platform_key.status, StatusCode::OK);
    let tenant_on_the_platform_key = &tenants_on_the_platform_key.body[0];
    assert_eq!(tenant_on_the_platform_key["ai_key_label"], "Platform AI");
    assert_eq!(
        tenant_on_the_platform_key["platform_ai_used_tokens"],
        used_tokens
    );
    assert_eq!(
        tenant_on_the_platform_key["platform_ai_token_allowance"],
        GENEROUS_TOKEN_ALLOWANCE
    );
    assert_eq!(tenants_on_own_key.body[0]["ai_key_label"], "Own key");
    assert_eq!(
        tenants_on_own_key.body[0]["platform_ai_used_tokens"],
        used_tokens
    );
    assert!(
        !tenants_on_own_key
            .body
            .to_string()
            .contains(AI_PROVIDER_API_KEY)
            && !tenants_on_own_key
                .body
                .to_string()
                .contains(PLATFORM_AI_API_KEY)
    );
}
