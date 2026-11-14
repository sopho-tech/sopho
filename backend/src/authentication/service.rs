pub use crate::authentication::constants::UserStatus;
use crate::authentication::constants::{
    ACCESS_TOKEN_EXPIRY_HOURS, ACCOUNT_DEACTIVATED_MESSAGE, CookieName, REFRESH_TOKEN_EXPIRY_DAYS,
    SIGN_IN_BLOCK_MINUTES, SIGN_IN_FAILURE_LIMIT, SessionStatus, TOO_MANY_SIGN_IN_ATTEMPTS_MESSAGE,
};
use crate::authentication::constants::{
    INVALID_CREDENTIALS_MESSAGE, PASSWORD_ADMINISTRATOR_NOT_CONFIGURED_MESSAGE,
    PASSWORD_SIGN_IN_OFF_MESSAGE, SESSION_NOT_FOUND_MESSAGE, USER_NOT_FOUND_MESSAGE,
};
use crate::authentication::dto;
use crate::authentication::error::{AccountCreationError, CredentialsError};
pub use crate::authentication::password::error::{PasswordFlowError, PasswordRuleError};
pub use crate::authentication::password::service::send_password_reset_email;
pub use crate::authentication::password::service::{normalized_password, validate_new_password};
use crate::authentication::repository;
pub use crate::authentication::repository::UserPage;
use crate::authentication::repository::{get_user, get_user_by_email};
use crate::authorization::constants::PlatformRole;
use crate::authorization::dto::{AuthenticatedUser, UserId};
use crate::common::AppState;
use crate::common::PasswordAdministratorConfiguration;
use crate::common::error_codes::{ErrorCode, codes};
use crate::common::time_utils;
use crate::common::token_utils;
use crate::entity;
use crate::feature_flag::constants::FeatureFlag;
use crate::feature_flag::service::is_feature_enabled;
use argon2::{
    Algorithm, Argon2, ParamsBuilder, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chrono::{Duration, Utc};
use getrandom;
use sea_orm::DbErr;
use time::OffsetDateTime;
use tower_cookies::Cookies;
use tower_cookies::cookie::Cookie;
use tower_cookies::cookie::SameSite;
use uuid::Uuid;

#[derive(Debug)]
pub enum AccessTokenValidationError {
    Missing,
    Invalid,
    Expired,
}

/// Checks if a session's refresh token has expired
/// Returns true if the refresh token has expired, false otherwise
/// Note: This function only checks expiry status and does not modify the database.
/// Expired sessions remain in the database to avoid high write load.
fn is_refresh_token_expired(session: &entity::session::Model) -> bool {
    let refresh_token_expires_at = session.refresh_token_expires_at.with_timezone(&Utc);
    if time_utils::is_expired(&refresh_token_expires_at) {
        tracing::warn!("Refresh token expired for session: {}", session.id);
        return true;
    }
    false
}

async fn find_session_by_access_token_cookie(
    cookies: &Cookies,
    app_state: &AppState,
) -> Result<entity::session::Model, AccessTokenValidationError> {
    let access_token = match cookies.get(&CookieName::AccessToken.to_string()) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            tracing::error!("No access token found");
            return Err(AccessTokenValidationError::Missing);
        }
    };

    repository::get_active_session_by_access_token(
        &app_state.database_connection,
        token_utils::hash_token(&access_token),
    )
    .await
    .map_err(|error| {
        tracing::error!("Failed to get session: {}", error);
        AccessTokenValidationError::Invalid
    })?
    .ok_or(AccessTokenValidationError::Invalid)
    .inspect(|session| crate::request_trace::service::record_user_id(session.user_id))
}

pub async fn authenticate_from_cookies(
    cookies: &Cookies,
    app_state: &AppState,
) -> Result<AuthenticatedUser, AccessTokenValidationError> {
    let session = find_session_by_access_token_cookie(cookies, app_state).await?;

    let access_token_expires_at = session.access_token_expires_at.with_timezone(&Utc);
    if time_utils::is_expired(&access_token_expires_at) {
        return Err(AccessTokenValidationError::Expired);
    }

    Ok(AuthenticatedUser::from_validated_session(session.user_id))
}

