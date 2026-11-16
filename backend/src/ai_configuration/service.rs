use crate::ai::agent_utils::{AgentName, AgentSettings, ModelClient, ModelRole};
use crate::ai::error::is_usage_recording_failure;
use crate::ai_configuration::constants::{
    AI_NOT_CONFIGURED_MESSAGE, AI_NOT_LIVE_MESSAGE, PLATFORM_AI_ALLOWANCE_USED_MESSAGE,
};
use crate::ai_configuration::dto::{
    AiConfigurationDto, AiConfigurationStatus, PlatformAiAllowanceDto, Provider,
    TestAiConfigurationResponse,
};
use crate::ai_configuration::error::{AiAccessError, AiConfigurationError};
use crate::ai_configuration::repository;
use crate::ai_usage::constants::KeySource;
use crate::ai_usage::datatypes::{AiAccess, AiCallContext};
use crate::ai_usage::service as ai_usage_service;
use crate::authorization::constants::Permission;
use crate::authorization::dto::{TenantContext, TenantId};
use crate::authorization::service::require_capability;
use crate::common::cryptography_utils::{decrypt, encrypt};
use crate::common::time_utils;
use crate::common::{AppState, Configurations};
use crate::entity;
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
    let saved_configuration = repository::find(&app_state.database_connection, tenant_id).await?;
    status_of_tenant(app_state, tenant_id, saved_configuration.as_ref()).await
}

async fn status_of_tenant(
    app_state: &AppState,
    tenant_id: &TenantId,
    saved_configuration: Option<&entity::ai_configuration::Model>,
) -> Result<AiConfigurationDto, AiConfigurationError> {
    if let Some(saved_configuration) = saved_configuration {
        let status = AiConfigurationStatus::from_db_liveness_when_row_exists(
            saved_configuration.liveness_status.as_str(),
        );
        let is_ai_available = status == AiConfigurationStatus::Live;
        return Ok(AiConfigurationDto {
            status,
            provider: Provider::from_db_str(&saved_configuration.provider),
            last_checked_at: saved_configuration.last_checked_at,
            is_ai_available,
            key_source: Some(KeySource::Tenant),
            unavailable_message: (!is_ai_available).then_some(AI_NOT_LIVE_MESSAGE),
            platform_ai: None,
        });
    }
    let Some(platform_ai) = &app_state.config.platform_ai else {
        return Ok(AiConfigurationDto {
            status: AiConfigurationStatus::NotConfigured,
            provider: None,
            last_checked_at: None,
            is_ai_available: false,
            key_source: None,
            unavailable_message: Some(AI_NOT_CONFIGURED_MESSAGE),
            platform_ai: None,
        });
    };
    let used_tokens = ai_usage_service::platform_ai_used_tokens(app_state, tenant_id).await?;
    let is_allowance_used = used_tokens >= platform_ai.token_allowance;
    Ok(AiConfigurationDto {
        status: AiConfigurationStatus::NotConfigured,
        provider: None,
        last_checked_at: None,
        is_ai_available: !is_allowance_used,
        key_source: Some(KeySource::Platform),
        unavailable_message: is_allowance_used.then_some(PLATFORM_AI_ALLOWANCE_USED_MESSAGE),
        platform_ai: Some(PlatformAiAllowanceDto {
            token_allowance: platform_ai.token_allowance,
            used_tokens,
            remaining_tokens: platform_ai.token_allowance.saturating_sub(used_tokens),
            is_allowance_used,
        }),
    })
}

