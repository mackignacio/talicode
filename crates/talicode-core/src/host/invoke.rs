// SPDX-License-Identifier: MIT
//! Skill invocation — expand the selection, compose lens guidance, run the
//! Auditor, and aggregate one verdict.
//!
//! Implements #13. An orchestrator skill (e.g. `code-review`) is expanded to
//! its `runs:` lenses; lenses pass through. The resolved lenses' guidance and
//! rules are composed into the Auditor's prompt, findings are de-duplicated by
//! (file, line, rule), and the run is APPROVED only when no findings remain.

use crate::auditor::{audit, AuditRequest};
use crate::host::discover::{Catalog, DiscoverError};
use crate::host::retrieve;
use crate::host::skill::{Skill, SkillKind};
use crate::provider::{Provider, ProviderError, Usage};
use crate::report::Finding;
use std::collections::BTreeSet;

/// Memory-aware options for an invocation (all default ⇒ pre-memory behavior:
/// every selected lens runs and no memory context is injected).
#[derive(Debug, Clone, Default)]
pub struct InvokeOptions {
    /// Assembled long-term memory context injected into the Auditor prompt.
    pub memory: String,
    /// `"search"` ⇒ native skill retrieval; anything else ⇒ run every lens.
    pub retrieval: String,
    /// Max skills the search injects per file.
    pub skill_limit: usize,
    /// Skills always injected regardless of the search (the security floor).
    pub always_run: Vec<String>,
}

/// The result of invoking the selected skills over one file.
#[derive(Debug, Clone)]
pub struct SweepOutcome {
    /// De-duplicated findings.
    pub findings: Vec<Finding>,
    /// Token usage for the audit.
    pub usage: Usage,
    /// True when no findings remain (the aggregated verdict).
    pub approved: bool,
}

/// Errors from invoking skills.
#[derive(Debug, thiserror::Error)]
pub enum InvokeError {
    /// A selected/`runs:` skill name isn't in the catalog.
    #[error(transparent)]
    Discover(#[from] DiscoverError),
    /// The provider call failed.
    #[error(transparent)]
    Provider(#[from] ProviderError),
}

/// Expand a selection to the concrete lenses to run. Orchestrators expand to
/// their `runs:` lenses (one level); lenses pass through. Order-preserving and
/// de-duplicated by name.
pub fn expand<'a>(
    catalog: &'a Catalog,
    selection: &[String],
) -> Result<Vec<&'a Skill>, DiscoverError> {
    let mut out: Vec<&Skill> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for name in selection {
        let discovered = catalog
            .get(name)
            .ok_or_else(|| DiscoverError::Unknown(name.clone()))?;
        match &discovered.skill.kind {
            SkillKind::Orchestrator { runs } => {
                for lens_name in runs {
                    let lens = catalog
                        .get(lens_name)
                        .ok_or_else(|| DiscoverError::Unknown(lens_name.clone()))?;
                    push_unique(&mut out, &mut seen, &lens.skill);
                }
            }
            SkillKind::Lens { .. } => push_unique(&mut out, &mut seen, &discovered.skill),
        }
    }
    Ok(out)
}

fn push_unique<'a>(out: &mut Vec<&'a Skill>, seen: &mut BTreeSet<String>, skill: &'a Skill) {
    if seen.insert(skill.name.clone()) {
        out.push(skill);
    }
}

/// Compose the resolved lenses' guidance + rules into one prompt fragment.
pub fn compose_guidance(lenses: &[&Skill]) -> String {
    let mut s = String::new();
    for lens in lenses {
        s.push_str(&format!(
            "## {} — {}\n{}\n",
            lens.name, lens.description, lens.guidance
        ));
        if let SkillKind::Lens { rules } = &lens.kind {
            for rule in rules {
                s.push_str(&format!(
                    "- rule `{}` [{}]: {}\n",
                    rule.id,
                    rule.severity.as_deref().unwrap_or("warning"),
                    rule.message
                ));
            }
        }
        s.push('\n');
    }
    s
}

/// Invoke the selected skills over one file's content.
#[allow(clippy::too_many_arguments)]
pub async fn invoke_file(
    provider: &dyn Provider,
    catalog: &Catalog,
    selection: &[String],
    file: &str,
    content: &str,
    model: &str,
    effort: &str,
    role: &str,
    opts: &InvokeOptions,
) -> Result<SweepOutcome, InvokeError> {
    let lenses = expand(catalog, selection)?;
    // Procedural memory: with "search", inject only the skills the native search
    // matches for this file (plus the always-run floor); otherwise run them all.
    let chosen: Vec<&Skill> = if opts.retrieval == "search" {
        retrieve::retrieve(&lenses, content, opts.skill_limit, &opts.always_run)
    } else {
        lenses
    };
    let guidance = compose_guidance(&chosen);
    let outcome = audit(
        provider,
        AuditRequest {
            file: file.to_string(),
            content: content.to_string(),
            model: model.to_string(),
            effort: effort.to_string(),
            role: role.to_string(),
            guidance,
            memory: opts.memory.clone(),
        },
    )
    .await?;

    let findings = dedup(outcome.findings);
    Ok(SweepOutcome {
        approved: findings.is_empty(),
        findings,
        usage: outcome.usage,
    })
}

