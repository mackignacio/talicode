// SPDX-License-Identifier: MIT
//! Findings and their severities.
//!
//! Implements #10 (the types the Auditor produces) and #21 (terminal rendering
//! and the gating exit-code decision).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How serious a finding is. Deserializes leniently: the model is asked for
/// `info`/`warning`/`error`, but common synonyms map to the same variant.
///
/// Variant order is significant: `Info < Warning < Error` (used by the gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Informational — a nit or suggestion.
    #[serde(alias = "low", alias = "note", alias = "hint")]
    Info,
    /// Warning — likely a problem.
    #[serde(alias = "medium", alias = "warn")]
    Warning,
    /// Error — a real defect.
    #[serde(alias = "high", alias = "critical", alias = "blocker")]
    Error,
}

/// A single line-anchored finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    /// Repo-relative path of the file the finding is in. Set by the Auditor
    /// from the audited file, so it defaults when the model omits it.
    #[serde(default)]
    pub file: String,
    /// 1-indexed line the finding anchors to.
    pub line: u32,
    /// Severity.
    pub severity: Severity,
    /// Originating skill/rule id (e.g. `code-no-keys`).
    pub rule: String,
    /// One-line description of the defect.
    pub message: String,
}

impl Severity {
    /// Lowercase label for display (`info`/`warning`/`error`).
    pub fn label(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

/// Findings at or above this severity cause a non-zero exit (so a future
/// pre-commit hook can gate on it). Info-level nits alone do not fail.
pub const DEFAULT_GATE: Severity = Severity::Warning;

/// Render findings for a terminal, grouped by file. Paths + lines are clickable
/// as `file:line`.
pub fn render_human(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "No findings.\n".to_string();
    }
    let mut by_file: BTreeMap<&str, Vec<&Finding>> = BTreeMap::new();
    for f in findings {
        by_file.entry(f.file.as_str()).or_default().push(f);
    }
    let mut out = String::new();
    for group in by_file.values() {
        for f in group {
            out.push_str(&format!(
                "{}:{} {} {} — {}\n",
                f.file,
                f.line,
                f.severity.label(),
                f.rule,
                f.message
            ));
        }
    }
    out.push_str(&format!("\n{} finding(s).\n", findings.len()));
    out
}

/// Render findings as pretty JSON for machine consumption.
pub fn render_json(findings: &[Finding]) -> String {
    serde_json::to_string_pretty(findings).unwrap_or_else(|_| "[]".to_string())
}

/// Exit code for a sweep: non-zero when any finding is at or above `gate`.
pub fn exit_code(findings: &[Finding], gate: Severity) -> i32 {
    i32::from(findings.iter().any(|f| f.severity >= gate))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn finding(file: &str, line: u32, sev: Severity, rule: &str) -> Finding {
        Finding {
            file: file.into(),
            line,
            severity: sev,
            rule: rule.into(),
            message: "m".into(),
        }
    }

    #[test]
    fn severity_parses_canonical_and_synonyms() {
        let e: Severity = serde_json::from_value(json!("error")).unwrap();
        assert_eq!(e, Severity::Error);
        let e2: Severity = serde_json::from_value(json!("critical")).unwrap();
        assert_eq!(e2, Severity::Error);
        let w: Severity = serde_json::from_value(json!("medium")).unwrap();
        assert_eq!(w, Severity::Warning);
    }

    #[test]
    fn finding_file_defaults_when_absent() {
        let f: Finding =
            serde_json::from_value(json!({"line": 3, "severity": "info", "rule": "r", "message": "m"}))
                .unwrap();
        assert_eq!(f.file, "");
        assert_eq!(f.line, 3);
    }

    #[test]
    fn severity_orders_info_lt_warning_lt_error() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn render_human_groups_by_file() {
        let out = render_human(&[
            finding("a.rs", 2, Severity::Error, "code-no-keys"),
            finding("a.rs", 5, Severity::Info, "code-kiss"),
        ]);
        assert!(out.contains("a.rs:2 error code-no-keys — m"));
        assert!(out.contains("a.rs:5 info code-kiss — m"));
        assert!(out.contains("2 finding(s)."));
    }

    #[test]
    fn render_human_empty_is_clean() {
        assert_eq!(render_human(&[]), "No findings.\n");
    }

    #[test]
    fn exit_code_gates_on_threshold() {
        let info = [finding("a", 1, Severity::Info, "r")];
        let err = [finding("a", 1, Severity::Error, "r")];
        assert_eq!(exit_code(&[], DEFAULT_GATE), 0);
        assert_eq!(exit_code(&info, DEFAULT_GATE), 0); // info alone doesn't fail
        assert_eq!(exit_code(&err, DEFAULT_GATE), 1);
        assert_eq!(exit_code(&info, Severity::Info), 1); // stricter gate
    }

    #[test]
    fn render_json_round_trips() {
        let findings = vec![finding("a.rs", 1, Severity::Warning, "r")];
        let json = render_json(&findings);
        let back: Vec<Finding> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, findings);
    }
}
