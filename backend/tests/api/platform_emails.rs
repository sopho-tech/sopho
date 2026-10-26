use crate::support::constants::{
    EMAIL_FAILED_STATUS, EMAIL_NOT_RETRYABLE_CODE, EMAIL_OUTBOX_TABLE, EMAIL_PENDING_STATUS,
    MEMBER_ROLE, OWNER_ROLE, PLATFORM_MEMBER,
};
use crate::support::database::{now, select_rows, select_uuid, update_rows};
use crate::support::tenants::{TenantMember, create_tenant, create_user, join_tenant, sign_in};
use crate::support::{TestApp, UserSession};
use chrono::Duration;
use reqwest::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

fn unique_recipient(label: &str) -> String {
    format!("{label}-{}@example.test", Uuid::new_v4().simple())
}

async fn invite(owner: &TenantMember, tenant_id: Uuid, recipient: &str) {
    let response = owner
        .session
        .post(
            &format!("/tenant/{tenant_id}/invitation"),
            json!({ "email": recipient, "role_name": MEMBER_ROLE }),
        )
        .await;
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.body);
}

async fn email_log(session: &UserSession, query: &str) -> Value {
    let log = session.get(&format!("/platform/email{query}")).await;
    assert_eq!(log.status, StatusCode::OK, "{}", log.body);
    log.body
}

fn recipients(log: &Value) -> Vec<String> {
    log["emails"]
        .as_array()
        .expect("emails")
        .iter()
        .map(|email| {
            email["recipient"]
                .as_str()
                .expect("a recipient")
                .to_string()
        })
        .collect()
}

async fn email_id_to(app: &TestApp, recipient: &str) -> Uuid {
    select_uuid(
        app,
        EMAIL_OUTBOX_TABLE,
        "id",
        &[("recipient", recipient.into())],
    )
    .await
}

#[tokio::test]
async fn the_email_log_needs_email_manage() {
    let app = TestApp::spawn_with_email().await;
    let member = create_user(&app, "member", PLATFORM_MEMBER).await;
    let member_session = sign_in(&app, &member).await;

    assert_eq!(
        member_session.get("/platform/email").await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        member_session
            .post(
                &format!("/platform/email/{}/retry", Uuid::new_v4()),
                json!({})
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn the_email_log_filters_by_status_and_recipient() {
    let app = TestApp::spawn_with_email().await;
    app.login_as_admin().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let delivered_recipient = unique_recipient("delivered");
    let waiting_recipient = unique_recipient("waiting");
    invite(&owner, tenant_id, &delivered_recipient).await;
    app.deliver_due_emails().await;
    invite(&owner, tenant_id, &waiting_recipient).await;

    let everything = email_log(&app, "").await;
    assert_eq!(everything["total"], 2);
    assert_eq!(everything["total_pages"], 1);
    assert_eq!(
        recipients(&everything),
        vec![waiting_recipient.clone(), delivered_recipient.clone()],
        "newest first"
    );

    let pending = email_log(&app, &format!("?status={EMAIL_PENDING_STATUS}")).await;
    assert_eq!(recipients(&pending), vec![waiting_recipient.clone()]);
    assert_eq!(pending["emails"][0]["kind"], "INVITATION");

    let searched = email_log(&app, &format!("?search={}", &delivered_recipient[..12])).await;
    assert_eq!(recipients(&searched), vec![delivered_recipient]);
    assert_eq!(searched["emails"][0]["status"], "SENT");
    assert_eq!(searched["emails"][0]["can_retry"], false);
}

#[tokio::test]
async fn retry_requeues_a_failed_email_only_while_its_links_are_valid() {
    let app = TestApp::spawn_with_refusing_mail_server().await;
    app.login_as_admin().await;
    let tenant_id = create_tenant(&app, "acme").await;
    let owner = join_tenant(&app, tenant_id, "owner", OWNER_ROLE).await;
    let recipient = unique_recipient("refused");
    invite(&owner, tenant_id, &recipient).await;
    app.deliver_due_emails().await;
    let email_id = email_id_to(&app, &recipient).await;
    let failed = email_log(&app, &format!("?search={recipient}")).await;
    assert_eq!(failed["emails"][0]["status"], EMAIL_FAILED_STATUS);
    assert_eq!(failed["emails"][0]["can_retry"], true);

    let retried = app
        .post(&format!("/platform/email/{email_id}/retry"), json!({}))
        .await;
    assert_eq!(retried.status, StatusCode::NO_CONTENT, "{}", retried.body);
    let requeued = select_rows(
        &app,
        EMAIL_OUTBOX_TABLE,
        &["status", "attempt_count"],
        &[("id", email_id.into())],
    )
    .await;
    assert_eq!(
        requeued[0].try_get::<String>("", "status").unwrap(),
        EMAIL_PENDING_STATUS
    );
    assert_eq!(requeued[0].try_get::<i32>("", "attempt_count").unwrap(), 0);

    update_rows(
        &app,
        EMAIL_OUTBOX_TABLE,
        &[
            ("status", EMAIL_FAILED_STATUS.into()),
            ("send_before", (now() - Duration::minutes(1)).into()),
        ],
        &[("id", email_id.into())],
    )
    .await;
    let expired_log = email_log(&app, &format!("?search={recipient}")).await;
    assert_eq!(expired_log["emails"][0]["can_retry"], false);
    let expired = app
        .post(&format!("/platform/email/{email_id}/retry"), json!({}))
        .await;
    assert_eq!(expired.status, StatusCode::CONFLICT, "{}", expired.body);
    assert_eq!(expired.body["error_code"], EMAIL_NOT_RETRYABLE_CODE);

    assert_eq!(
        app.post(
            &format!("/platform/email/{}/retry", Uuid::new_v4()),
            json!({})
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}
