use crate::authentication::constants::{
    CookieName, SessionStatus, ACCESS_TOKEN_EXPIRY_HOURS, REFRESH_TOKEN_EXPIRY_DAYS,
};
use crate::authentication::dto;
use crate::authentication::repository;
use crate::authentication::repository::{get_user, get_user_by_email};
use crate::common::error_codes::codes;
use crate::common::time_utils;
use crate::common::AppState;
use crate::entity;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, ParamsBuilder, Version,
};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use getrandom;
use sea_orm::DbErr;
use time::OffsetDateTime;
use tower_cookies::cookie::Cookie;
use tower_cookies::cookie::SameSite;
use tower_cookies::Cookies;
use uuid::Uuid;

/// Result type for access token validation
#[derive(Debug)]
#[allow(dead_code)] // Missing variant is handled in middleware but kept for completeness
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

/// Validates an access token and returns the session if valid
pub async fn validate_access_token(
    access_token: &str,
    app_state: &AppState,
) -> Result<entity::session::Model, AccessTokenValidationError> {
    let session = repository::get_active_session_by_access_token(
        &app_state.database_connection,
        access_token.to_string(),
    )
    .await
    .map_err(|_| AccessTokenValidationError::Invalid)?;

    // Check if access token has expired
    let access_token_expires_at = session.access_token_expires_at.with_timezone(&Utc);
    if time_utils::is_expired(&access_token_expires_at) {
        return Err(AccessTokenValidationError::Expired);
    }

    Ok(session)
}

/// Extracts and validates access token from cookies
/// Returns the session if valid, or an error response tuple
async fn extract_and_validate_session(
    cookies: &Cookies,
    app_state: &AppState,
) -> Result<entity::session::Model, (StatusCode, Json<serde_json::Value>)> {
    let access_token = match cookies.get(&CookieName::AccessToken.to_string()) {
        Some(token) => token.value().to_string(),
        None => {
            tracing::error!("No access token found");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "message": "Unauthorized"
                })),
            ));
        }
    };

    let session = match repository::get_active_session_by_access_token(
        &app_state.database_connection,
        access_token,
    )
    .await
    {
        Ok(sess) => sess,
        Err(e) => {
            tracing::error!("Failed to get session: {:?}", e);
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "message": "Unauthorized"
                })),
            ));
        }
    };

    // Check if refresh token has expired
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
                "message": "Access token expired",
                "error_code": codes::ACCESS_TOKEN_EXPIRED.as_str()
            })),
        );
    }

    let session_dto = dto::SessionDto {
        id: session.id,
        user_id: session.user_id,
        refresh_token: session.refresh_token,
        access_token: session.access_token,
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

    let user = match get_user(&app_state.database_connection, session.user_id).await {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to get user: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "message": "Failed to retrieve user"
                })),
            );
        }
    };

    let user_dto = dto::UserDto {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
    };

    (StatusCode::OK, Json(serde_json::json!(user_dto)))
}

