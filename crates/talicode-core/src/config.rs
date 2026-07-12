// SPDX-License-Identifier: MIT
//! The `config.tali` schema, loader, and validation.
//!
//! Implements #6. `config.tali` is a custom extension (it signals "this is a
//! TaliCode harness", like `.tf` for Terraform) but its bytes are plain YAML,
//! parsed here with `serde_yaml`. Unknown-but-valid fields are ignored rather
//! than rejected, so a config can carry forward-looking keys (e.g. a healing
//! `fallback`) without breaking the detect-only MVP.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The config file TaliCode looks for at the repo root.
pub const CONFIG_FILENAME: &str = "config.tali";

/// The default reasoning effort for an agent when none is set.
pub const DEFAULT_EFFORT: &str = "medium";

/// A parsed `config.tali`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Config {
    /// Config schema version (e.g. `"1.0"`).
    pub version: String,
    /// Human-readable pipeline name.
    pub name: String,
    /// Named agents (e.g. `auditor`). Keyed by name.
    #[serde(default)]
    pub agents: BTreeMap<String, Agent>,
    /// The ordered steps of the pipeline.
    #[serde(default)]
    pub execution_flow: Vec<Step>,
    /// Skills selected for the sweep. Empty ⇒ the bundled `code-review`
    /// orchestrator (all default lenses).
    #[serde(default)]
    pub skills: Vec<String>,
    /// Long-term memory settings (all defaulted; omit the section entirely to
    /// keep the pre-memory behavior).
    #[serde(default)]
    pub memory: MemoryConfig,
}

/// Settings for TaliCode's five-type memory. Every field has a default, so an
/// existing `config.tali` with no `memory:` section behaves exactly as before.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct MemoryConfig {
    /// Master switch for memory injection.
    pub enabled: bool,
    /// Per-turn context budget (tokens) for the assembled memory block.
    pub context_budget_tokens: usize,
    /// Soft conversation budget; compression may fire past this once idle.
    pub working_memory_soft_budget_tokens: u64,
    /// Hard conversation ceiling; compression fires even mid-task here.
    pub working_memory_hard_budget_tokens: u64,
    /// `"search"` (native skill retrieval) or `"all"` (run every selected lens).
    pub skill_retrieval: String,
    /// Max skills the native search injects per trigger.
    pub skill_search_limit: usize,
    /// Skills always injected regardless of the search (the security floor).
    pub always_run_skills: Vec<String>,
    /// Max semantic facts injected per session.
    pub semantic_limit: usize,
    /// Whether episodic memory is enabled.
    pub episodic: bool,
    /// Recurrence count at which an experience auto-promotes to a skill.
    pub experience_to_skill_threshold: usize,
    /// Whether recurring experiences auto-promote to skills.
    pub auto_promote_skills: bool,
    /// TTL (hours) for scratch episodes.
    pub episodic_scratch_ttl_hours: u64,
    /// Keyword-relevance weight in the episodic ranking.
    pub rank_w_fts: f64,
    /// Recency weight in the episodic ranking.
    pub rank_w_recency: f64,
    /// Recency half-life (days) in the episodic ranking.
    pub rank_half_life_days: f64,
    /// Whether architectural memory is built/refreshed.
    pub architecture: bool,
    /// Whether the architecture overview is injected into context.
    pub architecture_overview_in_context: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        MemoryConfig {
            enabled: true,
            context_budget_tokens: 2000,
            working_memory_soft_budget_tokens: 250_000,
            working_memory_hard_budget_tokens: 500_000,
            skill_retrieval: "search".to_string(),
            skill_search_limit: 6,
            always_run_skills: vec![
                "code-no-keys".to_string(),
                "code-no-credentials".to_string(),
            ],
            semantic_limit: 8,
            episodic: true,
            experience_to_skill_threshold: 2,
            auto_promote_skills: true,
            episodic_scratch_ttl_hours: 24,
            rank_w_fts: 0.4,
            rank_w_recency: 0.2,
            rank_half_life_days: 30.0,
            architecture: true,
            architecture_overview_in_context: true,
        }
    }
}

/// An agent definition — which provider/model plays a role.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Agent {
    /// Provider id (e.g. `"anthropic"`).
    pub provider: String,
    /// Model id (e.g. `"claude-sonnet-5"`).
    pub model: String,
    /// Free-text role description injected into the prompt.
    #[serde(default)]
    pub role: String,
    /// Reasoning effort (`low`..`max`); defaults to [`DEFAULT_EFFORT`].
    #[serde(default = "default_effort")]
    pub effort: String,
}

fn default_effort() -> String {
    DEFAULT_EFFORT.to_string()
}

/// One step in the `execution_flow`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Step {
    /// Step id (e.g. `"slop_sweep"`).
    pub step: String,
    /// The agent this step runs (must exist in [`Config::agents`]).
    pub agent: String,
    /// Optional target glob for non-staged sweeps.
    #[serde(default)]
    pub target: Option<String>,
}

