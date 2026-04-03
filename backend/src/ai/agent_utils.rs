use rig::client::CompletionClient;
use rig::completion::TypedPrompt;
use rig::completion::{Prompt, PromptError, StructuredOutputError};
use rig::message::Message;
use rig::http_client::Error;
use rig::providers::anthropic;
use rig::wasm_compat::WasmCompatSend;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

#[derive(Clone)]
pub enum ModelClient {
    Anthropic(rig::client::Client<anthropic::client::AnthropicExt>),
}

impl ModelClient {
    pub fn anthropic(anthropic_api_key: &str) -> Result<Self, Error> {
        let client = anthropic::Client::builder()
            .api_key(anthropic_api_key)
            .build()?;
        Ok(Self::Anthropic(client))
    }

    pub fn build_agent(
        &self,
        model_name: &str,
        agent_name: &str,
        system_prompt: &str,
        temperature: f64,
        max_tokens: u64,
    ) -> Agent {
        match self {
            Self::Anthropic(client) => Agent::Anthropic(
                client
                    .agent(model_name)
                    .name(agent_name)
                    .preamble(system_prompt)
                    .temperature(temperature)
                    .max_tokens(max_tokens)
                    .build(),
            ),
        }
    }
}

pub enum Agent {
    Anthropic(rig::agent::Agent<rig::providers::anthropic::completion::CompletionModel>),
}

impl Agent {
    pub async fn prompt(&self, prompt: &str) -> Result<String, PromptError> {
        match self {
            Self::Anthropic(agent) => agent.prompt(prompt).await,
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
            Self::Anthropic(agent) => agent
                .prompt_typed::<T>(prompt)
                .with_history(history)
                .await,
        }
    }
}
