// SPDX-License-Identifier: MIT
//! The skill model and parser.
//!
//! Implements #11. A skill is a folder with a `SKILL.md` (YAML frontmatter —
//! `name`, `description`, optional `runs:` — followed by the audit guidance)
//! and, for a **lens**, a `rules.yaml` (a list of rules). A skill that declares
//! `runs:` is an **orchestrator** (it lists other skills and carries no rules of
//! its own); everything else is a lens.

use serde::Deserialize;
use std::path::Path;

/// A parsed skill folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    /// Unique skill name (from frontmatter).
    pub name: String,
    /// One-line description.
    pub description: String,
    /// The `SKILL.md` body — the audit guidance fed to the Auditor.
    pub guidance: String,
    /// Lens or orchestrator.
    pub kind: SkillKind,
}

/// What a skill *is*.
#[derive(Debug, Clone, PartialEq)]
pub enum SkillKind {
    /// A judgment lens carrying concrete rules.
    Lens { rules: Vec<Rule> },
    /// An orchestrator that runs other skills by name.
    Orchestrator { runs: Vec<String> },
}

/// A single rule within a lens.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Rule {
    /// Rule id (e.g. `no-hardcoded-keys`).
    pub id: String,
    /// What the rule flags — guidance for the Auditor.
    pub message: String,
    /// Default severity for this rule, if any (`info`/`warning`/`error`).
    #[serde(default)]
    pub severity: Option<String>,
}

/// Errors from parsing a skill.
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    /// `SKILL.md` had no `--- ... ---` frontmatter block.
    #[error("skill `{0}`: SKILL.md is missing its `--- frontmatter ---` block")]
    MissingFrontmatter(String),
    /// Frontmatter YAML did not parse.
    #[error("skill `{0}`: invalid frontmatter: {1}")]
    Frontmatter(String, #[source] serde_yaml::Error),
    /// A lens had no `rules.yaml`.
    #[error("skill `{0}`: a lens needs a rules.yaml (or declare `runs:` to be an orchestrator)")]
    MissingRules(String),
    /// `rules.yaml` did not parse.
    #[error("skill `{0}`: invalid rules.yaml: {1}")]
    Rules(String, #[source] serde_yaml::Error),
    /// A file could not be read.
    #[error("skill `{0}`: {1}")]
    Io(String, #[source] std::io::Error),
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    name: String,
    description: String,
    #[serde(default)]
    runs: Option<Vec<String>>,
}

impl Skill {
    /// Parse a skill from its `SKILL.md` text and optional `rules.yaml` text.
    ///
    /// `dir_name` is only used to make error messages point at the folder.
    pub fn parse(
        dir_name: &str,
        skill_md: &str,
        rules_yaml: Option<&str>,
    ) -> Result<Self, SkillError> {
        let (fm_text, body) = split_frontmatter(skill_md)
            .ok_or_else(|| SkillError::MissingFrontmatter(dir_name.to_string()))?;
        let fm: Frontmatter = serde_yaml::from_str(fm_text)
            .map_err(|e| SkillError::Frontmatter(dir_name.to_string(), e))?;

        let kind = match fm.runs {
            Some(runs) => SkillKind::Orchestrator { runs },
            None => {
                let text = rules_yaml
                    .ok_or_else(|| SkillError::MissingRules(dir_name.to_string()))?;
                let rules: Vec<Rule> = serde_yaml::from_str(text)
                    .map_err(|e| SkillError::Rules(dir_name.to_string(), e))?;
                SkillKind::Lens { rules }
            }
        };

        Ok(Skill {
            name: fm.name,
            description: fm.description,
            guidance: body.trim().to_string(),
            kind,
        })
    }

    /// Load a skill from a folder containing `SKILL.md` (+ `rules.yaml` for a lens).
    pub fn load(dir: &Path) -> Result<Self, SkillError> {
        let dir_name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("<skill>")
            .to_string();
        let skill_md = std::fs::read_to_string(dir.join("SKILL.md"))
            .map_err(|e| SkillError::Io(dir_name.clone(), e))?;
        let rules_path = dir.join("rules.yaml");
        let rules = if rules_path.exists() {
            Some(
                std::fs::read_to_string(&rules_path)
                    .map_err(|e| SkillError::Io(dir_name.clone(), e))?,
            )
        } else {
            None
        };
        Skill::parse(&dir_name, &skill_md, rules.as_deref())
    }

    /// True for orchestrator skills.
    pub fn is_orchestrator(&self) -> bool {
        matches!(self.kind, SkillKind::Orchestrator { .. })
    }
}

/// Split a `--- ... ---` YAML frontmatter block from the body. Returns
/// `(frontmatter, body)` or `None` if there is no leading block.
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n").or_else(|| {
        rest.strip_suffix("\n---").map(|_| rest.len() - "\n---".len())
    })?;
    let fm = &rest[..end];
    let body = rest[end..].trim_start_matches(['\n', '-']).trim_start();
    Some((fm, body))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LENS_MD: &str = "---\nname: code-no-keys\ndescription: no hardcoded secrets\n---\nFlag hardcoded API keys, tokens, and passwords.";
    const LENS_RULES: &str =
        "- id: hardcoded-key\n  message: no hardcoded secrets\n  severity: error\n";

    #[test]
    fn parses_a_lens() {
        let s = Skill::parse("code-no-keys", LENS_MD, Some(LENS_RULES)).unwrap();
        assert_eq!(s.name, "code-no-keys");
        assert_eq!(s.description, "no hardcoded secrets");
        assert!(s.guidance.starts_with("Flag hardcoded"));
        assert!(!s.is_orchestrator());
        match &s.kind {
            SkillKind::Lens { rules } => {
                assert_eq!(rules[0].id, "hardcoded-key");
                assert_eq!(rules[0].severity.as_deref(), Some("error"));
            }
            _ => panic!("expected lens"),
        }
    }

    #[test]
    fn parses_an_orchestrator() {
        let md = "---\nname: code-review\ndescription: run all lenses\nruns:\n  - code-kiss\n  - code-dry\n---\nAggregate all lens findings into one verdict.";
        let s = Skill::parse("code-review", md, None).unwrap();
        assert!(s.is_orchestrator());
        match s.kind {
            SkillKind::Orchestrator { runs } => assert_eq!(runs, vec!["code-kiss", "code-dry"]),
            _ => panic!("expected orchestrator"),
        }
    }

    #[test]
    fn lens_without_rules_is_rejected() {
        assert!(matches!(
            Skill::parse("x", LENS_MD, None),
            Err(SkillError::MissingRules(_))
        ));
    }

    #[test]
    fn missing_frontmatter_is_rejected() {
        assert!(matches!(
            Skill::parse("x", "no frontmatter here", Some(LENS_RULES)),
            Err(SkillError::MissingFrontmatter(_))
        ));
    }

    #[test]
    fn malformed_rules_are_rejected() {
        assert!(matches!(
            Skill::parse("x", LENS_MD, Some("- id: [broken")),
            Err(SkillError::Rules(_, _))
        ));
    }
}
