use crate::support::canvas_builder::create_canvas;
use crate::support::constants::{
    ACTIVE_TENANT_STATUS, ADMIN_EMAIL, MEMBER_ROLE, OWNER_ROLE, PLATFORM_ADMINISTRATOR,
    PLATFORM_MEMBER, SUSPENDED_TENANT_STATUS, TENANT_SUSPEND_PERMISSION, TENANT_SUSPENDED_CODE,
    TENANT_SUSPENDED_MESSAGE,
};
use crate::support::database::select_rows;
use crate::support::mail_server::shared_mail_server;
use crate::support::tenants::{create_tenant, create_user, join_tenant, sign_in};
use crate::support::test_app::JsonResponse;
use crate::support::{TestApp, UserSession};
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

async fn suspend(session: &UserSession, tenant_id: Uuid) -> JsonResponse {
    session
        .post(&format!("/platform/tenant/{tenant_id}/suspend"), json!({}))
        .await
}

async fn restore(session: &UserSession, tenant_id: Uuid) -> JsonResponse {
    session
        .post(&format!("/platform/tenant/{tenant_id}/restore"), json!({}))
        .await
}

fn assert_suspended(response: &JsonResponse, route: &str) {
    assert_eq!(
        response.status,
        StatusCode::FORBIDDEN,
        "{route}: {}",
        response.body
    );
    assert_eq!(
        response.body["error_code"], TENANT_SUSPENDED_CODE,
        "{route}"
    );
    assert_eq!(
        response.body["message"], TENANT_SUSPENDED_MESSAGE,
        "{route}"
    );
}

async fn membership_of(session: &UserSession, tenant_id: Uuid) -> Value {
    let memberships = session.get("/tenant").await;
    assert_eq!(memberships.status, StatusCode::OK, "{}", memberships.body);
    memberships
        .body
        .as_array()
        .expect("memberships")
        .iter()
        .find(|membership| membership["tenant_id"] == tenant_id.to_string())
        .unwrap_or_else(|| panic!("no membership of {tenant_id}: {}", memberships.body))
        .clone()
}

async fn platform_tenant(app: &TestApp, tenant_id: Uuid) -> Value {
    let tenants = app.get("/platform/tenant").await;
    assert_eq!(tenants.status, StatusCode::OK, "{}", tenants.body);
    tenants
        .body
        .as_array()
        .expect("platform tenants")
        .iter()
        .find(|tenant| tenant["tenant_id"] == tenant_id.to_string())
        .unwrap_or_else(|| panic!("tenant {tenant_id} is not listed: {}", tenants.body))
        .clone()
}

#[tokio::test]
async fn a_suspended_tenant_refuses_every_member_and_comes_back_with_its_data() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let suspended_tenant_id = create_tenant(&app, "Suspended").await;
    let other_tenant_id = create_tenant(&app, "Untouched").await;
    let owner = join_tenant(&app, suspended_tenant_id, "owner", OWNER_ROLE).await;
    let member = join_tenant(&app, suspended_tenant_id, "member", MEMBER_ROLE).await;
    let other_owner = join_tenant(&app, other_tenant_id, "other", OWNER_ROLE).await;
    let canvas = create_canvas(&owner.session, "Kept canvas").await;
    let canvas_id = canvas.canvas_id;

    let suspended = suspend(&app, suspended_tenant_id).await;
    assert_eq!(
        suspended.status,
        StatusCode::NO_CONTENT,
        "{}",
        suspended.body
    );

    for session in [&owner.session, &member.session] {
        for route in [
            "/canvas",
            "/connection",
            "/member",
            "/role",
            "/ai_configuration",
            "/conversational_analytics",
            "/search?query=kept",
        ] {
            assert_suspended(&session.get(route).await, route);
        }
        assert_suspended(
            &session
                .post("/canvas", json!({ "name": "Refused canvas" }))
                .await,
            "POST /canvas",
        );
        let invitation_route = format!("/tenant/{suspended_tenant_id}/invitation");
        assert_suspended(&session.get(&invitation_route).await, &invitation_route);
        assert_eq!(
            membership_of(session, suspended_tenant_id).await["suspended_message"],
            TENANT_SUSPENDED_MESSAGE
        );
    }
    assert_eq!(
        other_owner.session.get("/canvas").await.status,
        StatusCode::OK
    );
    assert_eq!(
        membership_of(&other_owner.session, other_tenant_id).await["suspended_message"],
        Value::Null
    );
    assert_eq!(
        owner.session.get("/auth/me").await.status,
        StatusCode::OK,
        "suspending a tenant ends no session"
    );

    let restored = restore(&app, suspended_tenant_id).await;
    assert_eq!(restored.status, StatusCode::NO_CONTENT, "{}", restored.body);

    let kept_canvas = owner.session.get(&format!("/canvas/{canvas_id}")).await;
    assert_eq!(kept_canvas.status, StatusCode::OK, "{}", kept_canvas.body);
    assert_eq!(kept_canvas.body["name"], "Kept canvas");
    assert_eq!(member.session.get("/canvas").await.status, StatusCode::OK);
    assert_eq!(
        membership_of(&owner.session, suspended_tenant_id).await["suspended_message"],
        Value::Null
    );
}

