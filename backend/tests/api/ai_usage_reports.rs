use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    ADMIN_EMAIL, ADMIN_FULL_NAME, MEMBER_ROLE, ORDER_STATUS_QUESTION, OWNER_ROLE,
};
use crate::support::conversation::{
    answer_order_status_question, append_canvas_command, event_names, run_completion,
};
use crate::support::database::{insert_row, select_uuid, update_rows};
use crate::support::tenants::{
    add_member, create_custom_role, create_tenant, create_user, sign_in,
};
use crate::support::{TestApp, UserSession};
use chrono::{DateTime, FixedOffset};
use reqwest::StatusCode;
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use uuid::Uuid;

const RANGE_QUERY: &str = "from=2026-11-10&to=2026-11-12";
const USAGE_READER_ROLE: &str = "USAGE_READER";
const PLATFORM_MEMBER: &str = "MEMBER";
const PRIVATE_VISIBILITY: &str = "PRIVATE";
const DELETED_ACCOUNT_LABEL: &str = "Deleted account";

struct RecordedCall<'a> {
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    agent_name: &'a str,
    input_tokens: i64,
    output_tokens: i64,
    cached_input_tokens: i64,
    metadata: Value,
    created_at: &'a str,
}

fn moment(rfc3339_moment: &str) -> DateTime<FixedOffset> {
    DateTime::parse_from_rfc3339(rfc3339_moment).expect("a valid moment")
}

async fn record_call(app: &TestApp, recorded_call: RecordedCall<'_>) {
    insert_row(
        app,
        "ai_usage",
        &[
            ("id", Uuid::new_v4().into()),
            ("tenant_id", recorded_call.tenant_id.into()),
            ("user_id", recorded_call.user_id.into()),
            ("agent_name", recorded_call.agent_name.into()),
            ("provider", "anthropic".into()),
            ("model", "claude-haiku-4-5".into()),
            ("key_source", "TENANT".into()),
            ("input_tokens", recorded_call.input_tokens.into()),
            ("output_tokens", recorded_call.output_tokens.into()),
            (
                "cached_input_tokens",
                recorded_call.cached_input_tokens.into(),
            ),
            ("metadata", recorded_call.metadata.into()),
            ("created_at", moment(recorded_call.created_at).into()),
        ],
    )
    .await;
}

async fn admin_user_id(app: &TestApp) -> Uuid {
    select_uuid(app, "user", "id", &[("email", ADMIN_EMAIL.into())]).await
}

async fn usage(session: &UserSession, path_and_query: &str) -> Value {
    let response = session.get(&format!("/ai_usage/{path_and_query}")).await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.body);
    response.body
}

fn usage_paths(conversation_id: Uuid) -> [String; 6] {
    [
        "summary".to_string(),
        "over_time".to_string(),
        "by_user".to_string(),
        "by_kind".to_string(),
        "by_conversation".to_string(),
        format!("conversation/{conversation_id}/questions"),
    ]
}

async fn app_with_an_answered_and_named_conversation() -> (TestApp, String) {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    let connection = app.create_sqlite_demo_connection().await;
    let conversation_id =
        answer_order_status_question(&app, &string_field(&connection, "id")).await;
    let named = app
        .post(
            &format!("/conversational_analytics/{conversation_id}/suggest-name"),
            json!({}),
        )
        .await;
    assert_eq!(named.status, StatusCode::OK, "{}", named.body);
    (app, conversation_id)
}

fn questions_of(conversation_questions: &Value) -> Vec<Value> {
    conversation_questions["questions"]
        .as_array()
        .expect("questions")
        .clone()
}

#[tokio::test]
async fn usage_requests_refuse_people_without_the_permission_and_signed_out_callers() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let member = create_user(&app, "usage-member", PLATFORM_MEMBER).await;
    add_member(&app, app.tenant_id, member.id, MEMBER_ROLE).await;
    let member_session = sign_in(&app, &member).await.in_tenant(app.tenant_id);
    let signed_out_session = app.session(Some(app.tenant_id));

    for usage_path in usage_paths(Uuid::new_v4()) {
        let as_member = member_session.get(&format!("/ai_usage/{usage_path}")).await;
        let as_signed_out = signed_out_session
            .get(&format!("/ai_usage/{usage_path}"))
            .await;
        let as_owner = app.get(&format!("/ai_usage/{usage_path}")).await;

        assert_eq!(as_member.status, StatusCode::FORBIDDEN, "{usage_path}");
        assert_eq!(
            as_signed_out.status,
            StatusCode::UNAUTHORIZED,
            "{usage_path}"
        );
        assert_eq!(as_owner.status, StatusCode::OK, "{usage_path}");
    }
}

