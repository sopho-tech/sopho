use crate::ai::agent_utils::{AgentName, ModelClient, ModelRole};
use crate::ai_configuration::dto::{AiConfigurationDto, Provider, TestAiConfigurationResponse};
use crate::ai_configuration::error::AiConfigurationError;
use crate::ai_configuration::repository;
use crate::authorization::constants::Permission;
use crate::authorization::dto::{TenantContext, TenantId};
use crate::authorization::service::require_capability;
use crate::common::cryptography_utils::{decrypt, encrypt};
use crate::common::time_utils;
use crate::common::AppState;
use tracing::warn;

pub async fn get_status(
    app_state: &AppState,
    tenant_context: &TenantContext,
) -> Result<AiConfigurationDto, AiConfigurationError> {
    require_capability(tenant_context, Permission::AiConfigurationRead)?;
    execute_get_status(app_state, tenant_context.tenant_id()).await
}

pub async fn execute_get_status(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> Result<AiConfigurationDto, AiConfigurationError> {
    match repository::find(&app_state.database_connection, tenant_id).await? {
        Some(model) => Ok(AiConfigurationDto::from_model(&model)),
        None => Ok(AiConfigurationDto::unconfigured()),
    }
}

pub async fn upsert(
    app_state: &AppState,
    tenant_context: &TenantContext,
    provider: Provider,
    api_key: String,
) -> Result<AiConfigurationDto, AiConfigurationError> {
    require_capability(tenant_context, Permission::AiConfigurationWrite)?;
    let tenant_id = tenant_context.tenant_id();
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
        tenant_id,
        provider.as_db_str(),
        &encrypted,
        liveness_db,
        checked_at,
    )
    .await?;

    let new_client = build_client_for_provider(provider, &api_key)
        .map_err(|e| AiConfigurationError::ClientBuild(e.to_string()))?;
    app_state.set_model_client(tenant_id, new_client).await;

    Ok(AiConfigurationDto::from_model(&model))
}

pub async fn delete(
    app_state: &AppState,
    tenant_context: &TenantContext,
) -> Result<(), AiConfigurationError> {
    require_capability(tenant_context, Permission::AiConfigurationWrite)?;
    let tenant_id = tenant_context.tenant_id();
    repository::delete(&app_state.database_connection, tenant_id).await?;
    app_state.clear_model_client(tenant_id).await;
    Ok(())
}

pub async fn test(
    tenant_context: &TenantContext,
    provider: Provider,
    api_key: String,
) -> Result<TestAiConfigurationResponse, AiConfigurationError> {
    require_capability(tenant_context, Permission::AiConfigurationWrite)?;
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

pub async fn load_at_startup_for_tenant(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> Result<(), AiConfigurationError> {
    let Some(model) = repository::find(&app_state.database_connection, tenant_id).await? else {
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
    app_state.set_model_client(tenant_id, client).await;
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

pub async fn load_at_startup(app_state: &AppState) -> Result<(), AiConfigurationError> {
    let tenant_ids = crate::tenant::service::list_all_tenant_ids(app_state)
        .await
        .map_err(|e| AiConfigurationError::Database(sea_orm::DbErr::Custom(e.to_string())))?;
    for tenant_id in tenant_ids {
        let tenant_id = TenantId::from_system_unchecked(tenant_id);
        load_at_startup_for_tenant(app_state, &tenant_id).await?;
    }
    Ok(())
}
