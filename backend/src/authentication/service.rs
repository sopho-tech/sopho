use crate::common::AppState;
use crate::authentication::dto;
use crate::authentication::repository;
use axum::response::IntoResponse;
use axum::http::StatusCode;
use axum::Json;
use uuid::Uuid;
use reqwest;
use chrono::Utc;
use crate::entity;
use sea_orm::DbErr;
use getrandom;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use crate::authentication::dto::GoogleUserInfo;
use tower_cookies::Cookies;
use tower_cookies::cookie::Cookie;
use crate::authentication::constants::CookieName;
use tower_cookies::cookie::SameSite;
use time::OffsetDateTime;
use time::Duration;


pub async fn get_session(
    cookies: Cookies,
    app_state: AppState,
) -> impl IntoResponse {
    tracing::debug!("cookies: {:?}", cookies);
    let access_token = cookies.get(&CookieName::AccessToken.to_string());
    let session;
    match access_token {
        Some(access_token) => {
            match repository::get_session_by_access_token(&app_state.database_connection, access_token.value().to_string()).await {
                Ok(sess) => session = Some(sess),
                Err(e) => {
                    tracing::error!("Failed to get session: {:?}", e);
                    return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                        "message": "Unauthorized"
                    })));
                }
            }
        }
        None => {
            tracing::error!("No access token found");
            return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                "message": "Unauthorized"
            })));
        }
    }
    match session {
        Some(session) => {
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
        },
        None => {
            tracing::error!("Failed to get session");
            (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                "message": "Unauthorized"
            })))
        }
    }
}


pub async fn create_session(
    app_state: AppState,
    payload: dto::CreateSessionDto,
    cookies: Cookies,
) -> impl IntoResponse {
    tracing::debug!("payload: {:?}", payload);
    let response = match payload.provider.as_str() {
        "google" => {
            let response = handle_google_auth(app_state.clone(), payload.code).await;
            response
        } ,
        _ => {
            tracing::error!("Unknown provider");
            Err(format!("Unknown provider"))
        }
    };
    
    match response {
        Ok(user) => {
            match create_new_session(&app_state, user.id).await {
                Ok(session) => {
                    let expiration = OffsetDateTime::now_utc() + Duration::days(7);
                    
                    add_session_cookie(
                        &cookies, 
                        CookieName::AccessToken, 
                        &session.access_token, 
                        expiration,
                        &app_state
                    );
                    
                    add_session_cookie(
                        &cookies, 
                        CookieName::RefreshToken, 
                        &session.refresh_token, 
                        expiration,
                        &app_state
                    );
                    
                    (StatusCode::OK, Json(serde_json::json!(session)))
                },
                Err(e) => {
                    tracing::error!("Failed to create session: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                        "message": "Failed to create session"
                    })))
                }
            }
        },
        Err(error_message) => {
            tracing::error!("Authentication failed: {}", error_message);
            (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
                "message": error_message
            })))
        }
    }
}


fn add_session_cookie(
    cookies: &Cookies, 
    name: CookieName, 
    value: &str, 
    expiration: OffsetDateTime,
    app_state: &AppState
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

async fn handle_google_auth(app_state: AppState, code: String) -> Result<entity::user::Model, String> {
    tracing::debug!("Google provider");
    
    // Exchange authorization code for tokens
    let tokens = match exchange_code_for_tokens(&app_state, code).await {
        Ok(tokens) => tokens,
        Err(response) => return Err(format!("Failed to exchange code for tokens: {:?}", response)),
    };
    
    // Get user profile from Google
    let profile = match get_google_profile(tokens["access_token"].to_string()).await {
        Ok(profile) => profile,
        Err(e) => {
            tracing::error!("Failed to get Google profile: {:?}", e);
            return Err(format!("Failed to get Google profile: {:?}", e));
        }
    };
    
    // Find or create user
    match find_or_create_user(&app_state, &profile).await {
        Ok(user) => {
            Ok(user)
        },
        Err(response) => Err(format!("Failed to find or create user: {:?}", response)),
    }
}

async fn exchange_code_for_tokens(
    app_state: &AppState, 
    code: String
) -> Result<serde_json::Value, (StatusCode, Json<serde_json::Value>)> {
    let client = &app_state.client;
    let token_response = client
        .post("https://oauth2.googleapis.com/token")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "code": code,
            "client_id": app_state.config.google_client_id.to_string(),
            "client_secret": app_state.config.google_client_secret.to_string(),
            "redirect_uri": app_state.config.google_redirect_uri.to_string(),
            "grant_type": "authorization_code"
        }))
        .send()
        .await;
    
    tracing::debug!("token_response: {:?}", token_response);
    
    match token_response {
        Ok(response) => {
            if !response.status().is_success() {
                tracing::error!("Token request failed: {:?}", response.status());
                return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({ 
                    "message": "Authentication failed" 
                }))));
            }
            
            match response.json::<serde_json::Value>().await {
                Ok(tokens) => {
                    tracing::debug!("Tokens received: {:?}", tokens);
                    Ok(tokens)
                },
                Err(e) => {
                    tracing::error!("Failed to parse token response: {:?}", e);
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ 
                        "message": "Failed to process authentication" 
                    }))))
                }
            }
        },
        Err(e) => {
            tracing::error!("Error requesting tokens: {:?}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ 
                "message": "Failed to authenticate with provider" 
            }))))
        }
    }
}

async fn find_or_create_user(
    app_state: &AppState,
    profile: &GoogleUserInfo
) -> Result<entity::user::Model, String> {
    // Try to find existing user
    let user_result = repository::get_user_by_email(
        &app_state.database_connection, 
        profile.email.clone()
    ).await;
    
    match user_result {
        Ok(user) => Ok(user),
        Err(e) => {
            tracing::error!("Failed to get user: {:?}", e);
            // User not found, create a new one
            create_new_user(app_state, profile).await
        }
    }
}

async fn create_new_user(
    app_state: &AppState,
    profile: &GoogleUserInfo
) -> Result<entity::user::Model, String> {
    let user = repository::save_user(&app_state.database_connection, entity::user::Model {
        id: Uuid::new_v4(),
        email: profile.email.clone(),
        username: profile.email.clone(),
        full_name: profile.name.clone(),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    }).await;
    
    match user {
        Ok(user) => Ok(user),
        Err(e) => {
            tracing::error!("Failed to save user: {:?}", e);
            Err(format!("Failed to save user: {:?}", e))
        }
    }
}

async fn get_google_profile(access_token: String) -> Result<GoogleUserInfo, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    let profile = response.json::<GoogleUserInfo>().await?;
    Ok(profile)
}

async fn create_new_session(
    app_state: &AppState, 
    user_id: Uuid
) -> Result<dto::SessionDto, DbErr> {
    // Create a new session in the database
    let session = repository::create_session(&app_state.database_connection, entity::session::Model {
        id: Uuid::new_v4(),
        user_id,
        refresh_token: generate_refresh_token(),
        access_token: generate_access_token(),
        refresh_token_expires_at: (Utc::now() + chrono::Duration::days(7)).into(),
        access_token_expires_at: (Utc::now() + chrono::Duration::days(7)).into(),
        status: "active".to_string(),
        created_at: Utc::now().into(),
        updated_at: Utc::now().into(),
    }).await?;
    
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
