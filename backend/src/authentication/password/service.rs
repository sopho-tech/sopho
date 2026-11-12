use crate::authentication::constants::UserStatus;
use crate::authentication::error::CredentialsError;
use crate::authentication::password::constants::{
    COMMON_PASSWORDS, COMMON_PASSWORDS_HEADER_PREFIX, FORGOT_PASSWORD_PATH,
    MAXIMUM_PASSWORD_CHARACTERS, MINIMUM_PASSWORD_CHARACTERS, RESET_EMAILS_PER_HOUR,
    RESET_LINK_EXPIRY_MINUTES, RESET_LINK_PATH, SERVICE_NAME_IN_PASSWORD, SIGN_IN_PAGE_PATH,
};
use crate::authentication::password::dto::{
    ChangePasswordDto, PasswordResetDto, PreparedResetEmail, QueuedAccountEmail,
};
use crate::authentication::password::error::{PasswordFlowError, PasswordRuleError};
use crate::authentication::password::repository;
use crate::authentication::repository as authentication_repository;
use crate::authentication::service as authentication_service;
use crate::authorization::dto::UserId;
use crate::common::AppState;
use crate::common::token_utils;
use crate::email_delivery::dto::EmailMessage;
use crate::email_delivery::service as email_delivery_service;
use crate::email_template::constants::EmailKind;
use crate::email_template::dto::{
    NoPasswordEmail, PasswordChangedEmail, PasswordResetEmail, RenderedEmail,
};
use crate::email_template::service::{EmailTemplate, format_email_time};
use crate::entity::{password_reset_token, user};
use chrono::{DateTime, Duration, Utc};
use sea_orm::{DatabaseTransaction, DbErr, TransactionTrait};
use std::collections::HashSet;
use std::sync::LazyLock;
use tower_cookies::Cookies;
use unicode_normalization::UnicodeNormalization;
use uuid::Uuid;

static COMMON_PASSWORD_SET: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    COMMON_PASSWORDS
        .lines()
        .filter(|line| !line.starts_with(COMMON_PASSWORDS_HEADER_PREFIX))
        .collect()
});

pub fn normalized_password(password: &str) -> String {
    password.nfc().collect()
}

pub fn validate_new_password(password: &str, email: &str) -> Result<String, PasswordRuleError> {
    let normalized = normalized_password(password);
    let character_count = normalized.chars().count();
    if character_count < MINIMUM_PASSWORD_CHARACTERS {
        return Err(PasswordRuleError::TooShort);
    }
    if character_count > MAXIMUM_PASSWORD_CHARACTERS {
        return Err(PasswordRuleError::TooLong);
    }
    let lowercased = normalized.to_lowercase();
    if COMMON_PASSWORD_SET.contains(lowercased.as_str()) {
        return Err(PasswordRuleError::Common);
    }
    if lowercased.contains(SERVICE_NAME_IN_PASSWORD) {
        return Err(PasswordRuleError::ContainsServiceName);
    }
    let lowercased_email = email.trim().to_lowercase();
    if !lowercased_email.is_empty() && lowercased.contains(&lowercased_email) {
        return Err(PasswordRuleError::ContainsEmail);
    }
    Ok(normalized)
}

fn require_password_sign_in(app_state: &AppState) -> Result<(), PasswordFlowError> {
    if !authentication_service::is_password_sign_in_available(app_state) {
        return Err(PasswordFlowError::PasswordSignInOff);
    }
    Ok(())
}

pub fn request_password_reset(app_state: AppState, email: String) -> Result<(), PasswordFlowError> {
    require_password_sign_in(&app_state)?;
    if !email_delivery_service::is_email_enabled(&app_state) {
        return Err(PasswordFlowError::ResetUnavailable);
    }
    tokio::spawn(async move {
        if let Err(error) = queue_reset_email_for_address(&app_state, &email).await {
            tracing::error!("Could not handle a password reset request: {error}");
        }
    });
    Ok(())
}

