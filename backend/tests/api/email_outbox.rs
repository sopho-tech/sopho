use crate::support::TestApp;
use crate::support::constants::{
    EMAIL_FAILED_STATUS, EMAIL_OUTBOX_TABLE, EMAIL_PENDING_STATUS, EMAIL_SENDING_STATUS,
    EMAIL_SENT_STATUS, EXPIRED_EMAIL_MESSAGE, INVITATION_EMAIL_KIND, INVITATION_LINK_PATH,
    INVITATION_TABLE, MEMBER_ROLE, OWNER_ROLE, SILENT_MAIL_SERVER_DELIVERY_DEADLINE_SECONDS,
    SILENT_MAIL_SERVER_TIMEOUT_SECONDS, SMTP_TIMED_OUT_MESSAGE, UNDECRYPTABLE_EMAIL_MESSAGE,
};
use crate::support::database::{count_rows, now, select_rows, select_uuid, update_rows};
use crate::support::mail_server::{SilentMailServer, shared_mail_server};
use crate::support::tenants::{TenantMember, create_tenant, join_tenant};
use chrono::{DateTime, Duration, FixedOffset};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

struct QueuedEmail {
    id: Uuid,
    kind: String,
    status: String,
    attempt_count: i32,
    last_error: Option<String>,
    encrypted_text_body: Option<String>,
    encrypted_html_body: Option<String>,
    next_attempt_at: DateTime<FixedOffset>,
    sent_at: Option<DateTime<FixedOffset>>,
}

fn unique_recipient(label: &str) -> String {
    format!("{label}-{}@example.test", Uuid::new_v4().simple())
}

async fn tenant_with_owner(app: &TestApp) -> (Uuid, TenantMember) {
    let tenant_id = create_tenant(app, "acme").await;
    let owner = join_tenant(app, tenant_id, "owner", OWNER_ROLE).await;
    (tenant_id, owner)
}

async fn invite_by_email(owner: &TenantMember, tenant_id: Uuid, recipient: &str) -> Uuid {
    let response = owner
        .session
        .post(
            &format!("/tenant/{tenant_id}/invitation"),
            json!({ "email": recipient, "role_name": MEMBER_ROLE }),
        )
        .await;
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.body);
    response.body["id"]
        .as_str()
        .expect("an invitation id")
        .parse()
        .expect("the invitation id is a UUID")
}

async fn queued_emails_to(app: &TestApp, recipient: &str) -> Vec<QueuedEmail> {
    select_rows(
        app,
        EMAIL_OUTBOX_TABLE,
        &[
            "id",
            "kind",
            "status",
            "attempt_count",
            "last_error",
            "encrypted_text_body",
            "encrypted_html_body",
            "next_attempt_at",
            "sent_at",
        ],
        &[("recipient", recipient.into())],
    )
    .await
    .iter()
    .map(|row| QueuedEmail {
        id: row.try_get("", "id").expect("read the email id"),
        kind: row.try_get("", "kind").expect("read the email kind"),
        status: row.try_get("", "status").expect("read the email status"),
        attempt_count: row
            .try_get("", "attempt_count")
            .expect("read the attempt count"),
        last_error: row.try_get("", "last_error").expect("read the last error"),
        encrypted_text_body: row
            .try_get("", "encrypted_text_body")
            .expect("read the text body"),
        encrypted_html_body: row
            .try_get("", "encrypted_html_body")
            .expect("read the HTML body"),
        next_attempt_at: row
            .try_get("", "next_attempt_at")
            .expect("read the next attempt time"),
        sent_at: row.try_get("", "sent_at").expect("read the sent time"),
    })
    .collect()
}

async fn the_queued_email_to(app: &TestApp, recipient: &str) -> QueuedEmail {
    let mut queued_emails = queued_emails_to(app, recipient).await;
    assert_eq!(queued_emails.len(), 1, "expected one email to {recipient}");
    queued_emails.remove(0)
}

async fn lease_of_the_email_being_sent(app: &TestApp) -> DateTime<FixedOffset> {
    let emails_being_sent = select_rows(
        app,
        EMAIL_OUTBOX_TABLE,
        &["lease_expires_at"],
        &[("status", EMAIL_SENDING_STATUS.into())],
    )
    .await;
    assert_eq!(emails_being_sent.len(), 1, "expected one email being sent");
    emails_being_sent[0]
        .try_get("", "lease_expires_at")
        .expect("read the lease")
}