async fn extract_and_validate_session(
    cookies: &Cookies,
    app_state: &AppState,
) -> Result<entity::session::Model, (StatusCode, Json<serde_json::Value>)> {
    let session = find_session_by_access_token_cookie(cookies, app_state)
        .await
        .map_err(|_| {
            (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "message": "Unauthorized"
                })),
            )
        })?;

    if is_refresh_token_expired(&session) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "message": "Session expired",
                "error_code": codes::REFRESH_TOKEN_EXPIRED.as_str()
            })),
        ));
    }

    Ok(session)
}

pub async fn get_session(cookies: Cookies, app_state: AppState) -> impl IntoResponse {
    let session = match extract_and_validate_session(&cookies, &app_state).await {
        Ok(sess) => sess,
        Err(err_response) => return err_response,
    };

    // Check if access token has expired
    let access_token_expires_at = session.access_token_expires_at.with_timezone(&Utc);
    if time_utils::is_expired(&access_token_expires_at) {
        tracing::warn!("Access token expired for session: {}", session.id);
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "status": StatusCode::UNAUTHORIZED.to_string(),
                "message": "Access token expired",
                "code": codes::ACCESS_TOKEN_EXPIRED.as_str()
            })),
        );
    }

    let session_dto = dto::SessionDto {
        id: session.id,
        user_id: session.user_id,
        refresh_token: String::new(),
        access_token: String::new(),
        refresh_token_expires_at: session.refresh_token_expires_at.with_timezone(&Utc),
        access_token_expires_at: session.access_token_expires_at.with_timezone(&Utc),
        status: session.status,
        created_at: session.created_at.with_timezone(&Utc),
        updated_at: session.updated_at.with_timezone(&Utc),
    };
    (StatusCode::OK, Json(serde_json::json!(session_dto)))
}

pub async fn get_current_user(cookies: Cookies, app_state: AppState) -> impl IntoResponse {
    let session = match extract_and_validate_session(&cookies, &app_state).await {
        Ok(sess) => sess,
        Err(err_response) => return err_response,
    };

    match current_user_profile(
        &app_state,
        AuthenticatedUser::from_validated_session(session.user_id).user_id(),
    )
    .await
    {
        Ok(user_dto) => (StatusCode::OK, Json(serde_json::json!(user_dto))),
        Err(error) => {
            tracing::error!("Failed to get user: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "message": "Failed to retrieve user"
                })),
            )
        }
    }
}

pub async fn find_optional_user_by_email(
    app_state: &AppState,
    email: &str,
) -> Result<Option<entity::user::Model>, DbErr> {
    get_user_by_email(&app_state.database_connection, normalize_email(email)).await
}

pub async fn find_users_by_ids(
    app_state: &AppState,
    user_ids: Vec<Uuid>,
) -> Result<Vec<entity::user::Model>, DbErr> {
    repository::get_users_by_ids(&app_state.database_connection, user_ids).await
}

pub async fn find_user_by_id(
    app_state: &AppState,
    user_id: UserId,
) -> Result<entity::user::Model, DbErr> {
    get_user(&app_state.database_connection, user_id.get_uuid())
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(USER_NOT_FOUND_MESSAGE.into()))
}

pub fn platform_role_of(user: &entity::user::Model) -> anyhow::Result<PlatformRole> {
    PlatformRole::from_str(&user.platform_role).map_err(anyhow::Error::msg)
}

pub async fn find_platform_role(
    app_state: &AppState,
    user_id: UserId,
) -> anyhow::Result<PlatformRole> {
    platform_role_of(&find_user_by_id(app_state, user_id).await?)
}

pub async fn current_user_profile(
    app_state: &AppState,
    user_id: UserId,
) -> anyhow::Result<dto::UserDto> {
    let user = find_user_by_id(app_state, user_id).await?;
    let platform_permissions = platform_role_of(&user)?
        .permissions()
        .into_iter()
        .map(|permission| permission.as_str().to_string())
        .collect();
    Ok(dto::UserDto {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        platform_permissions,
    })
}

pub async fn start_session_for_user(
    app_state: &AppState,
    cookies: &Cookies,
    user_id: Uuid,
) -> Result<(), DbErr> {
    let session_dto = create_new_session(app_state, user_id).await?;
    set_session_cookies(cookies, &session_dto, app_state);
    Ok(())
}