pub async fn resolve_ai_access(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> Result<AiAccess, AiAccessError> {
    if let Some(saved_configuration) =
        repository::find(&app_state.database_connection, tenant_id).await?
    {
        let status = AiConfigurationStatus::from_db_liveness_when_row_exists(
            saved_configuration.liveness_status.as_str(),
        );
        if status != AiConfigurationStatus::Live {
            return Err(AiAccessError::NotLive);
        }
        let model_client = app_state.current_model_client(tenant_id).await.ok_or(
            AiAccessError::MissingModelClient {
                tenant_id: tenant_id.get_uuid(),
            },
        )?;
        return Ok(AiAccess {
            model_client,
            key_source: KeySource::Tenant,
        });
    }
    let (Some(platform_ai), Some(platform_model_client)) = (
        &app_state.config.platform_ai,
        &app_state.platform_model_client,
    ) else {
        return Err(AiAccessError::NotConfigured);
    };
    let used_tokens = ai_usage_service::platform_ai_used_tokens(app_state, tenant_id).await?;
    if used_tokens >= platform_ai.token_allowance {
        return Err(AiAccessError::PlatformAiAllowanceUsed);
    }
    Ok(AiAccess {
        model_client: platform_model_client.clone(),
        key_source: KeySource::Platform,
    })
}

pub async fn list_tenant_ids_with_saved_key(
    app_state: &AppState,
) -> Result<Vec<uuid::Uuid>, AiConfigurationError> {
    Ok(repository::list_tenant_ids(&app_state.database_connection).await?)
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

    let liveness = run_liveness_ping(app_state, tenant_context, provider, &api_key).await;
    let liveness_db = match liveness {
        Ok(()) => "LIVE",
        Err(error) if is_usage_recording_failure(&error) => {
            return Err(AiConfigurationError::UsageRecording(error.to_string()));
        }
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
        &tenant_context.actor(),
    )
    .await?;

    let new_client = build_client_for_provider(provider, &api_key, &app_state.config)
        .map_err(|e| AiConfigurationError::ClientBuild(e.to_string()))?;
    app_state.set_model_client(tenant_id, new_client).await;

    status_of_tenant(app_state, tenant_id, Some(&model)).await
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

pub async fn verify_provider_api_key(
    app_state: &AppState,
    tenant_context: &TenantContext,
    provider: Provider,
    api_key: String,
) -> Result<TestAiConfigurationResponse, AiConfigurationError> {
    require_capability(tenant_context, Permission::AiConfigurationWrite)?;
    if api_key.trim().is_empty() {
        return Err(AiConfigurationError::EmptyApiKey);
    }
    match run_liveness_ping(app_state, tenant_context, provider, &api_key).await {
        Ok(()) => Ok(TestAiConfigurationResponse {
            ok: true,
            error: None,
        }),
        Err(error) if is_usage_recording_failure(&error) => {
            Err(AiConfigurationError::UsageRecording(error.to_string()))
        }
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
    let client = build_client_for_provider(provider, &api_key, &app_state.config)
        .map_err(|e| AiConfigurationError::ClientBuild(e.to_string()))?;
    app_state.set_model_client(tenant_id, client).await;
    Ok(())
}

pub fn build_client_for_provider(
    provider: Provider,
    api_key: &str,
    configurations: &Configurations,
) -> Result<ModelClient, rig::http_client::Error> {
    match provider {
        Provider::Anthropic => {
            ModelClient::anthropic(api_key, configurations.anthropic_base_url.as_deref())
        }
        Provider::Openai => ModelClient::openai(api_key, configurations.openai_base_url.as_deref()),
    }
}

async fn run_liveness_ping(
    app_state: &AppState,
    tenant_context: &TenantContext,
    provider: Provider,
    api_key: &str,
) -> anyhow::Result<()> {
    let client = build_client_for_provider(provider, api_key, &app_state.config)?;
    let call_context = AiCallContext::for_user(
        tenant_context.tenant_id().clone(),
        tenant_context.user_id(),
        AiAccess {
            model_client: client.clone(),
            key_source: KeySource::Tenant,
        },
    );
    let agent = client.build_agent(
        app_state,
        &call_context,
        AgentSettings {
            role: ModelRole::Default,
            agent_name: AgentName::LivenessCheckAgent,
            system_prompt: "Respond with exactly the word OK and nothing else.",
            temperature: 0.0,
            max_tokens: 5,
        },
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