fn silent_delivery_deadline() -> std::time::Duration {
    std::time::Duration::from_secs(SILENT_MAIL_SERVER_DELIVERY_DEADLINE_SECONDS)
}

async fn update_queued_email(app: &TestApp, recipient: &str, changes: &[(&str, sea_orm::Value)]) {
    let updated = update_rows(
        app,
        EMAIL_OUTBOX_TABLE,
        changes,
        &[("recipient", recipient.into())],
    )
    .await;
    assert_eq!(updated, 1);
}

#[tokio::test]
async fn an_invitation_and_its_email_are_saved_together() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("together");

    let invitation_id = invite_by_email(&owner, tenant_id, &recipient).await;

    let queued_email = the_queued_email_to(&app, &recipient).await;
    assert_eq!(queued_email.kind, INVITATION_EMAIL_KIND);
    assert_eq!(queued_email.status, EMAIL_PENDING_STATUS);
    assert_eq!(
        select_uuid(
            &app,
            INVITATION_TABLE,
            "latest_email_id",
            &[("id", invitation_id.into())]
        )
        .await,
        queued_email.id
    );

    let unsendable_recipient = "bob(smith)@example.test";
    let refused = owner
        .session
        .post(
            &format!("/tenant/{tenant_id}/invitation"),
            json!({ "email": unsendable_recipient, "role_name": MEMBER_ROLE }),
        )
        .await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{}", refused.body);
    assert_eq!(
        count_rows(
            &app,
            INVITATION_TABLE,
            &[("email", unsendable_recipient.into())]
        )
        .await,
        0
    );
    assert_eq!(
        count_rows(
            &app,
            EMAIL_OUTBOX_TABLE,
            &[("recipient", unsendable_recipient.into())]
        )
        .await,
        0
    );
}

#[tokio::test]
async fn a_queued_email_is_sent_and_its_bodies_cleared() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("sent");
    invite_by_email(&owner, tenant_id, &recipient).await;

    app.deliver_due_emails().await;

    let received = shared_mail_server().await.emails_to(&recipient).await;
    assert_eq!(received.len(), 1);
    assert!(received[0].text.contains(INVITATION_LINK_PATH));
    assert!(received[0].html.contains(INVITATION_LINK_PATH));
    let queued_email = the_queued_email_to(&app, &recipient).await;
    assert_eq!(queued_email.status, EMAIL_SENT_STATUS);
    assert_eq!(queued_email.attempt_count, 1);
    assert!(queued_email.sent_at.is_some());
    assert_eq!(queued_email.last_error, None);
    assert_eq!(queued_email.encrypted_text_body, None);
    assert_eq!(queued_email.encrypted_html_body, None);
}

#[tokio::test]
async fn queued_bodies_are_encrypted() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("encrypted");
    invite_by_email(&owner, tenant_id, &recipient).await;

    let queued_email = the_queued_email_to(&app, &recipient).await;
    for stored_body in [
        queued_email
            .encrypted_text_body
            .expect("a queued text body"),
        queued_email
            .encrypted_html_body
            .expect("a queued HTML body"),
    ] {
        assert!(!stored_body.contains(INVITATION_LINK_PATH));
        assert!(!stored_body.contains("invited you"));
    }
}

#[tokio::test]
async fn an_unreachable_mail_server_is_retried_on_the_backoff_schedule() {
    let app = TestApp::spawn_with_unreachable_mail_server().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("retried");
    invite_by_email(&owner, tenant_id, &recipient).await;

    let first_attempt_at = now();
    app.deliver_due_emails().await;
    let after_first_attempt = the_queued_email_to(&app, &recipient).await;
    assert_eq!(after_first_attempt.status, EMAIL_PENDING_STATUS);
    assert_eq!(after_first_attempt.attempt_count, 1);
    assert!(after_first_attempt.last_error.is_some());
    let first_delay = after_first_attempt.next_attempt_at - first_attempt_at;
    assert!(
        first_delay > Duration::seconds(50) && first_delay < Duration::seconds(70),
        "the first retry waits about a minute, not {first_delay}"
    );

    update_queued_email(
        &app,
        &recipient,
        &[("next_attempt_at", (now() - Duration::seconds(1)).into())],
    )
    .await;
    let second_attempt_at = now();
    app.deliver_due_emails().await;
    let after_second_attempt = the_queued_email_to(&app, &recipient).await;
    assert_eq!(after_second_attempt.status, EMAIL_PENDING_STATUS);
    assert_eq!(after_second_attempt.attempt_count, 2);
    let second_delay = after_second_attempt.next_attempt_at - second_attempt_at;
    assert!(
        second_delay > Duration::minutes(4) && second_delay < Duration::minutes(6),
        "the second retry waits about five minutes, not {second_delay}"
    );
}