#[tokio::test]
async fn the_platform_tenant_list_reports_status_and_what_can_be_done() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let tenant_id = create_tenant(&app, "Listed").await;

    let active_tenant = platform_tenant(&app, tenant_id).await;
    assert_eq!(active_tenant["status"], ACTIVE_TENANT_STATUS);
    assert_eq!(
        active_tenant["capabilities"],
        json!({ "can_suspend": true, "can_restore": false })
    );

    assert_eq!(
        suspend(&app, tenant_id).await.status,
        StatusCode::NO_CONTENT
    );

    let suspended_tenant = platform_tenant(&app, tenant_id).await;
    assert_eq!(suspended_tenant["status"], SUSPENDED_TENANT_STATUS);
    assert_eq!(
        suspended_tenant["capabilities"],
        json!({ "can_suspend": false, "can_restore": true })
    );
    let stored_tenant = select_rows(
        &app,
        "tenant",
        &["status", "modified_by_actor"],
        &[("id", tenant_id.into())],
    )
    .await;
    assert_eq!(
        stored_tenant[0].try_get::<String>("", "status").unwrap(),
        SUSPENDED_TENANT_STATUS
    );
    assert_eq!(
        stored_tenant[0]
            .try_get::<String>("", "modified_by_actor")
            .unwrap(),
        "HUMAN"
    );

    let second_administrator = sign_in(
        &app,
        &create_user(&app, "second", PLATFORM_ADMINISTRATOR).await,
    )
    .await;
    assert_eq!(
        suspend(&second_administrator, tenant_id).await.status,
        StatusCode::NO_CONTENT,
        "suspending a suspended tenant succeeds"
    );
    let suspended_again = select_rows(
        &app,
        "tenant",
        &["modified_by_user_id", "updated_at"],
        &[("id", tenant_id.into())],
    )
    .await;
    let first_suspension =
        select_rows(&app, "user", &["id"], &[("email", ADMIN_EMAIL.into())]).await;
    assert_eq!(
        suspended_again[0]
            .try_get::<Option<Uuid>>("", "modified_by_user_id")
            .unwrap(),
        Some(first_suspension[0].try_get::<Uuid>("", "id").unwrap()),
        "a repeated suspension keeps who suspended the tenant first"
    );
}

#[tokio::test]
async fn only_platform_administrators_suspend_and_restore() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;
    let tenant_id = create_tenant(&app, "Guarded").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let outsider = sign_in(&app, &create_user(&app, "outsider", PLATFORM_MEMBER).await).await;

    for session in [&owner.session, &outsider] {
        let refused = suspend(session, tenant_id).await;
        assert_eq!(refused.status, StatusCode::FORBIDDEN, "{}", refused.body);
        assert_eq!(
            refused.body["message"],
            format!("Permission denied: {TENANT_SUSPEND_PERMISSION}")
        );
        assert_eq!(
            restore(session, tenant_id).await.status,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(owner.session.get("/canvas").await.status, StatusCode::OK);
    let signed_out = suspend(&app.session(None), tenant_id).await;
    assert_eq!(signed_out.status, StatusCode::UNAUTHORIZED);

    let unknown_tenant = suspend(&app, Uuid::new_v4()).await;
    assert_eq!(
        unknown_tenant.status,
        StatusCode::NOT_FOUND,
        "{}",
        unknown_tenant.body
    );
}

#[tokio::test]
async fn an_administrator_can_suspend_a_tenant_they_belong_to() {
    let app = TestApp::spawn().await;
    app.login_as_admin().await;

    assert_eq!(
        suspend(&app, app.tenant_id).await.status,
        StatusCode::NO_CONTENT
    );
    assert_suspended(&app.get("/canvas").await, "/canvas");
    assert_eq!(app.get("/platform/tenant").await.status, StatusCode::OK);

    assert_eq!(
        restore(&app, app.tenant_id).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(app.get("/canvas").await.status, StatusCode::OK);
}

#[tokio::test]
async fn the_background_refresh_skips_a_suspended_tenant() {
    let mut app = TestApp::spawn().await;
    app.login_as_admin().await;
    app.configure_live_ai_provider("anthropic").await;
    app.create_sqlite_demo_connection().await;
    assert_eq!(
        suspend(&app, app.tenant_id).await.status,
        StatusCode::NO_CONTENT
    );
    let llm_requests_before_refresh = app.fake_llm.received_requests().len();

    sopho::refresh_stale_suggested_questions(&app.app_state).await;

    assert_eq!(
        app.fake_llm.received_requests().len(),
        llm_requests_before_refresh
    );

    assert_eq!(
        restore(&app, app.tenant_id).await.status,
        StatusCode::NO_CONTENT
    );
    sopho::refresh_stale_suggested_questions(&app.app_state).await;

    assert!(
        app.fake_llm.received_requests().len() > llm_requests_before_refresh,
        "a restored tenant is refreshed again"
    );
}

#[tokio::test]
async fn queued_emails_of_a_suspended_tenant_wait_until_it_is_restored() {
    let app = TestApp::spawn_with_email().await;
    app.login_as_admin().await;
    let tenant_id = create_tenant(&app, "Mailing").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let invitee_email = format!("held-{}@example.test", Uuid::new_v4().simple());
    let invited = owner
        .session
        .post(
            &format!("/tenant/{tenant_id}/invitation"),
            json!({ "email": invitee_email, "role_name": MEMBER_ROLE }),
        )
        .await;
    assert_eq!(invited.status, StatusCode::CREATED, "{}", invited.body);
    assert_eq!(
        suspend(&app, tenant_id).await.status,
        StatusCode::NO_CONTENT
    );

    app.deliver_due_emails().await;

    assert!(
        shared_mail_server()
            .await
            .emails_to(&invitee_email)
            .await
            .is_empty(),
        "a suspended tenant's queued email was sent"
    );

    assert_eq!(
        restore(&app, tenant_id).await.status,
        StatusCode::NO_CONTENT
    );
    app.deliver_due_emails().await;

    assert_eq!(
        shared_mail_server()
            .await
            .emails_to(&invitee_email)
            .await
            .len(),
        1
    );
}