#[tokio::test]
async fn reports_add_up_the_calls_in_the_range_and_leave_out_the_rest() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let admin_user_id = admin_user_id(&app).await;
    let colleague = create_user(&app, "usage-colleague", PLATFORM_MEMBER).await;
    let conversation_id = Uuid::new_v4();
    let answer_message_id = Uuid::new_v4();
    let question_metadata = json!({
        "conversation_id": conversation_id,
        "conversation_message_id": answer_message_id,
    });
    for recorded_call in [
        RecordedCall {
            tenant_id: app.tenant_id,
            user_id: Some(admin_user_id),
            agent_name: "router_agent",
            input_tokens: 9000,
            output_tokens: 900,
            cached_input_tokens: 0,
            metadata: question_metadata.clone(),
            created_at: "2026-11-09T23:59:59Z",
        },
        RecordedCall {
            tenant_id: app.tenant_id,
            user_id: Some(admin_user_id),
            agent_name: "router_agent",
            input_tokens: 100,
            output_tokens: 20,
            cached_input_tokens: 10,
            metadata: question_metadata.clone(),
            created_at: "2026-11-10T00:00:00Z",
        },
        RecordedCall {
            tenant_id: app.tenant_id,
            user_id: Some(admin_user_id),
            agent_name: "sql_generation_agent",
            input_tokens: 50,
            output_tokens: 5,
            cached_input_tokens: 0,
            metadata: question_metadata.clone(),
            created_at: "2026-11-10T23:59:59Z",
        },
        RecordedCall {
            tenant_id: app.tenant_id,
            user_id: None,
            agent_name: "suggested_questions_agent",
            input_tokens: 30,
            output_tokens: 3,
            cached_input_tokens: 0,
            metadata: json!({}),
            created_at: "2026-11-12T08:30:00Z",
        },
        RecordedCall {
            tenant_id: app.tenant_id,
            user_id: Some(colleague.id),
            agent_name: "chart_summary_agent",
            input_tokens: 0,
            output_tokens: 0,
            cached_input_tokens: 0,
            metadata: json!({}),
            created_at: "2026-11-12T23:59:59Z",
        },
        RecordedCall {
            tenant_id: app.tenant_id,
            user_id: Some(admin_user_id),
            agent_name: "router_agent",
            input_tokens: 7000,
            output_tokens: 700,
            cached_input_tokens: 0,
            metadata: question_metadata.clone(),
            created_at: "2026-11-13T00:00:00Z",
        },
    ] {
        record_call(&app, recorded_call).await;
    }

    let summary = usage(&app, &format!("summary?{RANGE_QUERY}")).await;
    let over_time = usage(&app, &format!("over_time?{RANGE_QUERY}")).await;
    let by_user = usage(&app, &format!("by_user?{RANGE_QUERY}")).await;
    let by_kind = usage(&app, &format!("by_kind?{RANGE_QUERY}")).await;
    let by_conversation = usage(&app, &format!("by_conversation?{RANGE_QUERY}")).await;

    assert_eq!(
        summary,
        json!({
            "from": "2026-11-10",
            "to": "2026-11-12",
            "input_tokens": 180,
            "output_tokens": 28,
            "cached_input_tokens": 10,
            "total_tokens": 208,
            "call_count": 4,
            "has_usage": true,
            "is_platform_ai_on": false,
            "missing_token_counts_note": "1 model call in this range reported no token counts. It is included in the number of model calls but adds no tokens.",
        })
    );
    assert_eq!(
        over_time,
        json!({
            "days": [
                { "day": "2026-11-10", "input_tokens": 150, "output_tokens": 25, "total_tokens": 175, "call_count": 2 },
                { "day": "2026-11-11", "input_tokens": 0, "output_tokens": 0, "total_tokens": 0, "call_count": 0 },
                { "day": "2026-11-12", "input_tokens": 30, "output_tokens": 3, "total_tokens": 33, "call_count": 2 },
            ],
        })
    );
    assert_eq!(
        by_user,
        json!({
            "data": [
                { "user_id": admin_user_id, "name": ADMIN_FULL_NAME, "total_tokens": 175, "call_count": 2 },
                { "user_id": null, "name": "Background", "total_tokens": 33, "call_count": 1 },
                { "user_id": colleague.id, "name": colleague.full_name, "total_tokens": 0, "call_count": 1 },
            ],
            "total_pages": 1,
            "page": 0,
            "page_size": 10,
            "total_items": 3,
        })
    );
    let users_with_fewest_tokens_first = usage(
        &app,
        &format!("by_user?{RANGE_QUERY}&sort_by=total_tokens&sort_direction=asc"),
    )
    .await;
    assert_eq!(
        users_with_fewest_tokens_first["data"],
        json!([by_user["data"][2], by_user["data"][1], by_user["data"][0]])
    );
    let users_matching_a_search =
        usage(&app, &format!("by_user?{RANGE_QUERY}&search=%20BACKground")).await;
    assert_eq!(users_matching_a_search["data"], json!([by_user["data"][1]]));
    assert_eq!(users_matching_a_search["total_items"], 1);
    let conversations_matching_no_search = usage(
        &app,
        &format!("by_conversation?{RANGE_QUERY}&search=matches-no-conversation"),
    )
    .await;
    assert_eq!(conversations_matching_no_search["data"], json!([]));
    assert_eq!(conversations_matching_no_search["total_items"], 0);
    for refused_sort in [
        "by_user?sort_by=tenant_name",
        "by_conversation?sort_by=allowance_usage",
    ] {
        let refused = app
            .get(&format!("/ai_usage/{refused_sort}&{RANGE_QUERY}"))
            .await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{refused_sort}");
    }
    assert_eq!(
        by_kind,
        json!({
            "kinds": [
                { "kind": "ANSWERING_QUESTIONS", "label": "Answering questions", "total_tokens": 175, "call_count": 2 },
                { "kind": "SUGGESTED_QUESTIONS", "label": "Suggested questions", "total_tokens": 33, "call_count": 1 },
                { "kind": "SUMMARIES", "label": "Summaries", "total_tokens": 0, "call_count": 1 },
            ],
        })
    );
    assert_eq!(
        by_conversation,
        json!({
            "data": [
                {
                    "conversation_id": conversation_id,
                    "name": "Deleted conversation",
                    "name_kind": "DELETED",
                    "user_name": ADMIN_FULL_NAME,
                    "total_tokens": 175,
                    "call_count": 2,
                },
            ],
            "total_pages": 1,
            "page": 0,
            "page_size": 10,
            "total_items": 1,
        })
    );
}

