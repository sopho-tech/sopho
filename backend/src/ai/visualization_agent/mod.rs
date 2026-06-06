mod system_prompt;
mod user_prompt;

use crate::ai::agent_utils::{AgentName, ModelRole};
use crate::ai::dto::{Event, EventChannels, VisualizationRecommendation};
use crate::common::AppState;
use crate::database::service::execute_sql_query;
use crate::entity;
use anyhow::Result;
use tracing::info;

const VISUALIZATION_SAMPLE_ROWS: usize = 5;

pub async fn execute(
    app_state: &AppState,
    connection: &entity::connection::Model,
    question: &str,
    sql: &str,
    channels: &EventChannels,
) -> Result<()> {
    channels.send_sse_only(Event::ExecutingQuery).await;
    let query_result = execute_sql_query(connection, sql).await?;
    channels
        .send_sse_only(Event::ExecutedQuery {
            columns: query_result.columns.clone(),
            data: query_result.data.clone(),
        })
        .await;

    channels
        .send_sse_only(Event::RecommendingVisualization)
        .await;
    let visualization = recommend_visualization(
        app_state,
        question,
        &query_result.columns,
        &query_result.data,
    )
    .await?;
    info!(
        "visualization: {}",
        serde_json::to_string_pretty(&visualization).unwrap_or_default()
    );
    channels
        .send(Event::RecommendedVisualization { visualization })
        .await?;

    Ok(())
}

async fn recommend_visualization(
    app_state: &AppState,
    question: &str,
    columns: &[serde_json::Value],
    data: &[serde_json::Value],
) -> Result<VisualizationRecommendation> {
    let sample_data: Vec<serde_json::Value> = data
        .iter()
        .take(VISUALIZATION_SAMPLE_ROWS)
        .cloned()
        .collect();

    let model_client = app_state.require_model_client().await?;
    let agent = model_client.build_agent(
        ModelRole::Default,
        AgentName::VisualizationRecommendationAgent,
        system_prompt::SystemPrompt::Recommendation.as_str(),
        0.2,
        1024,
    );

    let prompt = user_prompt::UserPrompt::Recommendation {
        question: question.to_string(),
        columns: columns.to_vec(),
        sample_data,
    }
    .render();

    let recommendation: VisualizationRecommendation = agent.prompt_typed(prompt).await?;
    Ok(recommendation)
}