/// Invoke the selected skills over many files, aggregating findings and usage.
/// `sources` is `(path, content)` pairs.
#[allow(clippy::too_many_arguments)]
pub async fn invoke_files(
    provider: &dyn Provider,
    catalog: &Catalog,
    selection: &[String],
    sources: &[(String, String)],
    model: &str,
    effort: &str,
    role: &str,
    opts: &InvokeOptions,
) -> Result<SweepOutcome, InvokeError> {
    let mut findings = Vec::new();
    let mut usage = Usage::default();
    for (path, content) in sources {
        let outcome = invoke_file(
            provider, catalog, selection, path, content, model, effort, role, opts,
        )
        .await?;
        findings.extend(outcome.findings);
        usage += outcome.usage;
    }
    let findings = dedup(findings);
    Ok(SweepOutcome {
        approved: findings.is_empty(),
        findings,
        usage,
    })
}

/// De-duplicate findings by (file, line, rule) — keeps the first of each.
fn dedup(findings: Vec<Finding>) -> Vec<Finding> {
    let mut seen: BTreeSet<(String, u32, String)> = BTreeSet::new();
    findings
        .into_iter()
        .filter(|f| seen.insert((f.file.clone(), f.line, f.rule.clone())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::FakeProvider;
    use crate::report::Severity;
    use serde_json::json;
    use std::path::Path;

    fn catalog() -> Catalog {
        Catalog::discover(Path::new("/nonexistent-repo")).unwrap()
    }

    #[test]
    fn code_review_expands_to_all_default_lenses_without_aviation() {
        let cat = catalog();
        let lenses = expand(&cat, &["code-review".to_string()]).unwrap();
        let names: Vec<_> = lenses.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names.len(), 20);
        assert!(!names.contains(&"code-aviation"));
        assert!(names.contains(&"code-clear-exit"));
    }

    #[test]
    fn clear_exit_and_early_return_compose() {
        let cat = catalog();
        let lenses = expand(
            &cat,
            &[
                "code-clear-exit".to_string(),
                "code-early-return".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(lenses.len(), 2);
    }

    #[test]
    fn expand_errors_on_unknown_skill() {
        let cat = catalog();
        assert!(matches!(
            expand(&cat, &["ghost".to_string()]),
            Err(DiscoverError::Unknown(_))
        ));
    }

    #[test]
    fn compose_guidance_includes_rule_ids() {
        let cat = catalog();
        let lenses = expand(&cat, &["code-no-keys".to_string()]).unwrap();
        let g = compose_guidance(&lenses);
        assert!(g.contains("code-no-keys"));
        assert!(g.contains("rule `hardcoded-secret`"));
    }

    #[test]
    fn dedup_collapses_same_file_line_rule() {
        let f = |line, rule: &str| Finding {
            file: "a.rs".into(),
            line,
            severity: Severity::Error,
            rule: rule.into(),
            message: "m".into(),
        };
        let out = dedup(vec![f(1, "r"), f(1, "r"), f(2, "r"), f(1, "s")]);
        assert_eq!(out.len(), 3);
    }

    #[tokio::test]
    async fn invoke_file_aggregates_and_sets_verdict() {
        let cat = catalog();
        let provider = FakeProvider::new(
            json!({"findings": [{"line": 3, "severity": "error", "rule": "hardcoded-secret", "message": "key"}]}),
            Usage::default(),
        );
        let outcome = invoke_file(
            &provider,
            &cat,
            &["code-no-keys".to_string()],
            "src/lib.rs",
            "let k = \"sk-123\";",
            "claude-sonnet-5",
            "medium",
            "auditor",
            &InvokeOptions::default(),
        )
        .await
        .unwrap();
        assert_eq!(outcome.findings.len(), 1);
        assert!(!outcome.approved);
        assert_eq!(outcome.findings[0].file, "src/lib.rs");
    }

    #[tokio::test]
    async fn invoke_files_aggregates_across_sources_and_sums_usage() {
        let cat = catalog();
        let provider = FakeProvider::new(
            json!({"findings": [{"line": 1, "severity": "warning", "rule": "magic-number", "message": "n"}]}),
            Usage {
                input_tokens: 5,
                output_tokens: 1,
                ..Default::default()
            },
        );
        let sources = vec![
            ("a.rs".to_string(), "let a = 42;".to_string()),
            ("b.rs".to_string(), "let b = 99;".to_string()),
        ];
        let outcome = invoke_files(
            &provider,
            &cat,
            &["code-magic-numbers".to_string()],
            &sources,
            "claude-sonnet-5",
            "medium",
            "auditor",
            &InvokeOptions::default(),
        )
        .await
        .unwrap();
        // one finding per file (different files ⇒ not de-duped), usage summed
        assert_eq!(outcome.findings.len(), 2);
        assert_eq!(outcome.usage.input_tokens, 10);
        assert!(!outcome.approved);
    }

    #[tokio::test]
    async fn clean_file_is_approved() {
        let cat = catalog();
        let provider = FakeProvider::new(json!({"findings": []}), Usage::default());
        let outcome = invoke_file(
            &provider,
            &cat,
            &["code-review".to_string()],
            "src/lib.rs",
            "fn ok() {}",
            "claude-sonnet-5",
            "medium",
            "auditor",
            &InvokeOptions::default(),
        )
        .await
        .unwrap();
        assert!(outcome.approved);
        assert!(outcome.findings.is_empty());
    }
}