#[tokio::test]
async fn an_email_whose_links_expired_fails_as_expired() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("expired");
    invite_by_email(&owner, tenant_id, &recipient).await;
    update_queued_email(
        &app,
        &recipient,
        &[("send_before", (now() - Duration::minutes(1)).into())],
    )
    .await;

    app.deliver_due_emails().await;

    let queued_email = the_queued_email_to(&app, &recipient).await;
    assert_eq!(queued_email.status, EMAIL_FAILED_STATUS);
    assert_eq!(
        queued_email.last_error.as_deref(),
        Some(EXPIRED_EMAIL_MESSAGE)
    );
    assert_eq!(queued_email.encrypted_text_body, None);
    assert!(
        shared_mail_server()
            .await
            .emails_to(&recipient)
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn a_refused_recipient_fails_at_once_with_the_server_reply() {
    let app = TestApp::spawn_with_refusing_mail_server().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("refused");
    invite_by_email(&owner, tenant_id, &recipient).await;

    app.deliver_due_emails().await;

    let queued_email = the_queued_email_to(&app, &recipient).await;
    assert_eq!(queued_email.status, EMAIL_FAILED_STATUS);
    assert_eq!(queued_email.attempt_count, 1);
    let server_reply = queued_email.last_error.expect("the server's reply");
    assert!(server_reply.contains("550"), "{server_reply}");
    assert!(queued_email.encrypted_text_body.is_some());
}

#[tokio::test]
async fn two_senders_never_send_the_same_email() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("once");
    invite_by_email(&owner, tenant_id, &recipient).await;

    let (first_sender, second_sender) = tokio::join!(
        sopho::deliver_due_emails(&app.app_state),
        sopho::deliver_due_emails(&app.app_state)
    );
    first_sender.expect("the first sender runs");
    second_sender.expect("the second sender runs");

    assert_eq!(
        shared_mail_server().await.emails_to(&recipient).await.len(),
        1
    );
    let queued_email = the_queued_email_to(&app, &recipient).await;
    assert_eq!(queued_email.status, EMAIL_SENT_STATUS);
    assert_eq!(queued_email.attempt_count, 1);
}

#[tokio::test]
async fn an_email_whose_lease_ran_out_is_sent_again() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("lease");
    invite_by_email(&owner, tenant_id, &recipient).await;
    update_queued_email(
        &app,
        &recipient,
        &[
            ("status", EMAIL_SENDING_STATUS.into()),
            ("attempt_count", 1.into()),
            ("lease_expires_at", (now() - Duration::minutes(1)).into()),
        ],
    )
    .await;

    app.deliver_due_emails().await;

    let queued_email = the_queued_email_to(&app, &recipient).await;
    assert_eq!(queued_email.status, EMAIL_SENT_STATUS);
    assert_eq!(queued_email.attempt_count, 2);
    assert_eq!(
        shared_mail_server().await.emails_to(&recipient).await.len(),
        1
    );
}

#[tokio::test]
async fn a_name_with_a_line_break_cannot_add_a_header() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let smuggled_recipient = unique_recipient("smuggled");
    let updated = update_rows(
        &app,
        "user",
        &[(
            "full_name",
            format!("Mallory\r\nBcc: {smuggled_recipient}").into(),
        )],
        &[("id", owner.user.id.into())],
    )
    .await;
    assert_eq!(updated, 1);
    let recipient = unique_recipient("headers");
    invite_by_email(&owner, tenant_id, &recipient).await;

    app.deliver_due_emails().await;

    let mail_server = shared_mail_server().await;
    let received = mail_server.emails_to(&recipient).await;
    assert_eq!(received.len(), 1);
    let subject = &received[0].subject;
    assert!(!subject.contains(['\r', '\n']), "{subject:?}");
    assert!(
        subject.ends_with("invited you to acme on Sopho"),
        "{subject:?}"
    );
    assert!(mail_server.emails_to(&smuggled_recipient).await.is_empty());
}

