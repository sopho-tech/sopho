use crate::ai::error::AgentCallError;
use crate::ai_usage::dto::{AiCallContext, ModelCallRecord, TokenUsage};
use crate::ai_usage::service as ai_usage_service;
use crate::common::AppState;
use rig::agent::PromptResponse;
use rig::client::CompletionClient;
use rig::completion::{CompletionModel, Prompt, PromptError};
use rig::http_client::Error;
use rig::message::Message;
use rig::providers::{anthropic, openai};
use rig::wasm_compat::WasmCompatSend;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

#[derive(Clone, Copy, Debug)]
pub enum ModelRole {
    Default,
    Small,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "snake_case")]
#[expect(
    clippy::enum_variant_names,
    reason = "variant names are the agent names given to rig and shown in logs"
)]
pub enum AgentName {
    CanvasGenerationAgent,
    ChartSummaryAgent,
    ConversationNameAgent,
    DashboardSummaryAgent,
    LivenessCheckAgent,
    DataCatalogDeletionAgent,
    DataCatalogSelectionAgent,
    DataProfilingBeforeAgent,
    FollowupQuestionsAgent,
    FunctionalRoleAnalysisAgent,
    HypothesisGenerationAgent,
    IntegrateCandidatePlanAgent,
    ResultNarrationAgent,
    RouterAgent,
    SchemaLinkingFinalSynthesisAgent,
    SqlGenerationAgent,
    SuggestedQuestionsAgent,
    VisualizationRecommendationAgent,
}

impl AgentName {
    pub fn as_str(self) -> &'static str {
        self.into()
    }
}

pub struct AgentSettings<'a> {
    pub role: ModelRole,
    pub agent_name: AgentName,
    pub system_prompt: &'a str,
    pub temperature: f64,
    pub max_tokens: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum ModelProvider {
    Anthropic,
    Openai,
}

#[derive(Clone)]
pub enum ModelClient {
    Anthropic(rig::client::Client<anthropic::client::AnthropicExt>),
    OpenAI(rig::client::Client<openai::client::OpenAICompletionsExt>),
}

impl ModelClient {
    pub fn anthropic(api_key: &str, base_url: Option<&str>) -> Result<Self, Error> {
        let builder = anthropic::Client::builder().api_key(api_key);
        let builder = match base_url {
            Some(base_url) => builder.base_url(base_url),
            None => builder,
        };
        Ok(Self::Anthropic(builder.build()?))
    }

    pub fn openai(api_key: &str, base_url: Option<&str>) -> Result<Self, Error> {
        let builder = openai::CompletionsClient::builder().api_key(api_key);
        let builder = match base_url {
            Some(base_url) => builder.base_url(base_url),
            None => builder,
        };
        Ok(Self::OpenAI(builder.build()?))
    }

    pub fn build_agent(
        &self,
        app_state: &AppState,
        call_context: &AiCallContext,
        agent_settings: AgentSettings<'_>,
    ) -> Agent {
        let AgentSettings {
            role,
            agent_name,
            system_prompt,
            temperature,
            max_tokens,
        } = agent_settings;
        let model_name = model_name_for(self, role);
        let model_agent = match self {
            Self::Anthropic(client) => ModelAgent::Anthropic(
                client
                    .agent(model_name)
                    .name(agent_name.as_str())
                    .preamble(system_prompt)
                    .temperature(temperature)
                    .max_tokens(max_tokens)
                    .build(),
            ),
            Self::OpenAI(client) => ModelAgent::OpenAI(
                client
                    .agent(model_name)
                    .name(agent_name.as_str())
                    .preamble(system_prompt)
                    .temperature(temperature)
                    .max_tokens(max_tokens)
                    .build(),
            ),
        };
        Agent {
            model_agent,
            usage_recorder: UsageRecorder {
                app_state: app_state.clone(),
                call_context: call_context.clone(),
                agent_name,
                model_provider: self.model_provider(),
                model_name,
            },
        }
    }

    fn model_provider(&self) -> ModelProvider {
        match self {
            Self::Anthropic(_) => ModelProvider::Anthropic,
            Self::OpenAI(_) => ModelProvider::Openai,
        }
    }
}

fn model_name_for(client: &ModelClient, role: ModelRole) -> &'static str {
    match (client, role) {
        (ModelClient::Anthropic(_), ModelRole::Default) => "claude-haiku-4-5",
        (ModelClient::Anthropic(_), ModelRole::Small) => "claude-haiku-4-5",
        (ModelClient::OpenAI(_), ModelRole::Default) => "gpt-4o-mini",
        (ModelClient::OpenAI(_), ModelRole::Small) => "gpt-4o-mini",
    }
}

pub fn normalized_token_usage(
    model_provider: ModelProvider,
    reported_usage: rig::completion::Usage,
) -> TokenUsage {
    let input_tokens = match model_provider {
        ModelProvider::Anthropic => {
            reported_usage.input_tokens + reported_usage.cached_input_tokens
        }
        ModelProvider::Openai => reported_usage.input_tokens,
    };
    TokenUsage {
        input_tokens,
        output_tokens: reported_usage.output_tokens,
        cached_input_tokens: reported_usage.cached_input_tokens,
    }
}