pub async fn create_sso_user_transaction(
    txn: &sea_orm::DatabaseTransaction,
    email: &str,
    full_name: &str,
) -> Result<entity::user::Model, DbErr> {
    let email = normalize_email(email);
    repository::save_user(
        txn,
        entity::user::Model {
            id: Uuid::new_v4(),
            username: email.clone(),
            password_hash: None,
            email,
            full_name: full_name.to_string(),
            platform_role: PlatformRole::Member.as_str().to_string(),
            email_verified_at: Some(time_utils::now_utc_into()),
            status: UserStatus::Active.as_str().to_string(),
            failed_sign_in_count: 0,
            sign_in_blocked_until: None,
            created_at: time_utils::now_utc_into(),
            updated_at: time_utils::now_utc_into(),
        },
    )
    .await
}

pub async fn create_password_user_transaction(
    txn: &sea_orm::DatabaseTransaction,
    email: &str,
    full_name: &str,
    normalized_new_password: &str,
) -> Result<entity::user::Model, AccountCreationError> {
    let email = normalize_email(email);
    let password_hash = hash_password(normalized_new_password)
        .map_err(|error| AccountCreationError::PasswordHashing(error.to_string()))?;
    Ok(repository::save_user(
        txn,
        entity::user::Model {
            id: Uuid::new_v4(),
            username: email.clone(),
            password_hash: Some(password_hash),
            email,
            full_name: full_name.to_string(),
            platform_role: PlatformRole::Member.as_str().to_string(),
            email_verified_at: Some(time_utils::now_utc_into()),
            status: UserStatus::Active.as_str().to_string(),
            failed_sign_in_count: 0,
            sign_in_blocked_until: None,
            created_at: time_utils::now_utc_into(),
            updated_at: time_utils::now_utc_into(),
        },
    )
    .await?)
}

