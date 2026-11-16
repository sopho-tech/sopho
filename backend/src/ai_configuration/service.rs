use crate::ai::agent_utils::{AgentName, AgentSettings, ModelClient, ModelProvider, ModelRole};
use crate::ai::error::is_usage_recording_failure;
use crate::ai_configuration::constants::{
    AI_NOT_CONFIGURED_MESSAGE, AI_NOT_LIVE_MESSAGE, PLATFORM_AI_ALLOWANCE_USED_MESSAGE,
};
use crate::ai_configuration::datatypes::AiKeyDecision;
use crate::ai_configuration::dto::{
    AiConfigurationDto, AiConfigurationStatus, PlatformAiAllowanceDto, TestAiConfigurationResponse,
};
use crate::ai_configuration::error::{AiAccessError, AiConfigurationError};
use crate::ai_configuration::repository;
use crate::ai_usage::constants::KeySource;
use crate::ai_usage::datatypes::{AiAccess, AiCallContext};
use crate::ai_usage::error::AiUsageError;
use crate::ai_usage::service as ai_usage_service;
use crate::authorization::constants::Permission;
use crate::authorization::dto::{TenantContext, TenantId};
use crate::authorization::service::require_capability;
use crate::common::cryptography_utils::{decrypt, encrypt};
use crate::common::number_utils::format_whole_number;
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

async fn decide_ai_key<'saved_configuration>(
    app_state: &AppState,
    tenant_id: &TenantId,
    saved_configuration: Option<&'saved_configuration entity::ai_configuration::Model>,
) -> Result<AiKeyDecision<'saved_configuration>, AiUsageError> {
    if let Some(saved_configuration) = saved_configuration {
        return Ok(AiKeyDecision::Tenant {
            saved_configuration,
            status: AiConfigurationStatus::from_db_liveness_when_row_exists(
                saved_configuration.liveness_status.as_str(),
            ),
        });
    }
    let Some(platform_ai) = &app_state.config.platform_ai else {
        return Ok(AiKeyDecision::Unconfigured);
    };
    let used_tokens = ai_usage_service::platform_ai_used_tokens(app_state, tenant_id).await?;
    Ok(AiKeyDecision::Platform {
        token_allowance: platform_ai.token_allowance,
        used_tokens,
        is_allowance_used: used_tokens >= platform_ai.token_allowance,
    })
}

async fn status_of_tenant(
    app_state: &AppState,
    tenant_id: &TenantId,
    saved_configuration: Option<&entity::ai_configuration::Model>,
) -> Result<AiConfigurationDto, AiConfigurationError> {
    Ok(
        match decide_ai_key(app_state, tenant_id, saved_configuration).await? {
            AiKeyDecision::Tenant {
                saved_configuration,
                status,
            } => {
                let is_ai_available = status == AiConfigurationStatus::Live;
                AiConfigurationDto {
                    status,
                    provider: ModelProvider::from_db_str(&saved_configuration.provider),
                    last_checked_at: saved_configuration.last_checked_at,
                    is_ai_available,
                    key_source: Some(KeySource::Tenant),
                    unavailable_message: (!is_ai_available).then_some(AI_NOT_LIVE_MESSAGE),
                    platform_ai: None,
                }
            }
            AiKeyDecision::Platform {
                token_allowance,
                used_tokens,
                is_allowance_used,
            } => AiConfigurationDto {
                status: AiConfigurationStatus::NotConfigured,
                provider: None,
                last_checked_at: None,
                is_ai_available: !is_allowance_used,
                key_source: Some(KeySource::Platform),
                unavailable_message: is_allowance_used
                    .then_some(PLATFORM_AI_ALLOWANCE_USED_MESSAGE),
                platform_ai: Some(PlatformAiAllowanceDto {
                    allowance_usage: ai_usage_service::allowance_usage(
                        used_tokens,
                        token_allowance,
                    ),
                    tokens_left_label: format_whole_number(
                        token_allowance.saturating_sub(used_tokens),
                    ),
                    is_allowance_used,
                }),
            },
            AiKeyDecision::Unconfigured => AiConfigurationDto {
                status: AiConfigurationStatus::NotConfigured,
                provider: None,
                last_checked_at: None,
                is_ai_available: false,
                key_source: None,
                unavailable_message: Some(AI_NOT_CONFIGURED_MESSAGE),
                platform_ai: None,
            },
        },
    )
}

pub async fn resolve_ai_access(
    app_state: &AppState,
    tenant_id: &TenantId,
) -> Result<AiAccess, AiAccessError> {
    let saved_configuration = repository::find(&app_state.database_connection, tenant_id).await?;
    match decide_ai_key(app_state, tenant_id, saved_configuration.as_ref()).await? {
        AiKeyDecision::Tenant { status, .. } => {
            if status != AiConfigurationStatus::Live {
                return Err(AiAccessError::NotLive);
            }
            let model_client = app_state.current_model_client(tenant_id).await.ok_or(
                AiAccessError::MissingModelClient {
                    tenant_id: tenant_id.get_uuid(),
                },
            )?;
            Ok(AiAccess {
                model_client,
                key_source: KeySource::Tenant,
            })
        }
        AiKeyDecision::Platform {
            is_allowance_used, ..
        } => {
            if is_allowance_used {
                return Err(AiAccessError::PlatformAiAllowanceUsed);
            }
            let platform_model_client = app_state
                .platform_model_client
                .as_ref()
                .ok_or(AiAccessError::NotConfigured)?;
            Ok(AiAccess {
                model_client: platform_model_client.clone(),
                key_source: KeySource::Platform,
            })
        }
        AiKeyDecision::Unconfigured => Err(AiAccessError::NotConfigured),
    }
}

pub async fn list_tenant_ids_with_saved_key(
    app_state: &AppState,
) -> Result<Vec<uuid::Uuid>, AiConfigurationError> {
    Ok(repository::list_tenant_ids(&app_state.database_connection).await?)
}

pub async fn upsert(
    app_state: &AppState,
    tenant_context: &TenantContext,
    provider: ModelProvider,
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

    let new_client = build_client_for_provider(provider, &api_key, &app_state.config);
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
    provider: ModelProvider,
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
    let Some(provider) = ModelProvider::from_db_str(&model.provider) else {
        warn!(
            "ai_configuration row has unknown provider {}, ignoring",
            model.provider
        );
        return Ok(());
    };
    let client = build_client_for_provider(provider, &api_key, &app_state.config);
    app_state.set_model_client(tenant_id, client).await;
    Ok(())
}

pub fn build_client_for_provider(
    provider: ModelProvider,
    api_key: &str,
    configurations: &Configurations,
) -> ModelClient {
    match provider {
        ModelProvider::Anthropic => {
            ModelClient::anthropic(api_key, configurations.anthropic_base_url.as_deref())
        }
        ModelProvider::Openai => {
            ModelClient::openai(api_key, configurations.openai_base_url.as_deref())
        }
    }
}

async fn run_liveness_ping(
    app_state: &AppState,
    tenant_context: &TenantContext,
    provider: ModelProvider,
    api_key: &str,
) -> anyhow::Result<()> {
    let client = build_client_for_provider(provider, api_key, &app_state.config);
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
