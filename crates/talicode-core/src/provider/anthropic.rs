// SPDX-License-Identifier: MIT
//! Anthropic Messages-API provider.
//!
//! Implements #9. Calls `POST /v1/messages` over `reqwest` and forces the
//! findings schema with structured outputs (`output_config.format`) — the
//! reliable, thinking-compatible way to constrain output on current Claude
//! models. The request/response shaping is factored into pure functions so it
//! is unit-tested without a network, and the HTTP round-trip is covered with a
//! mocked server.

use super::{Completion, CompletionRequest, Provider, ProviderError, Usage};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Environment variable holding the Anthropic API key.
pub const API_KEY_ENV: &str = "ANTHROPIC_API_KEY";
const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";
const MAX_TOKENS: u32 = 4096;

/// A [`Provider`] backed by the Anthropic Messages API.
pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    http: reqwest::Client,
}

impl AnthropicProvider {
    /// Build from `ANTHROPIC_API_KEY`, failing clearly if it is unset.
    pub fn from_env() -> Result<Self, ProviderError> {
        let key = resolve_key(std::env::var(API_KEY_ENV).ok())?;
        Ok(Self::new(key, DEFAULT_BASE_URL.to_string()))
    }

    /// Build against an explicit key + base URL (used by tests).
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<Completion, ProviderError> {
        let body = build_body(&request);
        let resp = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| ProviderError::Request(e.to_string()))?;

        let status = resp.status();
        let value: Value = resp
            .json()
            .await
            .map_err(|e| ProviderError::Response(e.to_string()))?;

        if !status.is_success() {
            return Err(ProviderError::Request(format!("HTTP {status}: {value}")));
        }
        parse_completion(&value)
    }
}

/// Resolve the API key from an optional env value.
fn resolve_key(from_env: Option<String>) -> Result<String, ProviderError> {
    match from_env {
        Some(k) if !k.is_empty() => Ok(k),
        _ => Err(ProviderError::MissingApiKey(API_KEY_ENV)),
    }
}

/// Build the Messages-API request body: forces the tool's schema via structured
/// outputs and sets effort. `max_tokens` is fixed; findings are small.
fn build_body(req: &CompletionRequest) -> Value {
    json!({
        "model": req.model,
        "max_tokens": MAX_TOKENS,
        "system": req.system,
        "output_config": {
            "effort": req.effort,
            "format": { "type": "json_schema", "schema": req.tool.input_schema }
        },
        "messages": [ { "role": "user", "content": req.user } ]
    })
}

/// Extract the structured output (the first text block, parsed as JSON) and
/// token usage from a Messages-API response.
fn parse_completion(value: &Value) -> Result<Completion, ProviderError> {
    if value.get("stop_reason").and_then(Value::as_str) == Some("refusal") {
        return Err(ProviderError::Response("model refused the request".into()));
    }
    let text = value
        .get("content")
        .and_then(Value::as_array)
        .and_then(|blocks| {
            blocks
                .iter()
                .find(|b| b.get("type").and_then(Value::as_str) == Some("text"))
        })
        .and_then(|b| b.get("text"))
        .and_then(Value::as_str)
        .ok_or_else(|| ProviderError::Response("no text block in response".into()))?;

    let output: Value = serde_json::from_str(text)
        .map_err(|e| ProviderError::Response(format!("output was not valid JSON: {e}")))?;

    Ok(Completion {
        output,
        usage: parse_usage(value.get("usage")),
    })
}

/// Parse the `usage` object, defaulting any missing field to 0.
fn parse_usage(usage: Option<&Value>) -> Usage {
    let field = |k: &str| {
        usage
            .and_then(|u| u.get(k))
            .and_then(Value::as_u64)
            .unwrap_or(0)
    };
    Usage {
        input_tokens: field("input_tokens"),
        output_tokens: field("output_tokens"),
        cache_read_input_tokens: field("cache_read_input_tokens"),
        cache_creation_input_tokens: field("cache_creation_input_tokens"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ToolSpec;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn req() -> CompletionRequest {
        CompletionRequest {
            model: "claude-sonnet-5".into(),
            effort: "medium".into(),
            system: "sys".into(),
            user: "audit this".into(),
            tool: ToolSpec {
                name: "report_findings".into(),
                description: "report".into(),
                input_schema: json!({"type": "object"}),
            },
        }
    }

    #[test]
    fn resolve_key_requires_a_nonempty_value() {
        assert_eq!(resolve_key(Some("sk-x".into())).unwrap(), "sk-x");
        assert!(matches!(
            resolve_key(None),
            Err(ProviderError::MissingApiKey(_))
        ));
        assert!(matches!(
            resolve_key(Some(String::new())),
            Err(ProviderError::MissingApiKey(_))
        ));
    }

    #[test]
    fn build_body_forces_schema_and_effort() {
        let body = build_body(&req());
        assert_eq!(body["model"], "claude-sonnet-5");
        assert_eq!(body["output_config"]["effort"], "medium");
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert_eq!(body["output_config"]["format"]["schema"]["type"], "object");
    }

    #[test]
    fn parse_completion_reads_json_text_and_usage() {
        let value = json!({
            "content": [{"type": "text", "text": "{\"findings\": [{\"rule\": \"code-no-keys\"}]}"}],
            "usage": {"input_tokens": 12, "output_tokens": 3, "cache_read_input_tokens": 4}
        });
        let c = parse_completion(&value).unwrap();
        assert_eq!(c.output["findings"][0]["rule"], "code-no-keys");
        assert_eq!(c.usage.input_tokens, 12);
        assert_eq!(c.usage.cache_read_input_tokens, 4);
    }

    #[test]
    fn parse_completion_rejects_refusal() {
        let value = json!({"stop_reason": "refusal", "content": []});
        assert!(matches!(
            parse_completion(&value),
            Err(ProviderError::Response(_))
        ));
    }

    #[tokio::test]
    async fn complete_maps_a_mocked_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "content": [{"type": "text", "text": "{\"findings\": []}"}],
                "usage": {"input_tokens": 5, "output_tokens": 1}
            })))
            .mount(&server)
            .await;

        let p = AnthropicProvider::new("sk-test".into(), server.uri());
        let c = p.complete(req()).await.unwrap();
        assert_eq!(c.output, json!({"findings": []}));
        assert_eq!(c.usage.output_tokens, 1);
    }

    #[tokio::test]
    async fn complete_surfaces_http_errors() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(
                ResponseTemplate::new(401).set_body_json(json!({"error": "unauthorized"})),
            )
            .mount(&server)
            .await;

        let p = AnthropicProvider::new("sk-bad".into(), server.uri());
        assert!(matches!(
            p.complete(req()).await,
            Err(ProviderError::Request(_))
        ));
    }
}
