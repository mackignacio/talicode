// SPDX-License-Identifier: MIT
//! Findings and their severities.
//!
//! Implements #10 (the types the Auditor produces). Terminal rendering and the
//! gating exit-code decision are added in #21.

use serde::{Deserialize, Serialize};

/// How serious a finding is. Deserializes leniently: the model is asked for
/// `info`/`warning`/`error`, but common synonyms map to the same variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}