pub async fn mark_email_verified_transaction(
    txn: &sea_orm::DatabaseTransaction,
    user_id: Uuid,
) -> Result<(), DbErr> {
    repository::set_email_verified_if_missing(txn, user_id, time_utils::now_utc_into()).await?;
    Ok(())
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub async fn verify_credentials(
    app_state: &AppState,
    email: &str,
    password: &str,
) -> Result<entity::user::Model, CredentialsError> {
    let Some(user) =
        get_user_by_email(&app_state.database_connection, normalize_email(email)).await?
    else {
        tracing::info!("Sign-in refused: no account uses the given email");
        return Err(CredentialsError::InvalidCredentials);
    };
    let now = Utc::now();
    if user
        .sign_in_blocked_until
        .is_some_and(|blocked_until| blocked_until > now)
    {
        tracing::info!(
            "Sign-in refused for user {}: blocked after too many failed attempts",
            user.id
        );
        return Err(CredentialsError::TooManyAttempts);
    }
    let Some(password_hash) = user.password_hash.as_deref() else {
        tracing::info!(
            "Sign-in refused for user {}: the account has no password",
            user.id
        );
        return Err(CredentialsError::InvalidCredentials);
    };
    if verify_password(&normalized_password(password), password_hash).is_err() {
        tracing::info!("Sign-in refused for user {}: wrong password", user.id);
        repository::increment_failed_sign_in_count(
            &app_state.database_connection,
            user.id,
            now.into(),
        )
        .await?;
        repository::block_sign_in_after_failures(
            &app_state.database_connection,
            user.id,
            SIGN_IN_FAILURE_LIMIT,
            (now + Duration::minutes(SIGN_IN_BLOCK_MINUTES)).into(),
        )
        .await?;
        return Err(CredentialsError::InvalidCredentials);
    }
    if user.status == UserStatus::Deactivated.as_str() {
        tracing::info!(
            "Sign-in refused for user {}: the account is deactivated",
            user.id
        );
        return Err(CredentialsError::AccountDeactivated);
    }
    if user.failed_sign_in_count > 0 || user.sign_in_blocked_until.is_some() {
        repository::clear_failed_sign_ins(&app_state.database_connection, user.id, now.into())
            .await?;
    }
    Ok(user)
}

pub fn is_password_sign_in_available(app_state: &AppState) -> bool {
    is_feature_enabled(app_state, FeatureFlag::PasswordSignIn)
}

pub fn sign_in_page_options(app_state: &AppState) -> dto::SignInPageOptionsDto {
    let password_sign_in_available = is_password_sign_in_available(app_state);
    dto::SignInPageOptionsDto {
        providers: crate::authentication::oidc::service::list_providers(app_state),
        password_sign_in_available,
        password_reset_available: password_sign_in_available
            && crate::email_delivery::service::is_email_enabled(app_state),
    }
}

pub async fn find_optional_user(
    app_state: &AppState,
    user_id: Uuid,
) -> Result<Option<entity::user::Model>, DbErr> {
    get_user(&app_state.database_connection, user_id).await
}

pub async fn list_users_page(
    app_state: &AppState,
    search: Option<String>,
    page_index: u64,
    page_size: u64,
) -> Result<UserPage, DbErr> {
    repository::find_users_page(
        &app_state.database_connection,
        search,
        page_index,
        page_size,
    )
    .await
}

pub async fn count_active_platform_administrators(app_state: &AppState) -> Result<u64, DbErr> {
    repository::count_active_users_with_platform_role(
        &app_state.database_connection,
        PlatformRole::Administrator.as_str(),
    )
    .await
}

pub async fn deactivate_user_transaction(
    txn: &sea_orm::DatabaseTransaction,
    user_id: Uuid,
) -> Result<(), DbErr> {
    let now = time_utils::now_utc_into();
    repository::set_user_status(txn, user_id, UserStatus::Deactivated, now).await?;
    repository::deactivate_sessions_for_user(txn, user_id, None, now).await?;
    Ok(())
}

pub async fn set_platform_role(
    app_state: &AppState,
    user_id: Uuid,
    platform_role: PlatformRole,
) -> Result<(), DbErr> {
    repository::set_user_platform_role(
        &app_state.database_connection,
        user_id,
        platform_role.as_str(),
        time_utils::now_utc_into(),
    )
    .await?;
    Ok(())
}

pub async fn reactivate_user(app_state: &AppState, user_id: Uuid) -> Result<(), DbErr> {
    repository::set_user_status(
        &app_state.database_connection,
        user_id,
        UserStatus::Active,
        time_utils::now_utc_into(),
    )
    .await?;
    Ok(())
}

pub async fn current_session_id(
    cookies: &Cookies,
    app_state: &AppState,
) -> Result<Uuid, AccessTokenValidationError> {
    Ok(find_session_by_access_token_cookie(cookies, app_state)
        .await?
        .id)
}

pub async fn create_session(
    app_state: AppState,
    payload: dto::CreateSessionDto,
    cookies: Cookies,
) -> impl IntoResponse {
    if !is_password_sign_in_available(&app_state) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({
                "message": PASSWORD_SIGN_IN_OFF_MESSAGE,
                "error_code": ErrorCode::PasswordSignInOff.as_str()
            })),
        );
    }
    let user = match verify_credentials(&app_state, &payload.email, &payload.password).await {
        Ok(user) => user,
        Err(CredentialsError::InvalidCredentials) => {
            tracing::warn!("Rejected sign-in: the email or password is wrong");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "message": INVALID_CREDENTIALS_MESSAGE
                })),
            );
        }
        Err(CredentialsError::TooManyAttempts) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "message": TOO_MANY_SIGN_IN_ATTEMPTS_MESSAGE,
                    "error_code": ErrorCode::TooManySignInAttempts.as_str()
                })),
            );
        }
        Err(CredentialsError::AccountDeactivated) => {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({
                    "message": ACCOUNT_DEACTIVATED_MESSAGE,
                    "error_code": ErrorCode::AccountDeactivated.as_str()
                })),
            );
        }
        Err(CredentialsError::Database(error)) => {
            tracing::error!("Failed to look up user for sign-in: {}", error);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "message": "Failed to sign in"
                })),
            );
        }
    };

    let session_dto = match create_new_session(&app_state, user.id).await {
        Ok(session) => session,
        Err(e) => {
            tracing::error!("Failed to create session: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "message": "Failed to create session"
                })),
            );
        }
    };

    // Set cookies
    set_session_cookies(&cookies, &session_dto, &app_state);

    (StatusCode::OK, Json(serde_json::json!(session_dto)))
}