#[tokio::test]
async fn a_summary_with_complete_token_counts_carries_no_note() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    let summary = usage(&app, &format!("summary?{RANGE_QUERY}")).await;

    assert_eq!(summary["call_count"], 0);
    assert_eq!(summary["has_usage"], false);
    assert_eq!(summary["total_tokens"], 0);
    assert_eq!(summary["missing_token_counts_note"], Value::Null);
}

#[tokio::test]
async fn usage_of_one_tenant_is_not_returned_for_another() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let other_tenant_id = create_tenant(&app, "usage-other").await;
    let other_owner = create_user(&app, "usage-other-owner", PLATFORM_MEMBER).await;
    add_member(&app, other_tenant_id, other_owner.id, OWNER_ROLE).await;
    let other_owner_session = sign_in(&app, &other_owner).await.in_tenant(other_tenant_id);
    let conversation_id = Uuid::new_v4();
    record_call(
        &app,
        RecordedCall {
            tenant_id: app.tenant_id,
            user_id: Some(admin_user_id(&app).await),
            agent_name: "router_agent",
            input_tokens: 100,
            output_tokens: 20,
            cached_input_tokens: 0,
            metadata: json!({ "conversation_id": conversation_id }),
            created_at: "2026-11-11T12:00:00Z",
        },
    )
    .await;

    let summary = usage(&other_owner_session, &format!("summary?{RANGE_QUERY}")).await;
    let by_user = usage(&other_owner_session, &format!("by_user?{RANGE_QUERY}")).await;
    let by_kind = usage(&other_owner_session, &format!("by_kind?{RANGE_QUERY}")).await;
    let by_conversation = usage(
        &other_owner_session,
        &format!("by_conversation?{RANGE_QUERY}"),
    )
    .await;
    let conversation_questions = usage(
        &other_owner_session,
        &format!("conversation/{conversation_id}/questions?{RANGE_QUERY}"),
    )
    .await;

    assert_eq!(summary["call_count"], 0);
    assert_eq!(by_user["data"], json!([]));
    assert_eq!(by_kind["kinds"], json!([]));
    assert_eq!(by_conversation["data"], json!([]));
    assert_eq!(conversation_questions["questions"], json!([]));
    assert_eq!(
        usage(&app, &format!("summary?{RANGE_QUERY}")).await["call_count"],
        1
    );
}