pub async fn send_password_reset_email(
    app_state: &AppState,
    user: &user::Model,
    requested_by_user_id: Option<Uuid>,
) -> Result<(), PasswordFlowError> {
    if !email_delivery_service::is_email_enabled(app_state) {
        return Err(PasswordFlowError::ResetUnavailable);
    }
    let now = Utc::now();
    let prepared_email = prepare_reset_email(app_state, user, now).await?;

    let txn = app_state.database_connection.begin().await?;
    authentication_repository::lock_user_row(&txn, user.id).await?;
    let recent_reset_emails = email_delivery_service::count_recent_emails_transaction(
        &txn,
        &user.email,
        vec![
            EmailKind::PasswordReset.to_string(),
            EmailKind::NoPassword.to_string(),
        ],
        now - Duration::hours(1),
    )
    .await?;
    if recent_reset_emails >= RESET_EMAILS_PER_HOUR {
        txn.rollback().await?;
        return Err(PasswordFlowError::TooManyResetEmails);
    }
    let has_reset_link = prepared_email.reset_token.is_some();
    if let Some(reset_token) = prepared_email.reset_token {
        repository::insert_reset_token(&txn, reset_token).await?;
    }
    queue_email_transaction(
        &txn,
        app_state,
        QueuedAccountEmail {
            user,
            rendered_email: prepared_email.rendered_email,
            send_before: prepared_email.send_before,
            requested_by_user_id,
        },
    )
    .await?;
    txn.commit().await?;
    if has_reset_link {
        tracing::info!("Password reset email queued for user {}", user.id);
    } else {
        tracing::info!(
            "User {} has no password, so a no-password notice was queued instead of a reset link",
            user.id
        );
    }
    Ok(())
}

async fn prepare_reset_email(
    app_state: &AppState,
    user: &user::Model,
    now: DateTime<Utc>,
) -> Result<PreparedResetEmail, PasswordFlowError> {
    match user.password_hash {
        Some(_) => {
            let link_token = token_utils::new_link_token();
            let expires_at = now + Duration::minutes(RESET_LINK_EXPIRY_MINUTES);
            Ok(PreparedResetEmail {
                rendered_email: PasswordResetEmail {
                    full_name: user.full_name.clone(),
                    reset_link: email_delivery_service::public_link(
                        app_state,
                        &format!("{RESET_LINK_PATH}{link_token}"),
                    )?,
                }
                .render()?,
                send_before: expires_at,
                reset_token: Some(password_reset_token::Model {
                    id: Uuid::new_v4(),
                    user_id: user.id,
                    token_hash: token_utils::hash_token(&link_token),
                    expires_at: expires_at.into(),
                    used_at: None,
                    created_at: now.into(),
                    updated_at: now.into(),
                }),
            })
        }
        None => Ok(PreparedResetEmail {
            rendered_email: NoPasswordEmail {
                full_name: user.full_name.clone(),
                provider_names:
                    crate::authentication::oidc::service::connected_provider_display_names(
                        app_state, user.id,
                    )
                    .await?,
                sign_in_link: email_delivery_service::public_link(app_state, SIGN_IN_PAGE_PATH)?,
            }
            .render()?,
            send_before: email_delivery_service::notice_send_before(),
            reset_token: None,
        }),
    }
}

