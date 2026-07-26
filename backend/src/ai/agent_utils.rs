use rig::client::CompletionClient;
use rig::completion::TypedPrompt;
use rig::completion::{Prompt, PromptError, StructuredOutputError};
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

#[derive(Clone, Copy, Debug, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum AgentName {
    CanvasGenerationAgent,
    ConversationNameAgent,
    LivenessCheckAgent,
    DataCatalogDeletionAgent,
    DataCatalogSelectionAgent,
    DataProfilingBeforeAgent,
    FollowupQuestionsAgent,
    FunctionalRoleAnalysisAgent,
    HypothesisGenerationAgent,
    IntegrateCandidatePlanAgent,
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

#[derive(Clone)]
pub enum ModelClient {
    Anthropic(rig::client::Client<anthropic::client::AnthropicExt>),
    OpenAI(rig::client::Client<openai::client::OpenAICompletionsExt>),
}

impl ModelClient {
    pub fn anthropic(api_key: &str) -> Result<Self, Error> {
        let client = anthropic::Client::builder().api_key(api_key).build()?;
        Ok(Self::Anthropic(client))
    }

    pub fn openai(api_key: &str) -> Result<Self, Error> {
        let client = openai::CompletionsClient::builder().api_key(api_key).build()?;
        Ok(Self::OpenAI(client))
    }

    pub fn build_agent(
        &self,
        role: ModelRole,
        agent_name: AgentName,
        system_prompt: &str,
        temperature: f64,
        max_tokens: u64,
    ) -> Agent {
        match self {
            Self::Anthropic(client) => Agent::Anthropic(
                client
                    .agent(model_name_for(self, role))
                    .name(agent_name.as_str())
                    .preamble(system_prompt)
                    .temperature(temperature)
                    .max_tokens(max_tokens)
                    .build(),
            ),
            Self::OpenAI(client) => Agent::OpenAI(
                client
                    .agent(model_name_for(self, role))
                    .name(agent_name.as_str())
                    .preamble(system_prompt)
                    .temperature(temperature)
                    .max_tokens(max_tokens)
                    .build(),
            ),
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

pub enum Agent {
    Anthropic(rig::agent::Agent<rig::providers::anthropic::completion::CompletionModel>),
    OpenAI(rig::agent::Agent<rig::providers::openai::completion::CompletionModel>),
}

impl Agent {
    pub async fn prompt(&self, prompt: &str) -> Result<String, PromptError> {
        match self {
            Self::Anthropic(agent) => agent.prompt(prompt).await,
            Self::OpenAI(agent) => agent.prompt(prompt).await,
        }
    }

    pub async fn prompt_typed<T>(
        &self,
        prompt: impl Into<Message> + WasmCompatSend,
    ) -> Result<T, StructuredOutputError>
    where
        T: JsonSchema + DeserializeOwned + WasmCompatSend,
    {
        match self {
            Self::Anthropic(agent) => agent.prompt_typed::<T>(prompt).await,
            Self::OpenAI(agent) => agent.prompt_typed::<T>(prompt).await,
        }
    }

    pub async fn prompt_typed_with_history<T>(
        &self,
        prompt: impl Into<Message> + WasmCompatSend,
        history: &mut Vec<Message>,
    ) -> Result<T, StructuredOutputError>
    where
        T: JsonSchema + DeserializeOwned + WasmCompatSend,
    {
        match self {
            Self::Anthropic(agent) => agent.prompt_typed::<T>(prompt).with_history(history).await,
            Self::OpenAI(agent) => agent.prompt_typed::<T>(prompt).with_history(history).await,
        }
    }
}
