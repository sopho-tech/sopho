use crate::common::AppState;
use crate::common::EmailConfiguration;
use crate::common::cryptography_utils::{decrypt, encrypt};
use crate::common::time_utils;
use crate::email_delivery::constants::{
    EMAIL_LOG_RETENTION_DAYS, EXPIRED_EMAIL_MESSAGE, EmailStatus, NOTICE_SEND_WINDOW_HOURS,
    RETRY_DELAYS_MINUTES, SENDER_BATCH_SIZE, SENDER_POLL_INTERVAL_SECONDS, SENDING_LEASE_MINUTES,
    UNDECRYPTABLE_EMAIL_MESSAGE,
};
use crate::email_delivery::dto::{
    EmailDeliveryDto, EmailLogEntryDto, EmailLogPageDto, EmailMessage,
};
use crate::email_delivery::error::EmailDeliveryError;
use crate::email_delivery::repository;
pub use crate::email_delivery::transport::EmailTransport;
use crate::email_delivery::transport::{self, SendFailure};
use crate::entity::email_outbox;
use chrono::{Duration, Utc};
use sea_orm::DatabaseTransaction;
use sea_orm::prelude::DateTimeWithTimeZone;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

pub fn build_email_transport(
    configuration: &EmailConfiguration,
) -> Result<EmailTransport, EmailDeliveryError> {
    transport::build_email_transport(configuration).map_err(|error| {
        EmailDeliveryError::Internal(format!("Could not set up the mail server: {error}"))
    })
}

pub fn notice_send_before() -> chrono::DateTime<Utc> {
    Utc::now() + Duration::hours(NOTICE_SEND_WINDOW_HOURS)
}

pub fn public_link(app_state: &AppState, path: &str) -> Result<String, EmailDeliveryError> {
    let email_configuration = app_state
        .config
        .email
        .as_ref()
        .ok_or(EmailDeliveryError::Disabled)?;
    Ok(format!("{}{path}", email_configuration.public_url))
}

pub async fn count_recent_emails_transaction(
    txn: &DatabaseTransaction,
    recipient: &str,
    kinds: Vec<String>,
    since: chrono::DateTime<Utc>,
) -> Result<u64, EmailDeliveryError> {
    Ok(repository::count_emails_to_recipient_since(txn, recipient, kinds, since.into()).await?)
}

pub fn is_email_enabled(app_state: &AppState) -> bool {
    app_state.config.email.is_some()
}

pub async fn enqueue_email_transaction(
    txn: &DatabaseTransaction,
    app_state: &AppState,
    message: EmailMessage,
) -> Result<Uuid, EmailDeliveryError> {
    if !is_email_enabled(app_state) {
        return Err(EmailDeliveryError::Disabled);
    }
    if message.recipient.parse::<lettre::Address>().is_err() {
        return Err(EmailDeliveryError::InvalidRecipient {
            recipient: message.recipient,
        });
    }
    let now: DateTimeWithTimeZone = time_utils::now_utc_into();
    let email = repository::insert_email(
        txn,
        email_outbox::Model {
            id: Uuid::new_v4(),
            kind: message.kind,
            recipient: message.recipient,
            subject: without_line_breaks(&message.subject),
            encrypted_text_body: Some(encrypted_body(app_state, message.text_body)?),
            encrypted_html_body: Some(encrypted_body(app_state, message.html_body)?),
            status: EmailStatus::Pending.as_str().to_string(),
            attempt_count: 0,
            next_attempt_at: now,
            lease_expires_at: None,
            send_before: message.send_before.into(),
            last_error: None,
            sent_at: None,
            tenant_id: message.tenant_id,
            requested_by_user_id: message.requested_by_user_id,
            created_at: now,
            updated_at: now,
        },
    )
    .await?;
    Ok(email.id)
}

