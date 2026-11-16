use crate::support::canvas_builder::string_field;
use crate::support::constants::{
    ADMIN_EMAIL, ADMIN_FULL_NAME, ORDER_STATUS_QUESTION, OWNER_ROLE, PLATFORM_ADMINISTRATOR,
    PLATFORM_MEMBER,
};
use crate::support::conversation::answer_order_status_question;
use crate::support::database::{insert_row, now, select_uuid, update_rows};
use crate::support::tenants::{add_member, create_tenant, create_user, sign_in};
use crate::support::{DeploymentFlags, TestApp, UserSession};
use chrono::{DateTime, FixedOffset};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

const RANGE_QUERY: &str = "from=2026-11-10&to=2026-11-12";
const TOKEN_ALLOWANCE: u64 = 1_000_000;
const PLATFORM_KEY_SOURCE: &str = "PLATFORM";
const TENANT_KEY_SOURCE: &str = "TENANT";
const PRIVATE_VISIBILITY: &str = "PRIVATE";

struct RecordedCall<'a> {
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    key_source: &'a str,
    agent_name: &'a str,
    total_tokens: i64,
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
            ("key_source", recorded_call.key_source.into()),
            ("input_tokens", recorded_call.total_tokens.into()),
            ("output_tokens", 0_i64.into()),
            ("cached_input_tokens", 0_i64.into()),
            ("metadata", recorded_call.metadata.into()),
            ("created_at", moment(recorded_call.created_at).into()),
        ],
    )
    .await;
}

async fn record_running_total(app: &TestApp, tenant_id: Uuid, used_tokens: i64) {
    insert_row(
        app,
        "platform_ai_usage",
        &[
            ("tenant_id", tenant_id.into()),
            ("used_tokens", used_tokens.into()),
            ("updated_at", now().into()),
        ],
    )
    .await;
}

async fn app_with_platform_ai() -> TestApp {
    let app = TestApp::spawn_with_deployment_flags(DeploymentFlags::with_platform_ai(
        "anthropic",
        TOKEN_ALLOWANCE,
    ))
    .await;
    app.login_as_admin().await;
    app
}

async fn admin_user_id(app: &TestApp) -> Uuid {
    select_uuid(app, "user", "id", &[("email", ADMIN_EMAIL.into())]).await
}

async fn tenant_usage(session: &UserSession, path_and_query: &str) -> Value {
    let response = session.get(&format!("/ai_usage/{path_and_query}")).await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.body);
    response.body
}

async fn platform_usage(session: &UserSession, path_and_query: &str) -> Value {
    let response = session
        .get(&format!("/platform/ai_usage/{path_and_query}"))
        .await;
    assert_eq!(response.status, StatusCode::OK, "{}", response.body);
    response.body
}

fn platform_usage_paths(tenant_id: Uuid, conversation_id: Uuid) -> [String; 7] {
    [
        "summary".to_string(),
        "over_time".to_string(),
        "by_kind".to_string(),
        "by_tenant".to_string(),
        "by_user".to_string(),
        "by_conversation".to_string(),
        format!("tenant/{tenant_id}/conversation/{conversation_id}/questions"),
    ]
}

