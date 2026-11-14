use crate::authorization::dto::Actor;
use crate::authorization::dto::TenantId;
use crate::common::AppState;
use crate::feature_flag::constants::FeatureFlag;
use crate::feature_flag::service::is_feature_enabled;
use crate::request_trace::constants::SUGGESTED_QUESTION_REFRESH_JOB;
use crate::request_trace::service as request_trace_service;
use crate::suggested_question;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{Instrument, error, info};

const SUGGESTED_QUESTION_DAILY_CRON: &str = "0 0 3 * * *";

const SCHEDULER_METADATA_TABLE: &str = "scheduler_job";
const SCHEDULER_NOTIFICATION_TABLE: &str = "scheduler_notification";
const SCHEDULER_NOTIFICATION_STATES_TABLE: &str = "scheduler_notification_state";

pub async fn start(app_state: AppState) {
    let startup_state = app_state.clone();
    tokio::spawn(async move {
        run_startup_suggested_question_refresh(&startup_state).await;
    });

    tokio::spawn(async move {
        if let Err(e) = start_scheduler(app_state).await {
            error!("scheduler failed to start: {e}");
        }
    });
}

async fn start_scheduler(app_state: AppState) -> anyhow::Result<()> {
    let sched = build(&app_state).await?;

    if is_background_suggested_question_refresh_enabled(&app_state) {
        let job_state = app_state.clone();
        let job = Job::new_async(SUGGESTED_QUESTION_DAILY_CRON, move |_uuid, _lock| {
            let job_state = job_state.clone();
            Box::pin(async move {
                refresh_all_stale_for_every_tenant(&job_state).await;
            })
        })?;
        sched.add(job).await?;
    }

    sched.start().await?;
    info!("scheduler started");
    Ok(())
}

async fn build(app_state: &AppState) -> anyhow::Result<JobScheduler> {
    if app_state.config.database_url.starts_with("postgres") {
        build_postgres(app_state).await
    } else {
        Ok(JobScheduler::new().await?)
    }
}

async fn build_postgres(app_state: &AppState) -> anyhow::Result<JobScheduler> {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tokio_cron_scheduler::{
        PostgresMetadataStore, PostgresNotificationStore, PostgresStore, SimpleJobCode,
        SimpleNotificationCode,
    };

    let url = app_state.config.database_url.to_string();

    let metadata_store = PostgresMetadataStore {
        store: Arc::new(RwLock::new(PostgresStore::Created(url.clone()))),
        init_tables: true,
        table: SCHEDULER_METADATA_TABLE.to_string(),
    };

    let notification_store = PostgresNotificationStore {
        store: Arc::new(RwLock::new(PostgresStore::Created(url))),
        init_tables: true,
        table: SCHEDULER_NOTIFICATION_TABLE.to_string(),
        states_table: SCHEDULER_NOTIFICATION_STATES_TABLE.to_string(),
    };

    let sched = JobScheduler::new_with_storage_and_code(
        Box::new(metadata_store),
        Box::new(notification_store),
        Box::new(SimpleJobCode::default()),
        Box::new(SimpleNotificationCode::default()),
        200,
    )
    .await?;
    Ok(sched)
}

fn is_background_suggested_question_refresh_enabled(app_state: &AppState) -> bool {
    is_feature_enabled(app_state, FeatureFlag::BackgroundSuggestedQuestionRefresh)
}

pub(crate) async fn run_startup_suggested_question_refresh(app_state: &AppState) {
    if is_background_suggested_question_refresh_enabled(app_state) {
        refresh_all_stale_for_every_tenant(app_state).await;
    }
}

pub(crate) async fn refresh_all_stale_for_every_tenant(app_state: &AppState) {
    let tenant_ids = match crate::tenant::service::list_active_tenant_ids(app_state).await {
        Ok(tenant_ids) => tenant_ids,
        Err(e) => {
            tracing::error!("scheduler: failed to list tenants: {e}");
            return;
        }
    };
    for tenant_id in tenant_ids {
        let job_span = request_trace_service::background_job_span_for_tenant(
            SUGGESTED_QUESTION_REFRESH_JOB,
            tenant_id,
        );
        let tenant_id = TenantId::from_system_unchecked(tenant_id);
        suggested_question::service::refresh_all_stale(app_state, &tenant_id, &Actor::system())
            .instrument(job_span)
            .await;
    }
}
