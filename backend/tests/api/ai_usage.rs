use crate::support::canvas_builder::{
    create_canvas, create_chart_cell, create_sql_cell, order_status_chart_content,
    place_chart_on_dashboard, string_field,
};
use crate::support::constants::{
    ADMIN_EMAIL, ADMIN_ROLE, AI_PROVIDER_API_KEY, MEMBER_ROLE, ORDER_STATUS_QUERY,
    ORDER_STATUS_QUESTION, VIEWER_ROLE,
};
use crate::support::conversation::{event_names, last_message, run_completion, start_conversation};
use crate::support::database::{backend, select_rows, select_uuid};
use crate::support::tenants::join_tenant;
use crate::support::{TestApp, UserSession};
use fake_llm::FailureKind;
use reqwest::StatusCode;
use sea_orm::{ConnectionTrait, DbBackend};
use serde_json::{Value, json};
use uuid::Uuid;

const USAGE_TABLE: &str = "ai_usage";
const USAGE_READ_PERMISSION: &str = "usage:read";
const ANTHROPIC_MODEL: &str = "claude-haiku-4-5";
const OPENAI_MODEL: &str = "gpt-4o-mini";
const TENANT_KEY_SOURCE: &str = "TENANT";
const ROUTER_AGENT: &str = "router_agent";
const RESULT_NARRATION_AGENT: &str = "result_narration_agent";
const FOLLOWUP_QUESTIONS_AGENT: &str = "followup_questions_agent";
const CONVERSATION_NAME_AGENT: &str = "conversation_name_agent";
const LIVENESS_CHECK_AGENT: &str = "liveness_check_agent";
const CHART_SUMMARY_AGENT: &str = "chart_summary_agent";
const DASHBOARD_SUMMARY_AGENT: &str = "dashboard_summary_agent";
const SUGGESTED_QUESTIONS_AGENT: &str = "suggested_questions_agent";

#[derive(Debug, Clone, PartialEq)]
struct UsageRow {
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    agent_name: String,
    provider: String,
    model: String,
    key_source: String,
    input_tokens: i64,
    output_tokens: i64,
    cached_input_tokens: i64,
    metadata: Value,
}

async fn usage_rows(app: &TestApp) -> Vec<UsageRow> {
    select_rows(
        app,
        USAGE_TABLE,
        &[
            "tenant_id",
            "user_id",
            "agent_name",
            "provider",
            "model",
            "key_source",
            "input_tokens",
            "output_tokens",
            "cached_input_tokens",
            "metadata",
        ],
        &[],
    )
    .await
    .iter()
    .map(|row| UsageRow {
        tenant_id: row.try_get("", "tenant_id").expect("tenant_id"),
        user_id: row.try_get("", "user_id").expect("user_id"),
        agent_name: row.try_get("", "agent_name").expect("agent_name"),
        provider: row.try_get("", "provider").expect("provider"),
        model: row.try_get("", "model").expect("model"),
        key_source: row.try_get("", "key_source").expect("key_source"),
        input_tokens: row.try_get("", "input_tokens").expect("input_tokens"),
        output_tokens: row.try_get("", "output_tokens").expect("output_tokens"),
        cached_input_tokens: row
            .try_get("", "cached_input_tokens")
            .expect("cached_input_tokens"),
        metadata: row.try_get("", "metadata").expect("metadata"),
    })
    .collect()
}

async fn usage_rows_of_agent(app: &TestApp, agent_name: &str) -> Vec<UsageRow> {
    usage_rows(app)
        .await
        .into_iter()
        .filter(|usage_row| usage_row.agent_name == agent_name)
        .collect()
}

async fn admin_user_id(app: &TestApp) -> Uuid {
    select_uuid(app, "user", "id", &[("email", ADMIN_EMAIL.into())]).await
}

async fn refuse_usage_rows_of_agent(app: &TestApp, agent_name: &str) {
    let statement = match backend(app) {
        DbBackend::Sqlite => format!(
            "CREATE TRIGGER refuse_ai_usage BEFORE INSERT ON ai_usage \
             WHEN NEW.agent_name = '{agent_name}' \
             BEGIN SELECT RAISE(ABORT, 'ai usage refused by the test'); END"
        ),
        DbBackend::Postgres => format!(
            "ALTER TABLE ai_usage ADD CONSTRAINT refuse_ai_usage CHECK (agent_name <> '{agent_name}')"
        ),
        other_backend => panic!("unsupported metadata database {other_backend:?}"),
    };
    app.app_state
        .database_connection
        .execute_unprepared(&statement)
        .await
        .expect("refuse usage rows of the agent");
}

