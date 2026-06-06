use crate::ai::agent_utils::{AgentName, ModelClient, ModelRole};
use crate::ai_configuration::dto::{
    AiConfigurationDto, Provider, TestAiConfigurationResponse,
};
use crate::ai_configuration::error::AiConfigurationError;
use crate::ai_configuration::repository;
use crate::common::cryptography_utils::{decrypt, encrypt};
use crate::common::time_utils;
use crate::common::AppState;
use tracing::warn;

pub async fn get_status(
    app_state: &AppState,
) -> Result<AiConfigurationDto, AiConfigurationError> {
    match repository::find(&app_state.database_connection).await? {
        Some(model) => Ok(AiConfigurationDto::from_model(&model)),
        None => Ok(AiConfigurationDto::unconfigured()),
    }
}

pub async fn upsert(
    app_state: &AppState,
    provider: Provider,
    api_key: String,
) -> Result<AiConfigurationDto, AiConfigurationError> {
    if api_key.trim().is_empty() {
        return Err(AiConfigurationError::EmptyApiKey);
    }

    let encryption_key = app_state.config.encryption_key.to_string();
    let encrypted = encrypt(encryption_key, api_key.clone())
        .map_err(|e| AiConfigurationError::Encryption(e.to_string()))?;

    let liveness = run_liveness_ping(provider, &api_key).await;
    let liveness_db = match liveness {
        Ok(()) => "LIVE",
        Err(e) => {
            warn!("liveness ping failed: {e}");
            "FAILED"
        }
    };
    let checked_at = Some(time_utils::now_utc_into());

    let model = repository::upsert(
        &app_state.database_connection,
        provider.as_db_str(),
        &encrypted,
        liveness_db,
        checked_at,
    )
    .await?;

    let new_client = build_client_for_provider(provider, &api_key)
        .map_err(|e| AiConfigurationError::ClientBuild(e.to_string()))?;
    {
        let mut slot = app_state.model_client.write().await;
        *slot = Some(new_client);
    }

    Ok(AiConfigurationDto::from_model(&model))
}

pub async fn delete(app_state: &AppState) -> Result<(), AiConfigurationError> {
    repository::delete(&app_state.database_connection).await?;
    let mut slot = app_state.model_client.write().await;
    *slot = None;
    Ok(())
}

pub async fn test(
    provider: Provider,
    api_key: String,
) -> Result<TestAiConfigurationResponse, AiConfigurationError> {
    if api_key.trim().is_empty() {
        return Err(AiConfigurationError::EmptyApiKey);
    }
    match run_liveness_ping(provider, &api_key).await {
        Ok(()) => Ok(TestAiConfigurationResponse {
            ok: true,
            error: None,
        }),
        Err(e) => Ok(TestAiConfigurationResponse {
            ok: false,
            error: Some(e.to_string()),
        }),
    }
}

pub async fn load_at_startup(app_state: &AppState) -> Result<(), AiConfigurationError> {
    let Some(model) = repository::find(&app_state.database_connection).await? else {
        return Ok(());
    };
    let encryption_key = app_state.config.encryption_key.to_string();
    let api_key = decrypt(encryption_key, model.api_key_encrypted.clone())
        .map_err(|e| AiConfigurationError::Encryption(e.to_string()))?;
    let Some(provider) = Provider::from_db_str(&model.provider) else {
        warn!(
            "ai_configuration row has unknown provider {}, ignoring",
            model.provider
        );
        return Ok(());
    };
    let client = build_client_for_provider(provider, &api_key)
        .map_err(|e| AiConfigurationError::ClientBuild(e.to_string()))?;
    let mut slot = app_state.model_client.write().await;
    *slot = Some(client);
    Ok(())
}

fn build_client_for_provider(
    provider: Provider,
    api_key: &str,
) -> Result<ModelClient, rig::http_client::Error> {
    match provider {
        Provider::Anthropic => ModelClient::anthropic(api_key),
        Provider::Openai => ModelClient::openai(api_key),
    }
}

async fn run_liveness_ping(provider: Provider, api_key: &str) -> anyhow::Result<()> {
    let client = build_client_for_provider(provider, api_key)?;
    let agent = client.build_agent(
        ModelRole::Default,
        AgentName::LivenessCheckAgent,
        "Respond with exactly the word OK and nothing else.",
        0.0,
        5,
    );
    let _ = agent.prompt("ping").await?;
    Ok(())
}