pub async fn cancel_email_transaction(
    txn: &DatabaseTransaction,
    email_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    repository::cancel_unsent_email(txn, email_id, time_utils::now_utc_into()).await?;
    Ok(())
}

pub async fn deliver_due_emails(app_state: &AppState) -> Result<(), EmailDeliveryError> {
    let (Some(email_transport), Some(email_configuration)) =
        (&app_state.email_transport, &app_state.config.email)
    else {
        return Ok(());
    };
    let database_connection = &app_state.database_connection;
    let now = Utc::now();
    repository::fail_expired_pending_emails(database_connection, now.into()).await?;
    repository::clear_bodies_of_expired_failed_emails(database_connection, now.into()).await?;

    let due_email_ids =
        repository::find_due_email_ids(database_connection, now.into(), SENDER_BATCH_SIZE).await?;
    for email_id in due_email_ids {
        let claimed_at = Utc::now();
        let lease_expires_at = claimed_at + Duration::minutes(SENDING_LEASE_MINUTES);
        let Some(email) = repository::claim_email(
            database_connection,
            email_id,
            claimed_at.into(),
            lease_expires_at.into(),
        )
        .await?
        else {
            continue;
        };
        deliver_claimed_email(app_state, email_transport, email_configuration, email).await?;
    }
    Ok(())
}

pub fn start_email_sender(app_state: AppState) {
    if !is_email_enabled(&app_state) {
        return;
    }
    tokio::spawn(async move {
        let mut poll_interval =
            tokio::time::interval(std::time::Duration::from_secs(SENDER_POLL_INTERVAL_SECONDS));
        loop {
            poll_interval.tick().await;
            if let Err(error) = deliver_due_emails(&app_state).await {
                tracing::error!("Sending queued emails failed: {error}");
            }
        }
    });
}

pub async fn delete_expired_email_log(app_state: &AppState) -> Result<u64, EmailDeliveryError> {
    let cutoff = Utc::now() - Duration::days(EMAIL_LOG_RETENTION_DAYS);
    Ok(
        repository::delete_emails_created_before(&app_state.database_connection, cutoff.into())
            .await?,
    )
}

pub async fn find_delivery_statuses(
    app_state: &AppState,
    email_ids: Vec<Uuid>,
) -> Result<HashMap<Uuid, EmailDeliveryDto>, EmailDeliveryError> {
    repository::find_emails_by_ids(&app_state.database_connection, email_ids)
        .await?
        .into_iter()
        .map(|email| {
            let status = EmailStatus::from_str(&email.status).map_err(|_| {
                EmailDeliveryError::Internal(format!(
                    "Email {} has an unknown status {}",
                    email.id, email.status
                ))
            })?;
            Ok((
                email.id,
                EmailDeliveryDto {
                    status,
                    last_error: email.last_error,
                },
            ))
        })
        .collect()
}

pub async fn list_email_log(
    app_state: &AppState,
    status: Option<EmailStatus>,
    recipient_search: Option<String>,
    page: u64,
    page_size: u64,
) -> Result<EmailLogPageDto, EmailDeliveryError> {
    let email_page = repository::find_emails_page(
        &app_state.database_connection,
        status.map(|status| status.as_str()),
        recipient_search,
        page.saturating_sub(1),
        page_size,
    )
    .await?;
    let now = Utc::now();
    let emails = email_page
        .emails
        .into_iter()
        .map(|email| {
            let status = EmailStatus::from_str(&email.status).map_err(|_| {
                EmailDeliveryError::Internal(format!(
                    "Email {} has an unknown status {}",
                    email.id, email.status
                ))
            })?;
            Ok(EmailLogEntryDto {
                can_retry: status == EmailStatus::Failed
                    && email.encrypted_text_body.is_some()
                    && email.send_before > now,
                id: email.id,
                kind: email.kind,
                recipient: email.recipient,
                subject: email.subject,
                status,
                attempt_count: email.attempt_count,
                last_error: email.last_error,
                created_at: email.created_at,
                next_attempt_at: email.next_attempt_at,
                sent_at: email.sent_at,
            })
        })
        .collect::<Result<Vec<_>, EmailDeliveryError>>()?;
    Ok(EmailLogPageDto {
        emails,
        page,
        page_size,
        total: email_page.total,
        total_pages: email_page.total.div_ceil(page_size),
    })
}