pub async fn reset_password(
    app_state: &AppState,
    reset: PasswordResetDto,
) -> Result<(), PasswordFlowError> {
    require_password_sign_in(app_state)?;
    let reset_token = repository::find_unused_reset_token_by_hash(
        &app_state.database_connection,
        &token_utils::hash_token(&reset.token),
    )
    .await?
    .ok_or(PasswordFlowError::InvalidResetLink)?;
    let now = Utc::now();
    if reset_token.expires_at <= now {
        return Err(PasswordFlowError::InvalidResetLink);
    }
    let user =
        authentication_repository::get_user(&app_state.database_connection, reset_token.user_id)
            .await?
            .ok_or(PasswordFlowError::InvalidResetLink)?;
    if user.status != UserStatus::Active.as_str() {
        return Err(PasswordFlowError::InvalidResetLink);
    }
    let normalized_new_password = validate_new_password(&reset.new_password, &user.email)?;
    let password_hash = hashed_password(&normalized_new_password)?;
    let password_changed_notice = password_changed_email(app_state, &user, now)?;

    let txn = app_state.database_connection.begin().await?;
    if repository::use_up_reset_token(&txn, reset_token.id, now.into()).await? == 0 {
        return Err(PasswordFlowError::InvalidResetLink);
    }
    repository::use_up_reset_tokens_for_user(&txn, user.id, now.into()).await?;
    authentication_repository::update_password_hash(&txn, user.id, password_hash, now.into())
        .await?;
    authentication_repository::set_email_verified_if_missing(&txn, user.id, now.into()).await?;
    authentication_repository::clear_failed_sign_ins(&txn, user.id, now.into()).await?;
    authentication_repository::deactivate_sessions_for_user(&txn, user.id, None, now.into())
        .await?;
    if let Some(rendered_email) = password_changed_notice {
        queue_email_transaction(
            &txn,
            app_state,
            QueuedAccountEmail {
                user: &user,
                rendered_email,
                send_before: email_delivery_service::notice_send_before(),
                requested_by_user_id: None,
            },
        )
        .await?;
    }
    txn.commit().await?;
    Ok(())
}

pub async fn change_password(
    app_state: &AppState,
    cookies: &Cookies,
    user_id: UserId,
    change: ChangePasswordDto,
) -> Result<(), PasswordFlowError> {
    require_password_sign_in(app_state)?;
    let user = authentication_service::find_user_by_id(app_state, user_id).await?;
    if user.password_hash.is_none() {
        return Err(PasswordFlowError::NoPassword);
    }
    match authentication_service::verify_credentials(
        app_state,
        &user.email,
        &change.current_password,
    )
    .await
    {
        Ok(_) => {}
        Err(CredentialsError::InvalidCredentials) => {
            return Err(PasswordFlowError::WrongCurrentPassword);
        }
        Err(credentials_error) => return Err(PasswordFlowError::Credentials(credentials_error)),
    }
    let normalized_new_password = validate_new_password(&change.new_password, &user.email)?;
    let password_hash = hashed_password(&normalized_new_password)?;
    let current_session_id = authentication_service::current_session_id(cookies, app_state)
        .await
        .map_err(|_| DbErr::RecordNotFound("The current session was not found".to_string()))?;
    let now = Utc::now();
    let password_changed_notice = password_changed_email(app_state, &user, now)?;

    let txn = app_state.database_connection.begin().await?;
    authentication_repository::update_password_hash(&txn, user.id, password_hash, now.into())
        .await?;
    repository::use_up_reset_tokens_for_user(&txn, user.id, now.into()).await?;
    authentication_repository::deactivate_sessions_for_user(
        &txn,
        user.id,
        Some(current_session_id),
        now.into(),
    )
    .await?;
    if let Some(rendered_email) = password_changed_notice {
        queue_email_transaction(
            &txn,
            app_state,
            QueuedAccountEmail {
                user: &user,
                rendered_email,
                send_before: email_delivery_service::notice_send_before(),
                requested_by_user_id: Some(user.id),
            },
        )
        .await?;
    }
    txn.commit().await?;
    Ok(())
}

async fn queue_reset_email_for_address(
    app_state: &AppState,
    email: &str,
) -> Result<(), PasswordFlowError> {
    let Some(user) = authentication_service::find_optional_user_by_email(app_state, email).await?
    else {
        tracing::info!("Password reset requested for an email no account uses; no email sent");
        return Ok(());
    };
    if user.status != UserStatus::Active.as_str() {
        tracing::info!(
            "Password reset requested for user {} whose status is {}; no email sent",
            user.id,
            user.status
        );
        return Ok(());
    }
    match send_password_reset_email(app_state, &user, None).await {
        Ok(()) => Ok(()),
        Err(PasswordFlowError::TooManyResetEmails) => {
            tracing::info!(
                "Password reset requested for user {} who reached the hourly limit; no email sent",
                user.id
            );
            Ok(())
        }
        Err(other_error) => Err(other_error),
    }
}