#[tokio::test]
async fn a_mail_server_that_stops_answering_times_out_as_a_temporary_failure() {
    let silent_mail_server = SilentMailServer::start().await;
    let app = TestApp::spawn_with_silent_mail_server(&silent_mail_server).await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("silent");
    invite_by_email(&owner, tenant_id, &recipient).await;

    tokio::time::timeout(silent_delivery_deadline(), app.deliver_due_emails())
        .await
        .expect("the sender hung on the silent mail server");

    let queued_email = the_queued_email_to(&app, &recipient).await;
    assert_eq!(queued_email.status, EMAIL_PENDING_STATUS);
    assert_eq!(queued_email.attempt_count, 1);
    assert_eq!(
        queued_email.last_error.as_deref(),
        Some(SMTP_TIMED_OUT_MESSAGE)
    );
}

#[tokio::test]
async fn each_email_in_a_slow_batch_gets_a_full_lease() {
    let mut silent_mail_server = SilentMailServer::start().await;
    let app = TestApp::spawn_with_silent_mail_server(&silent_mail_server).await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    invite_by_email(&owner, tenant_id, &unique_recipient("first")).await;
    invite_by_email(&owner, tenant_id, &unique_recipient("second")).await;

    let observe_leases = async {
        silent_mail_server.wait_for_connection().await;
        let first_lease = lease_of_the_email_being_sent(&app).await;
        silent_mail_server.wait_for_connection().await;
        let second_lease = lease_of_the_email_being_sent(&app).await;
        (first_lease, second_lease)
    };
    let (delivered, (first_lease, second_lease)) = tokio::join!(
        tokio::time::timeout(silent_delivery_deadline(), app.deliver_due_emails()),
        observe_leases
    );

    delivered.expect("the sender hung on the silent mail server");
    let send_timeout = Duration::from_std(std::time::Duration::from_secs(
        SILENT_MAIL_SERVER_TIMEOUT_SECONDS,
    ))
    .expect("convert the send timeout");
    assert!(
        second_lease - first_lease >= send_timeout,
        "the second lease ({second_lease}) was counted from the start of the batch, like the first ({first_lease})"
    );
}

#[tokio::test]
async fn an_email_that_cannot_be_decrypted_fails_with_the_reason() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let recipient = unique_recipient("undecryptable");
    invite_by_email(&owner, tenant_id, &recipient).await;
    update_queued_email(
        &app,
        &recipient,
        &[("encrypted_text_body", "not encrypted with this key".into())],
    )
    .await;

    app.deliver_due_emails().await;

    let queued_email = the_queued_email_to(&app, &recipient).await;
    assert_eq!(queued_email.status, EMAIL_FAILED_STATUS);
    assert_eq!(
        queued_email.last_error.as_deref(),
        Some(UNDECRYPTABLE_EMAIL_MESSAGE)
    );
    assert_eq!(queued_email.encrypted_text_body, None);
    assert!(
        shared_mail_server()
            .await
            .emails_to(&recipient)
            .await
            .is_empty()
    );
}

#[tokio::test]
async fn log_rows_older_than_ninety_days_are_deleted() {
    let app = TestApp::spawn_with_email().await;
    let (tenant_id, owner) = tenant_with_owner(&app).await;
    let old_recipient = unique_recipient("old");
    let recent_recipient = unique_recipient("recent");
    invite_by_email(&owner, tenant_id, &old_recipient).await;
    invite_by_email(&owner, tenant_id, &recent_recipient).await;
    update_queued_email(
        &app,
        &old_recipient,
        &[("created_at", (now() - Duration::days(91)).into())],
    )
    .await;
    update_queued_email(
        &app,
        &recent_recipient,
        &[("created_at", (now() - Duration::days(89)).into())],
    )
    .await;

    let deleted = sopho::delete_expired_email_log(&app.app_state)
        .await
        .expect("delete the old email log rows");

    assert_eq!(deleted, 1);
    assert!(queued_emails_to(&app, &old_recipient).await.is_empty());
    assert_eq!(queued_emails_to(&app, &recent_recipient).await.len(), 1);
}
