use crate::ai::error::AgentCallError;
use crate::ai_usage::datatypes::{AiCallContext, ModelCallRecord, TokenUsage};
use crate::ai_usage::service as ai_usage_service;
use crate::common::AppState;
use rig_core::DynModel;
use rig_core::completion::{CompletionRequest, CompletionResponse};
use rig_core::message::Message;
use rig_core::operation::Completion;
use rig_core::providers::anthropic::{Anthropic, AnthropicConfig};
use rig_core::providers::openai::{OpenAI, OpenAIConfig};
use rig_core::wasm_compat::WasmCompatSend;
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
    reason = "variant names are the agent names recorded with usage and shown in logs"
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

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumString,
    strum::IntoStaticStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ModelProvider {
    Anthropic,
    Openai,
}

impl ModelProvider {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Self::Anthropic => "ANTHROPIC",
            Self::Openai => "OPENAI",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "ANTHROPIC" => Some(Self::Anthropic),
            "OPENAI" => Some(Self::Openai),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum ModelClient {
    Anthropic(Box<Anthropic>),
    OpenAI(Box<OpenAI>),
}

impl ModelClient {
    pub fn anthropic(api_key: &str, base_url: Option<&str>) -> Self {
        let anthropic_config = AnthropicConfig::new(api_key);
        let anthropic_config = match base_url {
            Some(base_url) => anthropic_config.with_base_url(base_url),
            None => anthropic_config,
        };
        Self::Anthropic(Box::new(anthropic_config.client()))
    }

    pub fn openai(api_key: &str, base_url: Option<&str>) -> Self {
        let openai_config = OpenAIConfig::new(api_key);
        let openai_config = match base_url {
            Some(base_url) => openai_config.with_base_url(base_url),
            None => openai_config,
        };
        Self::OpenAI(Box::new(openai_config.client()))
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
        let completion_model = match self {
            Self::Anthropic(client) => client.completion(model_name).erase(),
            Self::OpenAI(client) => client.chat(model_name).erase(),
        };
        Agent {
            completion_model,
            system_prompt: system_prompt.to_string(),
            temperature,
            max_tokens,
            usage_recorder: UsageRecorder {
                app_state: app_state.clone(),
                call_context: call_context.clone(),
                agent_name,
                model_name,
                model_provider: self.model_provider(),
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

pub fn token_usage_from_reported_usage(reported_usage: rig_core::completion::Usage) -> TokenUsage {
    TokenUsage {
        input_tokens: reported_usage.input_tokens.unwrap_or(0),
        output_tokens: reported_usage.output_tokens.unwrap_or(0),
        cached_input_tokens: reported_usage.cached_input_tokens.unwrap_or(0),
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
    async fn record(
        &self,
        reported_usage: rig_core::completion::Usage,
    ) -> Result<(), AgentCallError> {
        ai_usage_service::record_model_call(
            &self.app_state,
            &self.call_context,
            ModelCallRecord {
                agent_name: self.agent_name.as_str(),
                provider: self.model_provider.into(),
                model: self.model_name,
                token_usage: token_usage_from_reported_usage(reported_usage),
            },
        )
        .await?;
        Ok(())
    }
}

pub struct Agent {
    completion_model: DynModel<Completion>,
    system_prompt: String,
    temperature: f64,
    max_tokens: u64,
    usage_recorder: UsageRecorder,
}

impl Agent {
    pub async fn prompt(&self, prompt: &str) -> Result<String, AgentCallError> {
        let completion_response = self
            .complete(self.completion_request(Message::user(prompt), &[]))
            .await?;
        Ok(completion_response.text())
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
        let earlier_messages = history.as_deref().map_or(&[][..], Vec::as_slice);
        let completion_request = self
            .completion_request(prompt.clone(), earlier_messages)
            .output_schema(schemars::schema_for!(T));
        let completion_response = self.complete(completion_request).await?;
        if let Some(history) = history {
            history.push(prompt);
            history.extend(completion_response.message());
        }
        let answer_text = completion_response.text();
        if answer_text.is_empty() {
            return Err(AgentCallError::EmptyAnswer);
        }
        Ok(serde_json::from_str(&answer_text)?)
    }

    fn completion_request(
        &self,
        prompt: Message,
        earlier_messages: &[Message],
    ) -> CompletionRequest {
        CompletionRequest::new(prompt)
            .messages(earlier_messages.iter().cloned())
            .preamble(self.system_prompt.as_str())
            .temperature(self.temperature)
            .max_tokens(self.max_tokens)
    }

    async fn complete(
        &self,
        completion_request: CompletionRequest,
    ) -> Result<CompletionResponse, AgentCallError> {
        let completion_response = self
            .completion_model
            .call(completion_request)
            .await
            .map_err(Box::new)?;
        self.usage_recorder
            .record(completion_response.usage)
            .await?;
        Ok(completion_response)
    }
}

#[cfg(test)]
mod tests {
    use super::token_usage_from_reported_usage;
    use crate::ai_usage::datatypes::TokenUsage;
    use rig_core::completion::Usage;

    #[test]
    fn reported_counts_are_recorded_as_reported() {
        let reported_usage = Usage::new()
            .input_tokens(150)
            .output_tokens(15)
            .cached_input_tokens(30);

        assert_eq!(
            token_usage_from_reported_usage(reported_usage),
            TokenUsage {
                input_tokens: 150,
                output_tokens: 15,
                cached_input_tokens: 30,
            }
        );
    }

    #[test]
    fn counts_the_provider_did_not_report_are_recorded_as_zero() {
        assert_eq!(
            token_usage_from_reported_usage(Usage::new()),
            TokenUsage {
                input_tokens: 0,
                output_tokens: 0,
                cached_input_tokens: 0,
            }
        );
    }
}
