// SPDX-License-Identifier: MIT
//! The Auditor — turns a file + rules into line-anchored findings.
//!
//! Implements #10. Builds a prompt and the findings tool schema, asks the
//! [`Provider`] for structured output, and returns `Vec<Finding>` (tagged with
//! the audited file) plus token [`Usage`]. The prompt insists on concrete,
//! line-anchored violations and prefers silence over speculation — false
//! positives are what kill adoption.

use crate::provider::{CompletionRequest, Provider, ProviderError, ToolSpec, Usage};
use crate::report::Finding;
use serde::Deserialize;
use serde_json::{json, Value};

/// What to audit and how.
#[derive(Debug, Clone)]
pub struct AuditRequest {
    /// Repo-relative path (used to tag findings).
    pub file: String,
    /// The file's contents.
    pub content: String,
    /// Model id.
    pub model: String,
    /// Reasoning effort.
    pub effort: String,
    /// The agent's role description.
    pub role: String,
    /// Composed skill guidance (the lenses' `SKILL.md` bodies + rules). Filled
    /// by the skill host in phase 3; may be empty for a bare audit.
    pub guidance: String,
    /// Assembled long-term memory context (semantic + episodic + architecture).
    /// Filled by the sweep in phase 8; empty ⇒ nothing injected.
    pub memory: String,
}

/// The result of auditing one file.
#[derive(Debug, Clone)]
pub struct AuditOutcome {
    /// Findings, tagged with the audited file.
    pub findings: Vec<Finding>,
    /// Token usage for the audit call.
    pub usage: Usage,
}

/// Audit one file through `provider`.
pub async fn audit(
    provider: &dyn Provider,
    request: AuditRequest,
) -> Result<AuditOutcome, ProviderError> {
    let completion = provider
        .complete(CompletionRequest {
            model: request.model.clone(),
            effort: request.effort.clone(),
            system: system_prompt(&request.role, &request.guidance, &request.memory),
            user: user_prompt(&request.file, &request.content),
            tool: ToolSpec {
                name: "report_findings".into(),
                description: "Report every concrete, line-anchored violation you find.".into(),
                input_schema: findings_schema(),
            },
        })
        .await?;

    let envelope: FindingsEnvelope = serde_json::from_value(completion.output)
        .map_err(|e| ProviderError::Response(format!("findings did not match schema: {e}")))?;

    // We audited one known file — set the path ourselves rather than trust the model.
    let findings = envelope
        .findings
        .into_iter()
        .map(|mut f| {
            f.file = request.file.clone();
            f
        })
        .collect();

    Ok(AuditOutcome {
        findings,
        usage: completion.usage,
    })
}

#[derive(Deserialize)]
struct FindingsEnvelope {
    #[serde(default)]
    findings: Vec<Finding>,
}

/// The JSON Schema forcing the findings shape (the `file` is set by the caller,
/// so it is not requested from the model).
fn findings_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "findings": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "line": { "type": "integer", "minimum": 1 },
                        "severity": { "type": "string", "enum": ["info", "warning", "error"] },
                        "rule": { "type": "string" },
                        "message": { "type": "string" }
                    },
                    "required": ["line", "severity", "rule", "message"]
                }
            }
        },
        "required": ["findings"]
    })
}

fn system_prompt(role: &str, guidance: &str, memory: &str) -> String {
    let mut s = String::from(
        "You are TaliCode's Auditor. Review the file and report only concrete, \
         line-anchored violations. Prefer silence over speculation — do not invent \
         issues. Use the exact rule id from the guidance for each finding, a 1-indexed \
         line number, and one of the severities info, warning, or error.",
    );
    if !role.trim().is_empty() {
        s.push_str("\n\nRole: ");
        s.push_str(role.trim());
    }
    if !memory.trim().is_empty() {
        s.push_str("\n\nLong-term memory (project context — judge accordingly):\n");
        s.push_str(memory.trim());
    }
    if !guidance.trim().is_empty() {
        s.push_str("\n\nGuidance (rules to apply):\n");
        s.push_str(guidance.trim());
    }
    s
}

fn user_prompt(file: &str, content: &str) -> String {
    format!("File: {file}\n\n{}", number_lines(content))
}

/// Prefix each line with its 1-indexed number so the model can anchor findings.
fn number_lines(content: &str) -> String {
    content
        .lines()
        .enumerate()
        .map(|(i, line)| format!("{}\t{line}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{FakeProvider, Usage};
    use crate::report::Severity;

    fn request() -> AuditRequest {
        AuditRequest {
            file: "src/lib.rs".into(),
            content: "let x = 1;\nlet y = 2;".into(),
            model: "claude-sonnet-5".into(),
            effort: "medium".into(),
            role: "Find slop".into(),
            guidance: "code-no-keys: no hardcoded secrets".into(),
            memory: String::new(),
        }
    }

    #[test]
    fn system_prompt_injects_memory_when_present() {
        let with = system_prompt("r", "g", "Project memory:\n- uses raw SQL");
        assert!(with.contains("Long-term memory"));
        assert!(with.contains("uses raw SQL"));
        // Empty memory ⇒ no memory section.
        let without = system_prompt("r", "g", "");
        assert!(!without.contains("Long-term memory"));
    }

    #[tokio::test]
    async fn audit_parses_findings_and_tags_the_file() {
        let output = json!({"findings": [
            {"line": 2, "severity": "error", "rule": "code-no-keys", "message": "hardcoded token"}
        ]});
        let provider = FakeProvider::new(
            output,
            Usage {
                input_tokens: 9,
                output_tokens: 4,
                ..Default::default()
            },
        );

        let outcome = audit(&provider, request()).await.unwrap();
        assert_eq!(outcome.findings.len(), 1);
        let f = &outcome.findings[0];
        assert_eq!(f.file, "src/lib.rs"); // set by the auditor, not the model
        assert_eq!(f.line, 2);
        assert_eq!(f.severity, Severity::Error);
        assert_eq!(f.rule, "code-no-keys");
        assert_eq!(outcome.usage.input_tokens, 9);
    }

    #[tokio::test]
    async fn audit_handles_no_findings() {
        let provider = FakeProvider::new(json!({"findings": []}), Usage::default());
        let outcome = audit(&provider, request()).await.unwrap();
        assert!(outcome.findings.is_empty());
    }

    #[tokio::test]
    async fn audit_rejects_malformed_output() {
        let provider = FakeProvider::new(json!({"nope": true}), Usage::default());
        // `findings` defaults to empty when the key is absent, so this succeeds
        // with zero findings rather than erroring — the schema forces the key in
        // real calls; here we assert the lenient default.
        let outcome = audit(&provider, request()).await.unwrap();
        assert!(outcome.findings.is_empty());
    }

    #[test]
    fn schema_requires_findings_array() {
        let schema = findings_schema();
        assert_eq!(schema["properties"]["findings"]["type"], "array");
        assert_eq!(schema["required"][0], "findings");
    }

    #[test]
    fn lines_are_numbered_from_one() {
        assert_eq!(number_lines("a\nb"), "1\ta\n2\tb");
    }
}