async fn drop_usage_table(app: &TestApp) {
    app.app_state
        .database_connection
        .execute_unprepared("DROP TABLE ai_usage")
        .await
        .expect("drop the usage table");
}

async fn app_with_live_ai(provider: &str) -> (TestApp, String) {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider(provider).await;
    let connection = app.create_sqlite_demo_connection().await;
    let connection_id = string_field(&connection, "id");
    (app, connection_id)
}

async fn create_order_status_chart(app: &TestApp, connection_id: &str) -> String {
    let canvas = create_canvas(app, "Usage").await;
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

async fn summarize_chart(app: &TestApp, chart_id: &str) -> Value {
    let chart_summary = app
        .post(&format!("/ai_summary/chart/{chart_id}"), json!({}))
        .await;
    assert_eq!(
        chart_summary.status,
        StatusCode::OK,
        "{}",
        chart_summary.body
    );
    chart_summary.body
}

async fn permissions_in_tenant(session: &UserSession, tenant_id: Uuid) -> Vec<String> {
    let memberships = session.get("/tenant").await;
    assert_eq!(memberships.status, StatusCode::OK, "{}", memberships.body);
    memberships
        .body
        .as_array()
        .expect("memberships")
        .iter()
        .find(|membership| membership["tenant_id"] == tenant_id.to_string())
        .expect("the person belongs to the tenant")["permissions"]
        .as_array()
        .expect("permissions")
        .iter()
        .map(|permission| permission.as_str().expect("permission name").to_string())
        .collect()
}

#[tokio::test]
async fn answering_a_question_records_one_row_per_model_call() {
    let (app, connection_id) = app_with_live_ai("anthropic").await;
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    let rows_before_the_answer = usage_rows(&app).await.len();
    let requests_before_the_answer = app.fake_llm.received_requests().len();

    let events = run_completion(&app, &conversation_id).await;

    let names = event_names(&events);
    assert_eq!(
        names.last().map(String::as_str),
        Some("completed"),
        "{names:?}"
    );
    let answer_rows: Vec<UsageRow> = usage_rows(&app)
        .await
        .into_iter()
        .skip(rows_before_the_answer)
        .collect();
    let model_calls = app.fake_llm.received_requests().len() - requests_before_the_answer;
    assert!(
        model_calls > 12,
        "the answer made {model_calls} model calls"
    );
    assert_eq!(answer_rows.len(), model_calls);
    let answer_message_id = string_field(&last_message(&app, &conversation_id).await, "id");
    let admin_user_id = admin_user_id(&app).await;
    for answer_row in &answer_rows {
        assert_eq!(answer_row.tenant_id, app.tenant_id, "{answer_row:?}");
        assert_eq!(answer_row.user_id, Some(admin_user_id), "{answer_row:?}");
        assert_eq!(answer_row.provider, "anthropic", "{answer_row:?}");
        assert_eq!(answer_row.model, ANTHROPIC_MODEL, "{answer_row:?}");
        assert_eq!(answer_row.key_source, TENANT_KEY_SOURCE, "{answer_row:?}");
        assert_eq!(
            answer_row.metadata,
            json!({
                "conversation_id": conversation_id,
                "conversation_message_id": answer_message_id,
            }),
            "{answer_row:?}"
        );
    }
    let router_rows: Vec<&UsageRow> = answer_rows
        .iter()
        .filter(|answer_row| answer_row.agent_name == ROUTER_AGENT)
        .collect();
    assert_eq!(router_rows.len(), 1);
    assert_eq!(router_rows[0].input_tokens, 150);
    assert_eq!(router_rows[0].output_tokens, 15);
    assert_eq!(router_rows[0].cached_input_tokens, 30);
}

#[tokio::test]
async fn the_key_check_and_name_suggestion_record_rows_that_belong_to_no_question() {
    let (app, connection_id) = app_with_live_ai("anthropic").await;
    let admin_user_id = admin_user_id(&app).await;

    let tested = app
        .post(
            "/ai_configuration/test",
            json!({ "provider": "anthropic", "api_key": AI_PROVIDER_API_KEY }),
        )
        .await;
    assert_eq!(tested.body, json!({ "ok": true }));
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    let named = app
        .post(
            &format!("/conversational_analytics/{conversation_id}/suggest-name"),
            json!({}),
        )
        .await;
    assert_eq!(named.status, StatusCode::OK, "{}", named.body);

    let key_check_rows = usage_rows_of_agent(&app, LIVENESS_CHECK_AGENT).await;
    assert_eq!(key_check_rows.len(), 2, "saving and testing the key");
    for key_check_row in &key_check_rows {
        assert_eq!(key_check_row.user_id, Some(admin_user_id));
        assert_eq!(key_check_row.tenant_id, app.tenant_id);
        assert_eq!(key_check_row.metadata, json!({}));
    }
    let naming_rows = usage_rows_of_agent(&app, CONVERSATION_NAME_AGENT).await;
    assert_eq!(naming_rows.len(), 1);
    assert_eq!(naming_rows[0].user_id, Some(admin_user_id));
    assert_eq!(
        naming_rows[0].metadata,
        json!({ "conversation_id": conversation_id })
    );
    assert_eq!(naming_rows[0].input_tokens, 40);
    assert_eq!(naming_rows[0].output_tokens, 6);
    assert_eq!(naming_rows[0].cached_input_tokens, 0);
}

#[tokio::test]
async fn the_background_suggested_question_refresh_records_rows_with_no_person() {
    let (app, _connection_id) = app_with_live_ai("anthropic").await;

    sopho::refresh_stale_suggested_questions(&app.app_state).await;

    let refresh_rows = usage_rows_of_agent(&app, SUGGESTED_QUESTIONS_AGENT).await;
    assert_eq!(refresh_rows.len(), 2, "one per connection");
    for refresh_row in &refresh_rows {
        assert_eq!(refresh_row.user_id, None);
        assert_eq!(refresh_row.tenant_id, app.tenant_id);
        assert_eq!(refresh_row.metadata, json!({}));
    }
}

#[tokio::test]
async fn openai_summary_records_cached_input_inside_the_input_count() {
    let (app, connection_id) = app_with_live_ai("openai").await;
    let chart_id = create_order_status_chart(&app, &connection_id).await;

    let chart_summary = summarize_chart(&app, &chart_id).await;

    assert_eq!(chart_summary["status"], "READY", "{chart_summary}");
    let summary_rows = usage_rows_of_agent(&app, CHART_SUMMARY_AGENT).await;
    assert_eq!(summary_rows.len(), 1);
    assert_eq!(summary_rows[0].provider, "openai");
    assert_eq!(summary_rows[0].model, OPENAI_MODEL);
    assert_eq!(summary_rows[0].user_id, Some(admin_user_id(&app).await));
    assert_eq!(summary_rows[0].metadata, json!({}));
    assert_eq!(summary_rows[0].input_tokens, 200);
    assert_eq!(summary_rows[0].output_tokens, 60);
    assert_eq!(summary_rows[0].cached_input_tokens, 50);
}

#[tokio::test]
async fn a_dashboard_summary_records_a_row_for_the_person_who_asked_for_it() {
    let (app, connection_id) = app_with_live_ai("anthropic").await;
    let canvas = create_canvas(&app, "Dashboard usage").await;
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
    place_chart_on_dashboard(&app, &canvas, &string_field(&chart, "id")).await;

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
        dashboard_summary.body["status"], "READY",
        "{}",
        dashboard_summary.body
    );
    let summary_rows = usage_rows_of_agent(&app, DASHBOARD_SUMMARY_AGENT).await;
    assert_eq!(summary_rows.len(), 1);
    assert_eq!(summary_rows[0].user_id, Some(admin_user_id(&app).await));
    assert_eq!(summary_rows[0].tenant_id, app.tenant_id);
    assert_eq!(summary_rows[0].metadata, json!({}));
}