fn create_argon2id() -> Result<Argon2<'static>, argon2::Error> {
    let params = ParamsBuilder::new()
        // Memory cost is expressed in KiB; 19 MiB = 19 * 1024 KiB.
        .m_cost(19 * 1024)
        // Iteration count.
        .t_cost(2)
        // Degree of parallelism.
        .p_cost(1)
        .build()?;

    Ok(Argon2::new(Algorithm::Argon2id, Version::default(), params))
}

fn verify_password(
    password: &str,
    stored_password_hash: &str,
) -> Result<(), argon2::password_hash::Error> {
    // Create Argon2 instance
    let argon2 = create_argon2id().map_err(|_| argon2::password_hash::Error::Crypto)?;

    // Parse the stored password hash
    let parsed_hash = PasswordHash::new(stored_password_hash)?;

    // Verify the password against the stored hash
    argon2.verify_password(password.as_bytes(), &parsed_hash)
}

fn add_session_cookie(
    cookies: &Cookies,
    name: CookieName,
    value: &str,
    expiration: OffsetDateTime,
    app_state: &AppState,
) {
    let cookie = Cookie::build((name, value.to_string()))
        .same_site(SameSite::Strict)
        .expires(expiration)
        .path("/")
        .secure(app_state.config.cookie_secure)
        .http_only(true)
        .build();
    cookies.add(cookie);
}

/// Sets both access and refresh token cookies from a session DTO
/// Consolidates the duplicated cookie-setting logic
fn set_session_cookies(cookies: &Cookies, session_dto: &dto::SessionDto, app_state: &AppState) {
    // Convert DateTime<Utc> to OffsetDateTime for cookies
    let access_token_expires =
        time_utils::datetime_to_offset_datetime(&session_dto.access_token_expires_at);
    let refresh_token_expires =
        time_utils::datetime_to_offset_datetime(&session_dto.refresh_token_expires_at);

    // Set cookies
    add_session_cookie(
        cookies,
        CookieName::AccessToken,
        &session_dto.access_token,
        access_token_expires,
        app_state,
    );
    add_session_cookie(
        cookies,
        CookieName::RefreshToken,
        &session_dto.refresh_token,
        refresh_token_expires,
        app_state,
    );
}

async fn create_new_session(app_state: &AppState, user_id: Uuid) -> Result<dto::SessionDto, DbErr> {
    let refresh_token = generate_refresh_token();
    let access_token = generate_access_token();
    let session = repository::create_session(
        &app_state.database_connection,
        entity::session::Model {
            id: Uuid::new_v4(),
            user_id,
            refresh_token_hash: token_utils::hash_token(&refresh_token),
            access_token_hash: token_utils::hash_token(&access_token),
            refresh_token_expires_at: time_utils::now_plus_days_into(REFRESH_TOKEN_EXPIRY_DAYS),
            access_token_expires_at: time_utils::now_plus_hours_into(ACCESS_TOKEN_EXPIRY_HOURS),
            status: SessionStatus::Active.as_str().to_string(),
            created_at: time_utils::now_utc_into(),
            updated_at: time_utils::now_utc_into(),
        },
    )
    .await?;

    Ok(build_session_dto(session, access_token, refresh_token))
}

fn build_session_dto(
    session: entity::session::Model,
    access_token: String,
    refresh_token: String,
) -> dto::SessionDto {
    dto::SessionDto {
        id: session.id,
        user_id: session.user_id,
        refresh_token,
        access_token,
        refresh_token_expires_at: session.refresh_token_expires_at.with_timezone(&Utc),
        access_token_expires_at: session.access_token_expires_at.with_timezone(&Utc),
        status: session.status,
        created_at: session.created_at.with_timezone(&Utc),
        updated_at: session.updated_at.with_timezone(&Utc),
    }
}

