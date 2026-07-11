// SPDX-License-Identifier: MIT
//! The provider seam — how TaliCode invokes an LLM.
//!
//! Implements #8. A [`Provider`] takes a [`CompletionRequest`] (a prompt plus a
//! forced tool schema) and returns the tool's structured output and token
//! [`Usage`]. The trait is the test seam: [`FakeProvider`] backs unit tests
//! with no network. The Anthropic implementation lands in #9.

pub mod anthropic;

use async_trait::async_trait;
use serde_json::Value;

/// Token usage reported by a provider for one completion.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Usage {
    /// Uncached input tokens.
    pub input_tokens: u64,
    /// Output (completion) tokens.
    pub output_tokens: u64,
    /// Tokens served from the prompt cache.
    pub cache_read_input_tokens: u64,
    /// Tokens written to the prompt cache.
    pub cache_creation_input_tokens: u64,
}

/// A tool the model must call, forcing structured output.
#[derive(Debug, Clone)]
pub struct ToolSpec {
    /// Tool name (e.g. `report_findings`).
    pub name: String,
    /// What the tool is for (helps the model call it correctly).
    pub description: String,
    /// JSON Schema for the tool's `input` object.
    pub input_schema: Value,
}

/// A request for one structured completion.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    /// Model id (e.g. `claude-sonnet-5`).
    pub model: String,
    /// Reasoning effort (`low`..`max`).
    pub effort: String,
    /// System prompt.
    pub system: String,
    /// User message content.
    pub user: String,
    /// The tool the model is forced to call.
    pub tool: ToolSpec,
}

/// The result of a structured completion.
#[derive(Debug, Clone)]
pub struct Completion {
    /// The tool call's `input` object (schema-validated by the model).
    pub output: Value,
    /// Token usage for this completion.
    pub usage: Usage,
}

/// Errors a provider can return.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// No provider is registered under this name.
    #[error("unknown provider `{0}`")]
    Unknown(String),
    /// A required API-key environment variable is unset.
    #[error("missing API key: set {0}")]
    MissingApiKey(&'static str),
    /// The HTTP request to the provider failed.
    #[error("provider request failed: {0}")]
    Request(String),
    /// The provider replied, but not in the expected shape.
    #[error("provider returned an unexpected response: {0}")]
    Response(String),
}

/// A backend that can produce a structured completion.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Run one completion, forcing the model to call `request.tool`.
    async fn complete(&self, request: CompletionRequest) -> Result<Completion, ProviderError>;
}

/// Resolve a provider by its config `provider` name.
pub fn build_provider(name: &str) -> Result<Box<dyn Provider>, ProviderError> {
    match name {
        "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::from_env()?)),
        other => Err(ProviderError::Unknown(other.to_string())),
    }
}

/// An in-process provider for tests: returns canned output + usage.
pub struct FakeProvider {
    output: Value,
    usage: Usage,
}

impl FakeProvider {
    /// Build a fake that always returns `output` and `usage`.
    pub fn new(output: Value, usage: Usage) -> Self {
        Self { output, usage }
    }
}

#[async_trait]
impl Provider for FakeProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<Completion, ProviderError> {
        Ok(Completion {
            output: self.output.clone(),
            usage: self.usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn req() -> CompletionRequest {
        CompletionRequest {
            model: "m".into(),
            effort: "medium".into(),
            system: "s".into(),
            user: "u".into(),
            tool: ToolSpec {
                name: "t".into(),
                description: "d".into(),
                input_schema: json!({}),
            },
        }
    }

    #[tokio::test]
    async fn fake_provider_returns_canned_output_and_usage() {
        let usage = Usage {
            input_tokens: 10,
            output_tokens: 2,
            ..Default::default()
        };
        let p = FakeProvider::new(json!({"findings": []}), usage);
        let c = p.complete(req()).await.unwrap();
        assert_eq!(c.output, json!({"findings": []}));
        assert_eq!(c.usage.input_tokens, 10);
    }

    #[test]
    fn unknown_provider_errors() {
        // `Box<dyn Provider>` isn't Debug, so match the Result rather than unwrap_err.
        assert!(matches!(
            build_provider("openai"),
            Err(ProviderError::Unknown(_))
        ));
    }
}