#[tokio::test]
async fn a_tenant_can_look_at_its_platform_ai_usage_and_its_own_key_usage_apart() {
    let app = app_with_platform_ai().await;
    let admin_user_id = admin_user_id(&app).await;
    let conversation_id = Uuid::new_v4();
    let answer_message_id = Uuid::new_v4();
    let question_metadata = json!({
        "conversation_id": conversation_id,
        "conversation_message_id": answer_message_id,
    });
    for (key_source, agent_name, total_tokens) in [
        (PLATFORM_KEY_SOURCE, "router_agent", 100),
        (PLATFORM_KEY_SOURCE, "chart_summary_agent", 30),
        (TENANT_KEY_SOURCE, "router_agent", 7000),
    ] {
        record_call(
            &app,
            RecordedCall {
                tenant_id: app.tenant_id,
                user_id: Some(admin_user_id),
                key_source,
                agent_name,
                total_tokens,
                metadata: match agent_name {
                    "router_agent" => question_metadata.clone(),
                    _ => json!({}),
                },
                created_at: "2026-11-11T12:00:00Z",
            },
        )
        .await;
    }

    let all_usage = tenant_usage(&app, &format!("summary?{RANGE_QUERY}")).await;
    let platform_ai_usage =
        tenant_usage(&app, &format!("summary?{RANGE_QUERY}&key_source=PLATFORM")).await;
    let own_key_usage =
        tenant_usage(&app, &format!("summary?{RANGE_QUERY}&key_source=TENANT")).await;
    let platform_ai_by_kind =
        tenant_usage(&app, &format!("by_kind?{RANGE_QUERY}&key_source=PLATFORM")).await;
    let platform_ai_by_user =
        tenant_usage(&app, &format!("by_user?{RANGE_QUERY}&key_source=PLATFORM")).await;
    let platform_ai_over_time = tenant_usage(
        &app,
        &format!("over_time?{RANGE_QUERY}&key_source=PLATFORM"),
    )
    .await;
    let platform_ai_by_conversation = tenant_usage(
        &app,
        &format!("by_conversation?{RANGE_QUERY}&key_source=PLATFORM"),
    )
    .await;
    let own_key_questions = tenant_usage(
        &app,
        &format!("conversation/{conversation_id}/questions?{RANGE_QUERY}&key_source=TENANT"),
    )
    .await;
    let unknown_key = app
        .get(&format!(
            "/ai_usage/summary?{RANGE_QUERY}&key_source=SOMEONE"
        ))
        .await;

    assert_eq!(all_usage["total_tokens"], 7130);
    assert_eq!(all_usage["is_platform_ai_on"], true);
    assert_eq!(platform_ai_usage["total_tokens"], 130);
    assert_eq!(platform_ai_usage["call_count"], 2);
    assert_eq!(own_key_usage["total_tokens"], 7000);
    assert_eq!(own_key_usage["call_count"], 1);
    assert_eq!(
        platform_ai_by_kind["kinds"],
        json!([
            { "kind": "ANSWERING_QUESTIONS", "label": "Answering questions", "total_tokens": 100, "call_count": 1 },
            { "kind": "SUMMARIES", "label": "Summaries", "total_tokens": 30, "call_count": 1 },
        ])
    );
    assert_eq!(platform_ai_by_user["users"][0]["total_tokens"], 130);
    assert_eq!(platform_ai_over_time["days"][1]["total_tokens"], 130);
    assert_eq!(platform_ai_by_conversation["data"][0]["total_tokens"], 100);
    assert_eq!(own_key_questions["total_tokens"], 7000);
    assert_eq!(unknown_key.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn platform_administrators_see_platform_key_usage_across_tenants() {
    let app = app_with_platform_ai().await;
    let admin_user_id = admin_user_id(&app).await;
    let second_tenant_id = create_tenant(&app, "Second tenant").await;
    let second_tenant_owner = create_user(&app, "second-owner", PLATFORM_MEMBER).await;
    add_member(&app, second_tenant_id, second_tenant_owner.id, OWNER_ROLE).await;
    let conversation_id = Uuid::new_v4();
    for recorded_call in [
        RecordedCall {
            tenant_id: app.tenant_id,
            user_id: Some(admin_user_id),
            key_source: PLATFORM_KEY_SOURCE,
            agent_name: "router_agent",
            total_tokens: 100,
            metadata: json!({ "conversation_id": conversation_id }),
            created_at: "2026-11-10T09:00:00Z",
        },
        RecordedCall {
            tenant_id: second_tenant_id,
            user_id: Some(second_tenant_owner.id),
            key_source: PLATFORM_KEY_SOURCE,
            agent_name: "chart_summary_agent",
            total_tokens: 400,
            metadata: json!({}),
            created_at: "2026-11-12T09:00:00Z",
        },
        RecordedCall {
            tenant_id: second_tenant_id,
            user_id: None,
            key_source: PLATFORM_KEY_SOURCE,
            agent_name: "suggested_questions_agent",
            total_tokens: 50,
            metadata: json!({}),
            created_at: "2026-11-12T10:00:00Z",
        },
        RecordedCall {
            tenant_id: second_tenant_id,
            user_id: Some(second_tenant_owner.id),
            key_source: TENANT_KEY_SOURCE,
            agent_name: "router_agent",
            total_tokens: 9000,
            metadata: json!({}),
            created_at: "2026-11-11T09:00:00Z",
        },
        RecordedCall {
            tenant_id: second_tenant_id,
            user_id: Some(second_tenant_owner.id),
            key_source: PLATFORM_KEY_SOURCE,
            agent_name: "router_agent",
            total_tokens: 8000,
            metadata: json!({}),
            created_at: "2026-11-13T00:00:00Z",
        },
    ] {
        record_call(&app, recorded_call).await;
    }
    record_running_total(&app, app.tenant_id, 100).await;
    record_running_total(&app, second_tenant_id, 8450).await;

    let summary = platform_usage(&app, &format!("summary?{RANGE_QUERY}")).await;
    let over_time = platform_usage(&app, &format!("over_time?{RANGE_QUERY}")).await;
    let by_kind = platform_usage(&app, &format!("by_kind?{RANGE_QUERY}")).await;
    let by_tenant = platform_usage(&app, &format!("by_tenant?{RANGE_QUERY}")).await;
    let by_user = platform_usage(&app, &format!("by_user?{RANGE_QUERY}")).await;
    let by_conversation = platform_usage(&app, &format!("by_conversation?{RANGE_QUERY}")).await;
    let first_page_of_users =
        platform_usage(&app, &format!("by_user?{RANGE_QUERY}&page=0&page_size=2")).await;

    assert_eq!(summary["total_tokens"], 550);
    assert_eq!(summary["call_count"], 3);
    let tokens_per_day: Vec<i64> = over_time["days"]
        .as_array()
        .expect("days")
        .iter()
        .map(|day| day["total_tokens"].as_i64().expect("tokens"))
        .collect();
    assert_eq!(tokens_per_day, [100, 0, 450]);
    assert_eq!(
        by_kind["kinds"],
        json!([
            { "kind": "SUMMARIES", "label": "Summaries", "total_tokens": 400, "call_count": 1 },
            { "kind": "ANSWERING_QUESTIONS", "label": "Answering questions", "total_tokens": 100, "call_count": 1 },
            { "kind": "SUGGESTED_QUESTIONS", "label": "Suggested questions", "total_tokens": 50, "call_count": 1 },
        ])
    );
    assert_eq!(
        by_tenant["data"],
        json!([
            {
                "tenant_id": second_tenant_id,
                "tenant_name": "Second tenant",
                "total_tokens": 450,
                "call_count": 2,
                "platform_ai_used_tokens": 8450,
                "platform_ai_token_allowance": TOKEN_ALLOWANCE,
            },
            {
                "tenant_id": app.tenant_id,
                "tenant_name": "Default",
                "total_tokens": 100,
                "call_count": 1,
                "platform_ai_used_tokens": 100,
                "platform_ai_token_allowance": TOKEN_ALLOWANCE,
            },
        ])
    );
    assert_eq!(
        by_user["data"],
        json!([
            {
                "user_id": second_tenant_owner.id,
                "name": second_tenant_owner.full_name,
                "tenant_id": second_tenant_id,
                "tenant_name": "Second tenant",
                "total_tokens": 400,
                "call_count": 1,
            },
            {
                "user_id": admin_user_id,
                "name": ADMIN_FULL_NAME,
                "tenant_id": app.tenant_id,
                "tenant_name": "Default",
                "total_tokens": 100,
                "call_count": 1,
            },
            {
                "user_id": null,
                "name": "Background",
                "tenant_id": second_tenant_id,
                "tenant_name": "Second tenant",
                "total_tokens": 50,
                "call_count": 1,
            },
        ])
    );
    assert_eq!(
        first_page_of_users["data"].as_array().expect("users").len(),
        2
    );
    assert_eq!(first_page_of_users["total_items"], 3);
    assert_eq!(first_page_of_users["total_pages"], 2);
    let second_page_of_users =
        platform_usage(&app, &format!("by_user?{RANGE_QUERY}&page=1&page_size=2")).await;
    assert_eq!(second_page_of_users["data"], json!([by_user["data"][2]]));
    assert_eq!(second_page_of_users["page"], 1);
    let second_page_of_tenants =
        platform_usage(&app, &format!("by_tenant?{RANGE_QUERY}&page=1&page_size=1")).await;
    assert_eq!(
        second_page_of_tenants["data"],
        json!([by_tenant["data"][1]])
    );
    assert_eq!(second_page_of_tenants["total_items"], 2);
    assert_eq!(second_page_of_tenants["total_pages"], 2);
    for refused_request in [
        "by_tenant?page=2&page_size=1",
        "by_user?page=2&page_size=2",
        "by_conversation?page=1&page_size=10",
        "by_tenant?page=0&page_size=0",
        "by_user?page=0&page_size=51",
        "summary?key_source=TENANT",
        "by_tenant?key_source=PLATFORM",
    ] {
        let refused = app
            .get(&format!(
                "/platform/ai_usage/{refused_request}&{RANGE_QUERY}"
            ))
            .await;
        assert_eq!(
            refused.status,
            StatusCode::BAD_REQUEST,
            "{refused_request}: {}",
            refused.body
        );
    }
    assert_eq!(
        by_conversation["data"],
        json!([
            {
                "conversation_id": conversation_id,
                "tenant_id": app.tenant_id,
                "tenant_name": "Default",
                "name": "Deleted conversation",
                "name_kind": "DELETED",
                "user_name": ADMIN_FULL_NAME,
                "total_tokens": 100,
                "call_count": 1,
            },
        ])
    );
}

#[tokio::test]
async fn a_platform_administrator_outside_the_tenant_sees_a_private_conversations_name_and_questions()
 {
    let mut app = app_with_platform_ai().await;
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
    let outside_administrator = sign_in(
        &app,
        &create_user(&app, "outside-administrator", PLATFORM_ADMINISTRATOR).await,
    )
    .await;

    let by_conversation = platform_usage(&outside_administrator, "by_conversation").await;
    let conversation_questions = platform_usage(
        &outside_administrator,
        &format!(
            "tenant/{}/conversation/{conversation_id}/questions",
            app.tenant_id
        ),
    )
    .await;
    let questions_under_another_tenant = platform_usage(
        &outside_administrator,
        &format!(
            "tenant/{}/conversation/{conversation_id}/questions",
            Uuid::new_v4()
        ),
    )
    .await;

    let conversation = &by_conversation["data"][0];
    assert_eq!(conversation["conversation_id"], conversation_id);
    assert_eq!(conversation["name"], "Orders by status");
    assert_eq!(conversation["name_kind"], "NAME");
    assert_eq!(conversation["tenant_name"], "Default");
    assert_eq!(conversation["user_name"], ADMIN_FULL_NAME);
    assert_eq!(conversation_questions["name"], "Orders by status");
    assert_eq!(
        conversation_questions["total_tokens"],
        conversation["total_tokens"]
    );
    let questions = conversation_questions["questions"]
        .as_array()
        .expect("questions");
    assert_eq!(questions[0]["text"], ORDER_STATUS_QUESTION);
    assert_eq!(questions[0]["text_kind"], "TEXT");
    assert_eq!(questions[1]["text_kind"], "NO_QUESTION");
    assert_eq!(questions_under_another_tenant["questions"], json!([]));
    assert_eq!(questions_under_another_tenant["name_kind"], "DELETED");
    assert_ne!(questions_under_another_tenant["name"], "Orders by status");
}

#[tokio::test]
async fn platform_ai_usage_is_refused_without_the_platform_permission() {
    let app = app_with_platform_ai().await;
    let platform_member = sign_in(
        &app,
        &create_user(&app, "platform-member", PLATFORM_MEMBER).await,
    )
    .await;
    let signed_out_session = app.session(None);

    for usage_path in platform_usage_paths(app.tenant_id, Uuid::new_v4()) {
        let as_member = platform_member
            .get(&format!("/platform/ai_usage/{usage_path}"))
            .await;
        let as_signed_out = signed_out_session
            .get(&format!("/platform/ai_usage/{usage_path}"))
            .await;
        let as_administrator = app.get(&format!("/platform/ai_usage/{usage_path}")).await;

        assert_eq!(as_member.status, StatusCode::FORBIDDEN, "{usage_path}");
        assert_eq!(
            as_signed_out.status,
            StatusCode::UNAUTHORIZED,
            "{usage_path}"
        );
        assert_eq!(as_administrator.status, StatusCode::OK, "{usage_path}");
    }
    let reversed_range = app
        .get("/platform/ai_usage/summary?from=2026-11-12&to=2026-11-10")
        .await;
    assert_eq!(reversed_range.status, StatusCode::BAD_REQUEST);
}