pub async fn create_session(
    app_state: AppState,
    payload: dto::CreateSessionDto,
    cookies: Cookies,
) -> impl IntoResponse {
    let user = match get_user_by_email(&app_state.database_connection, payload.email.clone()).await
    {
        Ok(u) => u,
        Err(e) => {
            tracing::error!("Failed to get user by email: {:?}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "message": "Invalid credentials"
                })),
            );
        }
    };

    if verify_password(&payload.password, &user.password_hash).is_err() {
        tracing::error!("Invalid password for user: {}", payload.email);
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "message": "Invalid credentials"
            })),
        );
    }

    let session_dto = match create_new_session(&app_state, user.id).await {
        Ok(session) => session,
        Err(e) => {
            tracing::error!("Failed to create session: {:?}", e);
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
        .domain(app_state.config.cookie_domain.to_string())
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
    // Create a new session in the database
    let session = repository::create_session(
        &app_state.database_connection,
        entity::session::Model {
            id: Uuid::new_v4(),
            user_id,
            refresh_token: generate_refresh_token(),
            access_token: generate_access_token(),
            refresh_token_expires_at: time_utils::now_plus_days_into(REFRESH_TOKEN_EXPIRY_DAYS),
            access_token_expires_at: time_utils::now_plus_hours_into(ACCESS_TOKEN_EXPIRY_HOURS),
            status: SessionStatus::Active.as_str().to_string(),
            created_at: time_utils::now_utc_into(),
            updated_at: time_utils::now_utc_into(),
        },
    )
    .await?;

    // Return session data to client
    Ok(dto::SessionDto {
        id: session.id,
        user_id: session.user_id,
        refresh_token: session.refresh_token,
        access_token: session.access_token,
        refresh_token_expires_at: session.refresh_token_expires_at.with_timezone(&Utc),
        access_token_expires_at: session.access_token_expires_at.with_timezone(&Utc),
        status: session.status,
        created_at: session.created_at.with_timezone(&Utc),
        updated_at: session.updated_at.with_timezone(&Utc),
    })
}

fn generate_refresh_token() -> String {
    // Generate a cryptographically secure random token
    // Refresh tokens should be longer as they have longer lifetimes
    let mut buffer = [0u8; 64]; // 512 bits
    getrandom::fill(&mut buffer).expect("Failed to generate random bytes");
    STANDARD.encode(&buffer)
}

fn generate_access_token() -> String {
    // Generate a cryptographically secure random token
    // Access tokens can be shorter as they have shorter lifetimes
    let mut buffer = [0u8; 32]; // 256 bits
    getrandom::fill(&mut buffer).expect("Failed to generate random bytes");
    STANDARD.encode(&buffer)
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
        access_token,
    )
    .await
    {
        Ok(sess) => sess,
        Err(e) => {
            tracing::warn!("Failed to get session for signout: {:?}", e);
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
        Ok(_) => {
            tracing::info!("Session {} deactivated successfully", session.id);
        }
        Err(e) => {
            tracing::error!("Failed to deactivate session: {:?}", e);
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
        refresh_token,
    )
    .await
    {
        Ok(sess) => sess,
        Err(e) => {
            tracing::error!("Failed to get session by refresh token: {:?}", e);
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "message": "Unauthorized",
                    "error_code": codes::INVALID_REFRESH_TOKEN.as_str()
                })),
            );
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

    // Create new session
    let session_dto = match create_new_session(&app_state, old_session.user_id).await {
        Ok(session) => session,
        Err(e) => {
            tracing::error!("Failed to create new session: {:?}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "message": "Failed to refresh token"
                })),
            );
        }
    };

    // Set cookies with new tokens (both access and refresh tokens)
    set_session_cookies(&cookies, &session_dto, &app_state);

    (StatusCode::OK, Json(serde_json::json!(session_dto)))
}

fn invalidate_session_cookie(cookies: &Cookies, name: CookieName, app_state: &AppState) {
    let cookie = Cookie::build((name.to_string(), ""))
        .same_site(SameSite::Strict)
        .domain(app_state.config.cookie_domain.to_string())
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

pub async fn seed_admin_user(app_state: &AppState) {
    let username = app_state.config.admin_username.as_ref();
    let password = app_state.config.admin_password.as_ref();
    let email = app_state.config.admin_email.as_ref();
    let full_name = app_state.config.admin_full_name.as_ref();

    if get_user_by_email(&app_state.database_connection, email.to_string())
        .await
        .is_ok()
    {
        tracing::info!(
            "Admin user with email {} already exists, skipping seed",
            email
        );
        return;
    }

    let password_hash = match hash_password(password.as_ref()) {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("Admin seeding failed: could not hash password: {:?}", e);
            return;
        }
    };

    let user = entity::user::Model {
        id: Uuid::new_v4(),
        username: username.to_string(),
        password_hash,
        email: email.to_string(),
        full_name: full_name.to_string(),
        created_at: time_utils::now_utc_into(),
        updated_at: time_utils::now_utc_into(),
    };

    match repository::save_user(&app_state.database_connection, user).await {
        Ok(_) => tracing::info!("Admin user {} created successfully", username),
        Err(e) => tracing::error!("Admin seeding failed: could not save user: {:?}", e),
    }
}