#[tokio::test]
async fn a_response_without_token_counts_is_recorded_as_a_call_with_zero_tokens() {
    let (app, connection_id) = app_with_live_ai("openai").await;
    let chart_id = create_order_status_chart(&app, &connection_id).await;
    app.fake_llm
        .fail_next_request(CHART_SUMMARY_AGENT, FailureKind::MissingUsage);

    let chart_summary = summarize_chart(&app, &chart_id).await;

    assert_eq!(chart_summary["status"], "READY", "{chart_summary}");
    let summary_rows = usage_rows_of_agent(&app, CHART_SUMMARY_AGENT).await;
    assert_eq!(summary_rows.len(), 1);
    assert_eq!(summary_rows[0].input_tokens, 0);
    assert_eq!(summary_rows[0].output_tokens, 0);
    assert_eq!(summary_rows[0].cached_input_tokens, 0);
}

#[tokio::test]
async fn a_call_that_fails_at_the_provider_records_nothing() {
    let (app, connection_id) = app_with_live_ai("anthropic").await;
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    app.fake_llm
        .fail_next_request(ROUTER_AGENT, FailureKind::ServerError);

    let events = run_completion(&app, &conversation_id).await;

    assert_eq!(event_names(&events), ["starting", "routing", "error"]);
    assert_eq!(usage_rows_of_agent(&app, ROUTER_AGENT).await, []);
}

