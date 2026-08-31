use super::dto::SuggestedQuestionDto;
use super::repository;
use crate::ai::suggested_questions_agent;
use crate::authorization::constants::Action;
use crate::authorization::dto::{TenantContext, TenantId};
use crate::authorization::errors::AuthorizationError;
use crate::authorization::service as authorization_service;
use crate::common::time_utils;
use crate::common::AppState;
use crate::connection::service as connection_service;
use crate::data_catalog;
use crate::entity;
use crate::suggested_question::errors::SuggestedQuestionError;
use chrono::Utc;
use sea_orm::TransactionTrait;
use uuid::Uuid;

use tracing::{error, info};

const STALE_AFTER_HOURS: i64 = 24;

pub async fn list_for_connection(
    app_state: &AppState,
    tenant_context: &TenantContext,
    connection_id: Uuid,
) -> Result<Vec<SuggestedQuestionDto>, SuggestedQuestionError> {
    let subject = connection_service::load_authorization_subject(
        app_state,
        tenant_context.tenant_id(),
        connection_id,
    )
    .await
    .map_err(|_| SuggestedQuestionError::Unauthorized(AuthorizationError::NotReachable))?;
    authorization_service::require_access(app_state, tenant_context, &subject, Action::Read)
        .await?;

    let rows = repository::find_for_connection(
        &app_state.database_connection,
        tenant_context.tenant_id(),
        connection_id,
    )
    .await?;
    Ok(rows.into_iter().map(SuggestedQuestionDto::from).collect())
}

pub async fn delete_for_connection(
    app_state: &AppState,
    tenant_id: &TenantId,
    connection_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    repository::delete_for_connection(&app_state.database_connection, tenant_id, connection_id)
        .await
}

fn is_stale(generated_at: sea_orm::prelude::DateTimeWithTimeZone) -> bool {
    let now = time_utils::now_utc();
    now.signed_duration_since(generated_at.with_timezone(&Utc))
        > chrono::Duration::hours(STALE_AFTER_HOURS)
}

pub async fn refresh_all_stale(app_state: &AppState, tenant_id: &TenantId) {
    if app_state.current_model_client(tenant_id).await.is_none() {
        info!("suggested_question: AI not configured; skipping refresh");
        return;
    }

    let connections =
        match connection_service::execute_get_all_connections(app_state, tenant_id).await {
            Ok(c) => c,
            Err(e) => {
                error!("suggested_question: failed to load connections: {e}");
                return;
            }
        };

    for connection in connections {
        let latest = repository::latest_generated_at(
            &app_state.database_connection,
            tenant_id,
            connection.id,
        )
        .await;
        let needs_refresh = match latest {
            Ok(Some(ts)) => is_stale(ts),
            Ok(None) => true,
            Err(e) => {
                error!(
                    "suggested_question: staleness check failed for {}: {e}",
                    connection.id
                );
                continue;
            }
        };
        if !needs_refresh {
            continue;
        }

        if let Err(e) = regenerate(app_state, tenant_id, &connection).await {
            error!(
                "suggested_question: regen failed for connection {}: {e}",
                connection.id
            );
        } else {
            info!(
                "suggested_question: regenerated questions for connection {}",
                connection.id
            );
        }
    }
}

async fn regenerate(
    app_state: &AppState,
    tenant_id: &TenantId,
    connection: &entity::connection::Model,
) -> anyhow::Result<()> {
    let catalog = data_catalog::get_data_catalog_of_connection(app_state, connection).await?;
    let questions =
        suggested_questions_agent::suggest_questions(app_state, tenant_id, connection, &catalog)
            .await?;
    if questions.is_empty() {
        anyhow::bail!("no questions generated");
    }
    replace_batch(
        &app_state.database_connection,
        tenant_id,
        connection.id,
        questions,
    )
    .await?;
    Ok(())
}

async fn replace_batch(
    db: &sea_orm::DatabaseConnection,
    tenant_id: &TenantId,
    connection_id: Uuid,
    questions: Vec<String>,
) -> Result<(), sea_orm::DbErr> {
    let now = time_utils::now_utc_into();
    let txn = db.begin().await?;
    repository::delete_for_connection(&txn, tenant_id, connection_id).await?;
    for question_text in questions {
        repository::insert(&txn, tenant_id, connection_id, question_text, now).await?;
    }
    txn.commit().await
}