fn generate_refresh_token() -> String {
    // Generate a cryptographically secure random token
    // Refresh tokens should be longer as they have longer lifetimes
    let mut buffer = [0u8; 64]; // 512 bits
    getrandom::fill(&mut buffer).expect("Failed to generate random bytes");
    STANDARD.encode(buffer)
}

fn generate_access_token() -> String {
    // Generate a cryptographically secure random token
    // Access tokens can be shorter as they have shorter lifetimes
    let mut buffer = [0u8; 32]; // 256 bits
    getrandom::fill(&mut buffer).expect("Failed to generate random bytes");
    STANDARD.encode(buffer)
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let argon2 = create_argon2id().map_err(|_| argon2::password_hash::Error::Crypto)?;
    // Generate a random salt; it will be embedded in the resulting PHC string.
    let salt = SaltString::generate(&mut OsRng);

    // Hash the incoming password using Argon2id and return the PHC string.
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

pub async fn signout(cookies: Cookies, app_state: AppState) -> impl IntoResponse {
    let access_token = match cookies.get(&CookieName::AccessToken.to_string()) {
        Some(token) => token.value().to_string(),
        None => {
            tracing::warn!("No access token found for signout");
            // Still invalidate cookies even if no token found
            invalidate_session_cookies(&cookies, &app_state);
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "message": "Signed out successfully"
                })),
            );
        }
    };

    // Get the session by access token
    let session = match repository::get_active_session_by_access_token(
        &app_state.database_connection,
        token_utils::hash_token(&access_token),
    )
    .await
    .map(|session| session.ok_or(DbErr::RecordNotFound(SESSION_NOT_FOUND_MESSAGE.into())))
    .and_then(|session| session)
    {
        Ok(sess) => sess,
        Err(e) => {
            tracing::warn!("Failed to get session for signout: {}", e);
            // Still invalidate cookies even if session not found
            invalidate_session_cookies(&cookies, &app_state);
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "message": "Signed out successfully"
                })),
            );
        }
    };

    // Mark session as inactive
    match repository::deactivate_session(&app_state.database_connection, session.id).await {
        Ok(Some(_)) => {
            tracing::info!("Session {} deactivated successfully", session.id);
        }
        Ok(None) => {
            tracing::warn!("Session {} was already gone when signing out", session.id);
        }
        Err(e) => {
            tracing::error!("Failed to deactivate session: {}", e);
            // Still invalidate cookies even if database update fails
        }
    }

    // Invalidate cookies
    invalidate_session_cookies(&cookies, &app_state);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Signed out successfully"
        })),
    )
}

pub async fn refresh_token(cookies: Cookies, app_state: AppState) -> impl IntoResponse {
    let refresh_token = match cookies.get(&CookieName::RefreshToken.to_string()) {
        Some(token) => token.value().to_string(),
        None => {
            tracing::error!("No refresh token found");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "message": "Unauthorized"
                })),
            );
        }
    };

    let old_session = match repository::get_active_session_by_refresh_token(
        &app_state.database_connection,
        token_utils::hash_token(&refresh_token),
    )
    .await
    .map(|session| session.ok_or(DbErr::RecordNotFound(SESSION_NOT_FOUND_MESSAGE.into())))
    .and_then(|session| session)
    {
        Ok(sess) => sess,
        Err(e) => {
            tracing::error!("Failed to get session by refresh token: {}", e);
            return invalid_refresh_token_response();
        }
    };

    // Check if refresh token has expired
    if is_refresh_token_expired(&old_session) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "message": "Refresh token expired",
                "error_code": codes::REFRESH_TOKEN_EXPIRED.as_str()
            })),
        );
    }

    let session_dto = match rotate_session_tokens(&app_state, old_session).await {
        Ok(Some(session_dto)) => session_dto,
        Ok(None) => {
            tracing::warn!("Refresh token was already used to rotate its session");
            return invalid_refresh_token_response();
        }
        Err(e) => {
            tracing::error!("Failed to rotate session tokens: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "message": "Failed to refresh token"
                })),
            );
        }
    };

    set_session_cookies(&cookies, &session_dto, &app_state);

    (StatusCode::OK, Json(serde_json::json!(session_dto)))
}

fn invalid_refresh_token_response() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "message": "Unauthorized",
            "error_code": codes::INVALID_REFRESH_TOKEN.as_str()
        })),
    )
}