struct UsageRecorder {
    app_state: AppState,
    call_context: AiCallContext,
    agent_name: AgentName,
    model_provider: ModelProvider,
    model_name: &'static str,
}

impl UsageRecorder {
    async fn record(&self, reported_usage: rig::completion::Usage) -> Result<(), AgentCallError> {
        ai_usage_service::record_model_call(
            &self.app_state,
            &self.call_context,
            ModelCallRecord {
                agent_name: self.agent_name.as_str(),
                provider: self.model_provider.into(),
                model: self.model_name,
                token_usage: normalized_token_usage(self.model_provider, reported_usage),
            },
        )
        .await?;
        Ok(())
    }
}

enum ModelAgent {
    Anthropic(rig::agent::Agent<rig::providers::anthropic::completion::CompletionModel>),
    OpenAI(rig::agent::Agent<rig::providers::openai::completion::CompletionModel>),
}

pub struct Agent {
    model_agent: ModelAgent,
    usage_recorder: UsageRecorder,
}

impl Agent {
    pub async fn prompt(&self, prompt: &str) -> Result<String, AgentCallError> {
        let prompt_response = match &self.model_agent {
            ModelAgent::Anthropic(agent) => agent.prompt(prompt).extended_details().await,
            ModelAgent::OpenAI(agent) => agent.prompt(prompt).extended_details().await,
        }?;
        self.usage_recorder.record(prompt_response.usage).await?;
        Ok(prompt_response.output)
    }

    pub async fn prompt_typed<T>(
        &self,
        prompt: impl Into<Message> + WasmCompatSend,
    ) -> Result<T, AgentCallError>
    where
        T: JsonSchema + DeserializeOwned + WasmCompatSend,
    {
        self.prompt_for_typed_answer(prompt.into(), None).await
    }

    pub async fn prompt_typed_with_history<T>(
        &self,
        prompt: impl Into<Message> + WasmCompatSend,
        history: &mut Vec<Message>,
    ) -> Result<T, AgentCallError>
    where
        T: JsonSchema + DeserializeOwned + WasmCompatSend,
    {
        self.prompt_for_typed_answer(prompt.into(), Some(history))
            .await
    }

    async fn prompt_for_typed_answer<T>(
        &self,
        prompt: Message,
        history: Option<&mut Vec<Message>>,
    ) -> Result<T, AgentCallError>
    where
        T: JsonSchema + DeserializeOwned + WasmCompatSend,
    {
        let output_schema = schemars::schema_for!(T);
        let prompt_response = match &self.model_agent {
            ModelAgent::Anthropic(agent) => {
                prompt_with_output_schema(agent, output_schema, prompt, history).await
            }
            ModelAgent::OpenAI(agent) => {
                prompt_with_output_schema(agent, output_schema, prompt, history).await
            }
        }?;
        self.usage_recorder.record(prompt_response.usage).await?;
        if prompt_response.output.is_empty() {
            return Err(AgentCallError::EmptyAnswer);
        }
        Ok(serde_json::from_str(&prompt_response.output)?)
    }
}

async fn prompt_with_output_schema<M>(
    agent: &rig::agent::Agent<M>,
    output_schema: schemars::Schema,
    prompt: Message,
    history: Option<&mut Vec<Message>>,
) -> Result<PromptResponse, PromptError>
where
    M: CompletionModel + 'static,
{
    let mut agent_with_output_schema = agent.clone();
    agent_with_output_schema.output_schema = Some(output_schema);
    let prompt_request = agent_with_output_schema.prompt(prompt).extended_details();
    match history {
        Some(history) => prompt_request.with_history(history).await,
        None => prompt_request.await,
    }
}

#[cfg(test)]
mod tests {
    use super::{ModelProvider, normalized_token_usage};
    use crate::ai_usage::dto::TokenUsage;

    fn reported_usage(
        input_tokens: u64,
        output_tokens: u64,
        cached_input_tokens: u64,
    ) -> rig::completion::Usage {
        rig::completion::Usage {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            cached_input_tokens,
        }
    }

    #[test]
    fn anthropic_input_gains_the_tokens_read_from_cache() {
        assert_eq!(
            normalized_token_usage(ModelProvider::Anthropic, reported_usage(120, 15, 30)),
            TokenUsage {
                input_tokens: 150,
                output_tokens: 15,
                cached_input_tokens: 30,
            }
        );
    }

    #[test]
    fn openai_input_already_includes_the_tokens_read_from_cache() {
        assert_eq!(
            normalized_token_usage(ModelProvider::Openai, reported_usage(200, 60, 50)),
            TokenUsage {
                input_tokens: 200,
                output_tokens: 60,
                cached_input_tokens: 50,
            }
        );
    }
}