/// Errors from loading or validating a `config.tali`.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// No `config.tali` at the expected path.
    #[error("no `{CONFIG_FILENAME}` found at {0} — run `tali init` first")]
    NotFound(PathBuf),
    /// The file could not be read.
    #[error("could not read {0}: {1}")]
    Read(PathBuf, #[source] std::io::Error),
    /// The YAML did not parse into the schema.
    #[error("invalid `{CONFIG_FILENAME}`: {0}")]
    Parse(#[from] serde_yaml::Error),
    /// The config parsed but is internally inconsistent.
    #[error("invalid `{CONFIG_FILENAME}`: {0}")]
    Validation(String),
}

impl Config {
    /// Parse a config from a YAML string (the bytes of a `.tali` file).
    pub fn parse(yaml: &str) -> Result<Self, ConfigError> {
        let config: Config = serde_yaml::from_str(yaml)?;
        config.validate()?;
        Ok(config)
    }

    /// Load `config.tali` from `dir`.
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = dir.as_ref().join(CONFIG_FILENAME);
        if !path.exists() {
            return Err(ConfigError::NotFound(path));
        }
        let text = std::fs::read_to_string(&path).map_err(|e| ConfigError::Read(path, e))?;
        Config::parse(&text)
    }

    /// Check internal consistency: every step references a defined agent.
    pub fn validate(&self) -> Result<(), ConfigError> {
        for step in &self.execution_flow {
            if !self.agents.contains_key(&step.agent) {
                return Err(ConfigError::Validation(format!(
                    "step `{}` references unknown agent `{}`",
                    step.step, step.agent
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
version: "1.0"
name: "Local-Auto-Heal-Pipeline"
agents:
  auditor:
    provider: "anthropic"
    model: "claude-sonnet-5"
    role: "Identify AI slop, unverified imports, and security risks."
execution_flow:
  - step: "slop_sweep"
    agent: "auditor"
    target: "./src/**/*.rs"
skills:
  - code-review
"#;

    #[test]
    fn valid_config_parses_and_defaults_effort() {
        let c = Config::parse(VALID).expect("valid config parses");
        assert_eq!(c.version, "1.0");
        assert_eq!(c.agents["auditor"].effort, "medium");
        assert_eq!(c.execution_flow[0].agent, "auditor");
        assert_eq!(c.skills, vec!["code-review"]);
    }

    #[test]
    fn memory_defaults_when_section_absent() {
        let c = Config::parse(VALID).expect("valid config parses");
        // No `memory:` block ⇒ the full default config.
        assert_eq!(c.memory, MemoryConfig::default());
        assert!(c.memory.enabled);
        assert_eq!(c.memory.working_memory_soft_budget_tokens, 250_000);
        assert_eq!(c.memory.working_memory_hard_budget_tokens, 500_000);
        assert_eq!(c.memory.skill_retrieval, "search");
        assert!(c
            .memory
            .always_run_skills
            .contains(&"code-no-keys".to_string()));
    }

    #[test]
    fn memory_partial_section_overrides_only_named_fields() {
        let yaml = r#"
version: "1.0"
name: "x"
agents:
  auditor: { provider: "anthropic", model: "claude-sonnet-5" }
memory:
  skill_retrieval: "all"
  skill_search_limit: 3
"#;
        let c = Config::parse(yaml).expect("partial memory block parses");
        assert_eq!(c.memory.skill_retrieval, "all");
        assert_eq!(c.memory.skill_search_limit, 3);
        // Unspecified fields keep their defaults.
        assert!(c.memory.enabled);
        assert_eq!(c.memory.experience_to_skill_threshold, 2);
    }

    #[test]
    fn unknown_fields_are_ignored_not_rejected() {
        let yaml = r#"
version: "1.0"
name: "x"
agents:
  auditor: { provider: "anthropic", model: "claude-sonnet-5" }
execution_flow:
  - step: "slop_sweep"
    agent: "auditor"
    anti_slop: { mode: "aggressive" }
    fallback: { action: "auto_fix", resolver: "surgeon", max_retries: 2 }
"#;
        let c = Config::parse(yaml).expect("forward-looking keys must not be rejected");
        assert_eq!(c.agents["auditor"].effort, "medium");
    }

    #[test]
    fn step_referencing_unknown_agent_is_rejected() {
        let yaml = r#"
version: "1.0"
name: "x"
agents:
  auditor: { provider: "anthropic", model: "claude-sonnet-5" }
execution_flow:
  - step: "slop_sweep"
    agent: "ghost"
"#;
        let err = Config::parse(yaml).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)), "got {err:?}");
    }

    #[test]
    fn malformed_yaml_is_a_parse_error() {
        let err = Config::parse("version: [unterminated").unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)), "got {err:?}");
    }

    #[test]
    fn missing_file_is_not_found() {
        let dir = std::env::temp_dir().join("talicode-cfg-missing-xyz");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let err = Config::load(&dir).unwrap_err();
        assert!(matches!(err, ConfigError::NotFound(_)), "got {err:?}");
    }
}