async fn rotate_session_tokens(
    app_state: &AppState,
    session: entity::session::Model,
) -> Result<Option<dto::SessionDto>, DbErr> {
    let refresh_token = generate_refresh_token();
    let access_token = generate_access_token();
    let presented_refresh_token_hash = session.refresh_token_hash.clone();
    let rotated_session = repository::rotate_session_tokens(
        &app_state.database_connection,
        entity::session::Model {
            refresh_token_hash: token_utils::hash_token(&refresh_token),
            access_token_hash: token_utils::hash_token(&access_token),
            refresh_token_expires_at: time_utils::now_plus_days_into(REFRESH_TOKEN_EXPIRY_DAYS),
            access_token_expires_at: time_utils::now_plus_hours_into(ACCESS_TOKEN_EXPIRY_HOURS),
            updated_at: time_utils::now_utc_into(),
            ..session
        },
        presented_refresh_token_hash,
    )
    .await?;
    Ok(rotated_session.map(|session| build_session_dto(session, access_token, refresh_token)))
}

fn invalidate_session_cookie(cookies: &Cookies, name: CookieName, app_state: &AppState) {
    let cookie = Cookie::build((name.to_string(), ""))
        .same_site(SameSite::Strict)
        .max_age(time_utils::zero_duration_seconds())
        .path("/")
        .secure(app_state.config.cookie_secure)
        .http_only(true)
        .build();
    cookies.add(cookie);
}

fn invalidate_session_cookies(cookies: &Cookies, app_state: &AppState) {
    invalidate_session_cookie(cookies, CookieName::AccessToken, app_state);
    invalidate_session_cookie(cookies, CookieName::RefreshToken, app_state);
}

pub async fn seed_admin_user(app_state: &AppState) -> anyhow::Result<()> {
    if crate::configuration::service::is_admin_user_setup_done(app_state).await {
        return Ok(());
    }

    let password_administrator = password_administrator_configuration(app_state)?;
    let username = password_administrator.username.as_str();
    let password = password_administrator.password.as_str();
    let email = normalize_email(&password_administrator.email);
    let full_name = password_administrator.full_name.as_str();

    if get_user_by_email(&app_state.database_connection, email.clone())
        .await?
        .is_some()
    {
        tracing::info!(
            "Admin user with email {} already exists, skipping seed",
            email
        );
        crate::configuration::service::mark_admin_user_setup_done(app_state).await;
        return Ok(());
    }

    let password_hash = hash_password(&normalized_password(password))
        .map_err(|error| anyhow::anyhow!("could not hash the admin password: {error:?}"))?;

    let user = entity::user::Model {
        id: Uuid::new_v4(),
        username: username.to_string(),
        password_hash: Some(password_hash),
        email,
        full_name: full_name.to_string(),
        platform_role: PlatformRole::Administrator.as_str().to_string(),
        email_verified_at: None,
        status: UserStatus::Active.as_str().to_string(),
        failed_sign_in_count: 0,
        sign_in_blocked_until: None,
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };

    repository::save_user(&app_state.database_connection, user).await?;
    tracing::info!("Admin user {} created successfully", username);
    crate::configuration::service::mark_admin_user_setup_done(app_state).await;
    Ok(())
}

fn password_administrator_configuration(
    app_state: &AppState,
) -> anyhow::Result<&PasswordAdministratorConfiguration> {
    app_state
        .config
        .password_administrator
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!(PASSWORD_ADMINISTRATOR_NOT_CONFIGURED_MESSAGE))
}

pub async fn find_seeded_password_administrator(
    app_state: &AppState,
) -> anyhow::Result<entity::user::Model> {
    let password_administrator = password_administrator_configuration(app_state)?;
    Ok(find_user_by_email(app_state, &password_administrator.email).await?)
}

pub async fn find_user_by_email(
    app_state: &AppState,
    email: &str,
) -> Result<entity::user::Model, DbErr> {
    get_user_by_email(&app_state.database_connection, normalize_email(email))
        .await?
        .ok_or_else(|| DbErr::RecordNotFound(USER_NOT_FOUND_MESSAGE.into()))
}