#[tokio::test]
async fn conversations_are_listed_largest_first_one_page_at_a_time() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let admin_user_id = admin_user_id(&app).await;
    let conversation_ids = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    for (conversation_id, input_tokens) in conversation_ids.iter().zip([10, 300, 20]) {
        record_call(
            &app,
            RecordedCall {
                tenant_id: app.tenant_id,
                user_id: Some(admin_user_id),
                agent_name: "router_agent",
                input_tokens,
                output_tokens: 0,
                cached_input_tokens: 0,
                metadata: json!({ "conversation_id": conversation_id }),
                created_at: "2026-11-11T12:00:00Z",
            },
        )
        .await;
    }

    let first_page = usage(
        &app,
        &format!("by_conversation?{RANGE_QUERY}&page=0&page_size=2"),
    )
    .await;
    let second_page = usage(
        &app,
        &format!("by_conversation?{RANGE_QUERY}&page=1&page_size=2"),
    )
    .await;

    let tokens_on = |page: &Value| -> Vec<i64> {
        page["data"]
            .as_array()
            .expect("conversations")
            .iter()
            .map(|conversation| conversation["total_tokens"].as_i64().expect("tokens"))
            .collect()
    };
    assert_eq!(tokens_on(&first_page), [300, 20]);
    assert_eq!(tokens_on(&second_page), [10]);
    assert_eq!(first_page["total_items"], 3);
    assert_eq!(first_page["total_pages"], 2);
    for refused_page in [
        "page=2&page_size=2",
        "page=0&page_size=0",
        "page=0&page_size=51",
    ] {
        let refused = app
            .get(&format!(
                "/ai_usage/by_conversation?{RANGE_QUERY}&{refused_page}"
            ))
            .await;
        assert_eq!(
            refused.status,
            StatusCode::BAD_REQUEST,
            "{refused_page}: {}",
            refused.body
        );
    }
}

#[tokio::test]
async fn a_conversation_row_names_the_user_who_used_the_most_tokens_in_it() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let colleague = create_user(&app, "usage-heavy-colleague", PLATFORM_MEMBER).await;
    let conversation_id = Uuid::new_v4();
    for (user_id, input_tokens) in [
        (Some(admin_user_id(&app).await), 40),
        (Some(colleague.id), 90),
        (None, 10),
    ] {
        record_call(
            &app,
            RecordedCall {
                tenant_id: app.tenant_id,
                user_id,
                agent_name: "router_agent",
                input_tokens,
                output_tokens: 0,
                cached_input_tokens: 0,
                metadata: json!({ "conversation_id": conversation_id }),
                created_at: "2026-11-11T12:00:00Z",
            },
        )
        .await;
    }

    let by_conversation = usage(&app, &format!("by_conversation?{RANGE_QUERY}")).await;

    assert_eq!(by_conversation["total_items"], 1);
    assert_eq!(by_conversation["data"][0]["total_tokens"], 140);
    assert_eq!(by_conversation["data"][0]["call_count"], 3);
    assert_eq!(by_conversation["data"][0]["user_name"], colleague.full_name);
}

#[tokio::test]
async fn usage_of_a_user_who_no_longer_exists_is_labelled_as_a_deleted_account() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let deleted_user_id = Uuid::new_v4();
    let conversation_id = Uuid::new_v4();
    record_call(
        &app,
        RecordedCall {
            tenant_id: app.tenant_id,
            user_id: Some(deleted_user_id),
            agent_name: "router_agent",
            input_tokens: 70,
            output_tokens: 5,
            cached_input_tokens: 0,
            metadata: json!({ "conversation_id": conversation_id }),
            created_at: "2026-11-11T12:00:00Z",
        },
    )
    .await;

    let by_user = usage(&app, &format!("by_user?{RANGE_QUERY}")).await;
    let by_conversation = usage(&app, &format!("by_conversation?{RANGE_QUERY}")).await;

    assert_eq!(
        by_user["data"],
        json!([
            { "user_id": deleted_user_id, "name": DELETED_ACCOUNT_LABEL, "total_tokens": 75, "call_count": 1 },
        ])
    );
    assert_eq!(
        by_conversation["data"][0]["user_name"],
        DELETED_ACCOUNT_LABEL
    );
}