pub async fn retry_email(app_state: &AppState, email_id: Uuid) -> Result<(), EmailDeliveryError> {
    repository::find_email(&app_state.database_connection, email_id)
        .await?
        .ok_or(EmailDeliveryError::NotFound)?;
    let requeued = repository::requeue_failed_email(
        &app_state.database_connection,
        email_id,
        time_utils::now_utc_into(),
    )
    .await?;
    if requeued == 0 {
        return Err(EmailDeliveryError::NotRetryable);
    }
    Ok(())
}

async fn deliver_claimed_email(
    app_state: &AppState,
    email_transport: &EmailTransport,
    email_configuration: &EmailConfiguration,
    email: email_outbox::Model,
) -> Result<(), EmailDeliveryError> {
    let database_connection = &app_state.database_connection;
    let now = Utc::now();
    if email.send_before <= now {
        repository::mark_email_failed_and_clear_bodies(
            database_connection,
            email.id,
            EXPIRED_EMAIL_MESSAGE,
            now.into(),
        )
        .await?;
        return Ok(());
    }
    let (Some(text_body), Some(html_body)) = (
        decrypted_body(app_state, email.encrypted_text_body),
        decrypted_body(app_state, email.encrypted_html_body),
    ) else {
        repository::mark_email_failed_and_clear_bodies(
            database_connection,
            email.id,
            UNDECRYPTABLE_EMAIL_MESSAGE,
            now.into(),
        )
        .await?;
        return Ok(());
    };
    let message = match transport::build_message(
        &email_configuration.from_address,
        &email.recipient,
        &email.subject,
        text_body,
        html_body,
    ) {
        Ok(message) => message,
        Err(reason) => {
            repository::mark_email_failed(database_connection, email.id, &reason, now.into())
                .await?;
            return Ok(());
        }
    };

    let send_timeout = std::time::Duration::from_secs(email_configuration.smtp_timeout_seconds);
    match transport::send_message(email_transport, message, send_timeout).await {
        Ok(()) => {
            repository::mark_email_sent(database_connection, email.id, Utc::now().into()).await?;
        }
        Err(SendFailure::Permanent { reason }) => {
            repository::mark_email_failed(database_connection, email.id, &reason, now.into())
                .await?;
        }
        Err(SendFailure::Temporary { reason }) => {
            let retry_at = usize::try_from(email.attempt_count - 1)
                .ok()
                .and_then(|attempt_index| RETRY_DELAYS_MINUTES.get(attempt_index))
                .map(|delay_minutes| now + Duration::minutes(*delay_minutes))
                .filter(|retry_at| *retry_at < email.send_before);
            match retry_at {
                Some(retry_at) => {
                    repository::schedule_email_retry(
                        database_connection,
                        email.id,
                        retry_at.into(),
                        &reason,
                        now.into(),
                    )
                    .await?;
                }
                None => {
                    repository::mark_email_failed(
                        database_connection,
                        email.id,
                        &reason,
                        now.into(),
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}

fn encrypted_body(app_state: &AppState, body: String) -> Result<String, EmailDeliveryError> {
    encrypt(app_state.config.encryption_key.to_string(), body)
        .map_err(|error| EmailDeliveryError::Encryption(error.to_string()))
}

fn decrypted_body(app_state: &AppState, encrypted_body: Option<String>) -> Option<String> {
    decrypt(app_state.config.encryption_key.to_string(), encrypted_body?).ok()
}

fn without_line_breaks(header_value: &str) -> String {
    header_value.replace(['\r', '\n'], " ")
}