#[tokio::test]
async fn an_answer_that_cannot_be_parsed_is_recorded_and_the_call_fails() {
    let (app, connection_id) = app_with_live_ai("anthropic").await;
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    app.fake_llm
        .fail_next_request(ROUTER_AGENT, FailureKind::UnparsableAnswer);

    let events = run_completion(&app, &conversation_id).await;

    assert_eq!(event_names(&events), ["starting", "routing", "error"]);
    let router_rows = usage_rows_of_agent(&app, ROUTER_AGENT).await;
    assert_eq!(router_rows.len(), 1);
    assert_eq!(router_rows[0].input_tokens, 150);
    assert_eq!(router_rows[0].output_tokens, 15);
}

#[tokio::test]
async fn an_answer_fails_when_its_usage_cannot_be_recorded() {
    let (app, connection_id) = app_with_live_ai("anthropic").await;
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    drop_usage_table(&app).await;

    let events = run_completion(&app, &conversation_id).await;

    assert_eq!(event_names(&events), ["starting", "routing", "error"]);
    assert_eq!(
        last_message(&app, &conversation_id).await["status"],
        "FAILED"
    );
}

async fn assert_answer_fails_when_usage_of_agent_is_refused(agent_name: &str) {
    let (app, connection_id) = app_with_live_ai("anthropic").await;
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    refuse_usage_rows_of_agent(&app, agent_name).await;

    let events = run_completion(&app, &conversation_id).await;

    let names = event_names(&events);
    assert!(names.contains(&"executed_query".to_string()), "{names:?}");
    assert_eq!(names.last().map(String::as_str), Some("error"), "{names:?}");
    assert_eq!(
        last_message(&app, &conversation_id).await["status"],
        "FAILED"
    );
}

#[tokio::test]
async fn an_answer_fails_when_the_usage_of_result_narration_cannot_be_recorded() {
    assert_answer_fails_when_usage_of_agent_is_refused(RESULT_NARRATION_AGENT).await;
}

#[tokio::test]
async fn an_answer_fails_when_the_usage_of_followup_questions_cannot_be_recorded() {
    assert_answer_fails_when_usage_of_agent_is_refused(FOLLOWUP_QUESTIONS_AGENT).await;
}

#[tokio::test]
async fn name_suggestion_and_the_key_check_fail_when_usage_cannot_be_recorded() {
    let (app, connection_id) = app_with_live_ai("anthropic").await;
    let conversation_id = start_conversation(&app, &connection_id, ORDER_STATUS_QUESTION).await;
    drop_usage_table(&app).await;

    let named = app
        .post(
            &format!("/conversational_analytics/{conversation_id}/suggest-name"),
            json!({}),
        )
        .await;
    let tested = app
        .post(
            "/ai_configuration/test",
            json!({ "provider": "anthropic", "api_key": AI_PROVIDER_API_KEY }),
        )
        .await;
    let saved = app.configure_ai_provider("anthropic").await;

    assert_eq!(
        named.status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "{}",
        named.body
    );
    assert_eq!(
        tested.status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "{}",
        tested.body
    );
    assert_eq!(
        saved.status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "{}",
        saved.body
    );
}

#[tokio::test]
async fn owners_and_admins_may_read_usage_and_members_and_viewers_may_not() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let admin = join_tenant(&app, app.tenant_id, "usage-admin", ADMIN_ROLE).await;
    let member = join_tenant(&app, app.tenant_id, "usage-member", MEMBER_ROLE).await;
    let viewer = join_tenant(&app, app.tenant_id, "usage-viewer", VIEWER_ROLE).await;
    let usage_read = USAGE_READ_PERMISSION.to_string();

    assert!(
        permissions_in_tenant(&app, app.tenant_id)
            .await
            .contains(&usage_read)
    );
    assert!(
        permissions_in_tenant(&admin.session, app.tenant_id)
            .await
            .contains(&usage_read)
    );
    assert!(
        !permissions_in_tenant(&member.session, app.tenant_id)
            .await
            .contains(&usage_read)
    );
    assert!(
        !permissions_in_tenant(&viewer.session, app.tenant_id)
            .await
            .contains(&usage_read)
    );
}