#[tokio::test]
async fn a_number_of_days_gives_a_range_that_ends_today() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    let over_time = usage(&app, "over_time?last_day_count=7").await;
    let summary = usage(&app, "summary?last_day_count=7").await;
    let ambiguous = app
        .get("/ai_usage/summary?last_day_count=7&from=2026-11-01")
        .await;
    let page_out_of_range = app
        .get("/ai_usage/by_conversation?page=18446744073709551615&page_size=50")
        .await;

    let days = over_time["days"].as_array().expect("days");
    assert_eq!(days.len(), 7);
    assert_eq!(days[0]["day"], summary["from"]);
    assert_eq!(days[6]["day"], summary["to"]);
    assert_eq!(summary["to"], chrono::Utc::now().date_naive().to_string());
    assert_eq!(
        ambiguous.status,
        StatusCode::BAD_REQUEST,
        "{}",
        ambiguous.body
    );
    assert_eq!(
        page_out_of_range.status,
        StatusCode::BAD_REQUEST,
        "{}",
        page_out_of_range.body
    );
}

#[tokio::test]
async fn a_range_that_is_reversed_too_long_or_unreadable_is_refused() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    let reversed = app
        .get("/ai_usage/summary?from=2026-11-12&to=2026-11-10")
        .await;
    let too_long = app
        .get("/ai_usage/summary?from=2025-11-14&to=2026-11-15")
        .await;
    let unreadable = app.get("/ai_usage/summary?from=yesterday").await;

    assert_eq!(
        reversed.status,
        StatusCode::BAD_REQUEST,
        "{}",
        reversed.body
    );
    assert_eq!(
        reversed.body["error"],
        "The date range starts after it ends"
    );
    assert_eq!(
        too_long.status,
        StatusCode::BAD_REQUEST,
        "{}",
        too_long.body
    );
    assert_eq!(
        too_long.body["error"],
        "The date range may not be longer than 366 days"
    );
    assert_eq!(unreadable.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn an_answered_question_shows_up_in_every_report_and_its_rows_add_up() {
    let (app, conversation_id) = app_with_an_answered_and_named_conversation().await;
    append_canvas_command(&app, &conversation_id).await;
    let canvas_events = run_completion(&app, &conversation_id).await;
    assert_eq!(
        event_names(&canvas_events).last().map(String::as_str),
        Some("completed")
    );

    let summary = usage(&app, "summary").await;
    let over_time = usage(&app, "over_time").await;
    let by_conversation = usage(&app, "by_conversation").await;
    let conversation_questions =
        usage(&app, &format!("conversation/{conversation_id}/questions")).await;

    let days = over_time["days"].as_array().expect("days");
    assert_eq!(days.len(), 30);
    assert_eq!(days[29]["call_count"], summary["call_count"]);
    assert_eq!(days[29]["total_tokens"], summary["total_tokens"]);
    let conversation = &by_conversation["data"][0];
    assert_eq!(by_conversation["total_items"], 1);
    assert_eq!(conversation["conversation_id"], conversation_id);
    assert_eq!(conversation["name"], "Orders by status");
    assert_eq!(conversation["name_kind"], "NAME");
    assert_eq!(conversation["user_name"], ADMIN_FULL_NAME);
    assert_eq!(
        conversation_questions["total_tokens"],
        conversation["total_tokens"]
    );
    assert_eq!(
        conversation_questions["call_count"],
        conversation["call_count"]
    );
    assert_eq!(conversation_questions["name"], "Orders by status");
    assert_eq!(conversation_questions["question_count"], 2);
    let questions = questions_of(&conversation_questions);
    let text_and_kind: Vec<(String, String)> = questions
        .iter()
        .map(|question| {
            (
                string_field(question, "text"),
                string_field(question, "text_kind"),
            )
        })
        .collect();
    assert_eq!(
        text_and_kind,
        [
            (ORDER_STATUS_QUESTION.to_string(), "TEXT".to_string()),
            (
                "Conversation naming and other model calls outside a question".to_string(),
                "NO_QUESTION".to_string()
            ),
            ("Canvas command".to_string(), "COMMAND".to_string()),
        ]
    );
    let question_tokens: i64 = questions
        .iter()
        .map(|question| question["total_tokens"].as_i64().expect("tokens"))
        .sum();
    let question_calls: i64 = questions
        .iter()
        .map(|question| question["call_count"].as_i64().expect("calls"))
        .sum();
    assert_eq!(json!(question_tokens), conversation["total_tokens"]);
    assert_eq!(json!(question_calls), conversation["call_count"]);
}

#[tokio::test]
async fn a_private_conversation_is_labelled_for_a_viewer_who_may_not_read_it() {
    let (app, conversation_id) = app_with_an_answered_and_named_conversation().await;
    update_rows(
        &app,
        "conversation",
        &[("visibility", PRIVATE_VISIBILITY.into())],
        &[(
            "id",
            Uuid::parse_str(&conversation_id)
                .expect("conversation id")
                .into(),
        )],
    )
    .await;
    let usage_reader_role_id = create_custom_role(
        &app,
        app.tenant_id,
        USAGE_READER_ROLE,
        &["usage:read", "conversation:read"],
    )
    .await;
    let usage_reader = create_user(&app, "usage-reader", PLATFORM_MEMBER).await;
    insert_row(
        &app,
        "tenant_membership",
        &crate::support::tenants::with_system_attribution(vec![
            ("id", Uuid::new_v4().into()),
            ("tenant_id", app.tenant_id.into()),
            ("user_id", usage_reader.id.into()),
            ("role_id", usage_reader_role_id.into()),
            ("status", "ACTIVE".into()),
        ]),
    )
    .await;
    let usage_reader_session = sign_in(&app, &usage_reader).await.in_tenant(app.tenant_id);

    let by_conversation_for_reader = usage(&usage_reader_session, "by_conversation").await;
    let questions_for_reader = usage(
        &usage_reader_session,
        &format!("conversation/{conversation_id}/questions"),
    )
    .await;
    let by_conversation_for_owner = usage(&app, "by_conversation").await;

    let conversation_for_reader = &by_conversation_for_reader["data"][0];
    assert_eq!(conversation_for_reader["name"], "Private conversation");
    assert_eq!(conversation_for_reader["name_kind"], "PRIVATE");
    assert_eq!(
        conversation_for_reader["total_tokens"],
        by_conversation_for_owner["data"][0]["total_tokens"]
    );
    assert_eq!(questions_for_reader["name"], "Private conversation");
    let questions = questions_of(&questions_for_reader);
    assert_eq!(questions[0]["text"], "Question in a private conversation");
    assert_eq!(questions[0]["text_kind"], "PRIVATE");
    assert!(
        !questions_for_reader
            .to_string()
            .contains(ORDER_STATUS_QUESTION),
        "{questions_for_reader}"
    );
    assert_eq!(by_conversation_for_owner["data"][0]["name_kind"], "NAME");
}

#[tokio::test]
async fn a_deleted_conversation_keeps_its_tokens_and_is_labelled_as_deleted() {
    let (app, conversation_id) = app_with_an_answered_and_named_conversation().await;
    let summary_before_deletion = usage(&app, "summary").await;
    let deleted = app
        .delete(&format!("/conversational_analytics/{conversation_id}"))
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);

    let summary = usage(&app, "summary").await;
    let by_conversation = usage(&app, "by_conversation").await;
    let conversation_questions =
        usage(&app, &format!("conversation/{conversation_id}/questions")).await;

    assert_eq!(summary, summary_before_deletion);
    assert_eq!(by_conversation["data"][0]["name"], "Deleted conversation");
    assert_eq!(by_conversation["data"][0]["name_kind"], "DELETED");
    assert_eq!(
        by_conversation["data"][0]["total_tokens"],
        conversation_questions["total_tokens"]
    );
    let questions = questions_of(&conversation_questions);
    assert_eq!(questions[0]["text"], "Question in a deleted conversation");
    assert_eq!(questions[0]["text_kind"], "DELETED");
}

#[tokio::test]
async fn an_existing_tenant_gains_the_usage_permission_at_the_next_startup() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.app_state
        .database_connection
        .execute_unprepared("DELETE FROM role_permission WHERE permission = 'usage:read'")
        .await
        .expect("remove the usage permission from every role");
    let refused = app.get("/ai_usage/summary").await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.body);

    app.restart_initialization().await;

    let allowed = app.get("/ai_usage/summary").await;
    assert_eq!(allowed.status, StatusCode::OK, "{}", allowed.body);
}