fn hashed_password(normalized_new_password: &str) -> Result<String, PasswordFlowError> {
    authentication_service::hash_password(normalized_new_password)
        .map_err(|error| PasswordFlowError::Hashing(error.to_string()))
}

fn password_changed_email(
    app_state: &AppState,
    user: &user::Model,
    changed_at: DateTime<Utc>,
) -> Result<Option<RenderedEmail>, PasswordFlowError> {
    if !email_delivery_service::is_email_enabled(app_state) {
        tracing::info!(
            "Email is not configured, so user {} gets no password-changed notice",
            user.id
        );
        return Ok(None);
    }
    Ok(Some(
        PasswordChangedEmail {
            full_name: user.full_name.clone(),
            changed_at: format_email_time(changed_at),
            forgot_password_link: email_delivery_service::public_link(
                app_state,
                FORGOT_PASSWORD_PATH,
            )?,
        }
        .render()?,
    ))
}

async fn queue_email_transaction(
    txn: &DatabaseTransaction,
    app_state: &AppState,
    queued_email: QueuedAccountEmail<'_>,
) -> Result<Uuid, PasswordFlowError> {
    Ok(email_delivery_service::enqueue_email_transaction(
        txn,
        app_state,
        EmailMessage {
            kind: queued_email.rendered_email.kind.to_string(),
            recipient: queued_email.user.email.clone(),
            subject: queued_email.rendered_email.subject,
            text_body: queued_email.rendered_email.text_body,
            html_body: queued_email.rendered_email.html_body,
            send_before: queued_email.send_before,
            tenant_id: None,
            requested_by_user_id: queued_email.requested_by_user_id,
        },
    )
    .await?)
}

#[cfg(test)]
mod tests {
    use super::{normalized_password, validate_new_password};
    use crate::authentication::password::error::PasswordRuleError;

    const EMAIL: &str = "ada@example.com";

    #[test]
    fn passwords_shorter_than_fifteen_characters_are_refused() {
        assert_eq!(
            validate_new_password("fourteen chars", EMAIL),
            Err(PasswordRuleError::TooShort)
        );
        assert!(validate_new_password("fifteen chars!!", EMAIL).is_ok());
        assert_eq!(
            validate_new_password("éééééééééééééé", EMAIL),
            Err(PasswordRuleError::TooShort),
            "length counts characters, not bytes"
        );
    }

    #[test]
    fn passwords_longer_than_one_hundred_twenty_eight_are_refused() {
        assert!(
            validate_new_password(
                &"long walk "
                    .repeat(12)
                    .chars()
                    .take(128)
                    .collect::<String>(),
                EMAIL
            )
            .is_ok()
        );
        assert_eq!(
            validate_new_password(&"x".repeat(129), EMAIL),
            Err(PasswordRuleError::TooLong)
        );
    }

    #[test]
    fn common_passwords_are_refused_whatever_their_case() {
        assert_eq!(
            validate_new_password("PasswordPassword", EMAIL),
            Err(PasswordRuleError::Common)
        );
        assert_eq!(
            validate_new_password("qwertyuiopasdfgh", EMAIL),
            Err(PasswordRuleError::Common)
        );
    }

    #[test]
    fn passwords_containing_sopho_or_the_email_are_refused() {
        assert_eq!(
            validate_new_password("my Sopho dashboards rock", EMAIL),
            Err(PasswordRuleError::ContainsServiceName)
        );
        assert_eq!(
            validate_new_password("x ADA@example.com x", EMAIL),
            Err(PasswordRuleError::ContainsEmail)
        );
    }

    #[test]
    fn passwords_are_compared_after_unicode_normalization() {
        let composed = "café au lait every morning";
        let decomposed = "cafe\u{301} au lait every morning";
        assert_ne!(composed, decomposed);
        assert_eq!(
            normalized_password(composed),
            normalized_password(decomposed)
        );
        assert_eq!(validate_new_password(decomposed, EMAIL).unwrap(), composed);
    }
}
