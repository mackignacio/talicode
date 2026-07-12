// SPDX-License-Identifier: MIT
//! TaliCode Test — universal test orchestration (skeleton).
//!
//! The foundation for the roadmap module (see `docs/roadmaps/ROADMAP-TEST.md`):
//! TaliCode detects the stack **from the file itself** and drives that stack's
//! native quality gate, normalizing failures into the same [`Finding`] model as
//! `tali sweep`. Running/parsing real suites and wiring the `tali test`
//! subcommand come later; the pure, unit-tested core lives here:
//!
//! - [`detect`] — classify a file into an [`Adapter`] from its extension, with a
//!   content-signature fallback (no manual selection).
//! - [`gate`] — the ordered [`GateStep`]s a stack must pass (all green).
//! - [`failure_finding`] — normalize a failed step into a core [`Finding`].

use serde::{Deserialize, Serialize};
use talicode_core::report::{Finding, Severity};

/// A test/quality-gate stack TaliCode Test can drive. Extensible — each new
/// stack is one more adapter, not a core change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Adapter {
    /// Python (ruff / flake8 / pylint / pytest).
    Python,
    /// Node / TypeScript (also `.js`).
    TypeScript,
    /// Go.
    Go,
    /// Rust.
    Rust,
    /// Terraform / HCL.
    Terraform,
    /// Swift (iOS).
    Swift,
    /// Kotlin (Android).
    Kotlin,
    /// Dart (Flutter).
    Dart,
    /// Ruby.
    Ruby,
    /// PHP.
    Php,
}

impl Adapter {
    /// Stable kebab-case name — matches the `--adapter` flag and `config.tali`.
    pub fn name(self) -> &'static str {
        match self {
            Adapter::Python => "python",
            Adapter::TypeScript => "typescript",
            Adapter::Go => "go",
            Adapter::Rust => "rust",
            Adapter::Terraform => "terraform",
            Adapter::Swift => "swift",
            Adapter::Kotlin => "kotlin",
            Adapter::Dart => "dart",
            Adapter::Ruby => "ruby",
            Adapter::Php => "php",
        }
    }
}

/// The lowercase file extension of `path` (no dot), or `None` if it has none.
fn extension(path: &str) -> Option<String> {
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    let (stem, ext) = name.rsplit_once('.')?;
    if stem.is_empty() {
        return None; // dotfile like `.gitignore` — not an extension
    }
    Some(ext.to_ascii_lowercase())
}

/// Detect the adapter for a file from its extension alone.
pub fn detect_by_extension(path: &str) -> Option<Adapter> {
    match extension(path)?.as_str() {
        "py" | "pyw" | "pyi" => Some(Adapter::Python),
        "ts" | "tsx" | "mts" | "cts" | "js" | "jsx" | "mjs" | "cjs" => Some(Adapter::TypeScript),
        "go" => Some(Adapter::Go),
        "rs" => Some(Adapter::Rust),
        "tf" | "tfvars" => Some(Adapter::Terraform),
        "swift" => Some(Adapter::Swift),
        "kt" | "kts" => Some(Adapter::Kotlin),
        "dart" => Some(Adapter::Dart),
        "rb" => Some(Adapter::Ruby),
        "php" => Some(Adapter::Php),
        _ => None,
    }
}

/// Detect from a leading shebang when the extension is missing/unknown.
fn detect_by_content(contents: &str) -> Option<Adapter> {
    let first = contents.lines().next().unwrap_or("");
    if !first.starts_with("#!") {
        return None;
    }
    if first.contains("python") {
        Some(Adapter::Python)
    } else if first.contains("ruby") {
        Some(Adapter::Ruby)
    } else if first.contains("node") {
        Some(Adapter::TypeScript)
    } else {
        None
    }
}

/// Classify a file into its adapter — extension first, then a content-signature
/// fallback — so the right suite is chosen automatically. `None` when no adapter
/// recognizes the file.
pub fn detect(path: &str, contents: &str) -> Option<Adapter> {
    detect_by_extension(path).or_else(|| detect_by_content(contents))
}

/// One step in a stack's quality gate: a command that must exit zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateStep {
    /// Short id for the step (e.g. `pylint`).
    pub name: String,
    /// The command + args to run.
    pub command: Vec<String>,
}

impl GateStep {
    fn new(name: &str, command: &[&str]) -> Self {
        GateStep {
            name: name.to_string(),
            command: command.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// The ordered quality gate for an adapter — every step must be green. The
/// commands are defaults; a project overrides them in the `test:` block of
/// `config.tali`. Adapters whose gate isn't defined yet return an empty vec.
pub fn gate(adapter: Adapter) -> Vec<GateStep> {
    match adapter {
        // The reference gate: formatted + lint-clean + a perfect pylint score +
        // passing tests. Any non-zero exit (or pylint below 10.00) fails.
        Adapter::Python => vec![
            GateStep::new("format", &["ruff", "format", "--check"]),
            GateStep::new("lint", &["ruff", "check"]),
            GateStep::new("flake8", &["flake8"]),
            GateStep::new("pylint", &["pylint", "--fail-under=10.0"]),
            GateStep::new("pytest", &["pytest"]),
        ],
        _ => Vec::new(),
    }
}

/// Normalize a failed gate step into a core [`Finding`], so test failures gate a
/// commit exactly like sweep findings.
pub fn failure_finding(adapter: Adapter, step: &GateStep, file: &str) -> Finding {
    Finding {
        file: file.to_string(),
        line: 1,
        severity: Severity::Error,
        rule: format!("test-{}:{}", adapter.name(), step.name),
        message: format!("{} gate step '{}' failed", adapter.name(), step.name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_common_extensions() {
        assert_eq!(detect_by_extension("src/app.py"), Some(Adapter::Python));
        assert_eq!(detect_by_extension("web/x.tsx"), Some(Adapter::TypeScript));
        assert_eq!(detect_by_extension("main.go"), Some(Adapter::Go));
        assert_eq!(detect_by_extension("lib.rs"), Some(Adapter::Rust));
        assert_eq!(
            detect_by_extension("infra/main.tf"),
            Some(Adapter::Terraform)
        );
        assert_eq!(detect_by_extension("App.swift"), Some(Adapter::Swift));
        assert_eq!(detect_by_extension("Main.kt"), Some(Adapter::Kotlin));
        assert_eq!(detect_by_extension("main.dart"), Some(Adapter::Dart));
        assert_eq!(detect_by_extension("spec.rb"), Some(Adapter::Ruby));
        assert_eq!(detect_by_extension("index.php"), Some(Adapter::Php));
    }

    #[test]
    fn unknown_and_dotfiles_are_not_detected() {
        assert_eq!(detect_by_extension("notes.txt"), None);
        assert_eq!(detect_by_extension("README"), None);
        assert_eq!(detect_by_extension(".gitignore"), None);
    }

    #[test]
    fn falls_back_to_shebang_when_extension_is_missing() {
        assert_eq!(
            detect("scripts/deploy", "#!/usr/bin/env python3\n..."),
            Some(Adapter::Python)
        );
        assert_eq!(detect("run", "#!/usr/bin/env ruby"), Some(Adapter::Ruby));
        assert_eq!(
            detect("tool", "#!/usr/bin/env node"),
            Some(Adapter::TypeScript)
        );
        assert_eq!(detect("plain", "no shebang here"), None);
    }

    #[test]
    fn extension_wins_over_content() {
        // A .py file with a node shebang is still Python (extension is stronger).
        assert_eq!(detect("a.py", "#!/usr/bin/env node"), Some(Adapter::Python));
    }

    #[test]
    fn python_gate_is_the_strict_quality_gate() {
        let g = gate(Adapter::Python);
        let names: Vec<&str> = g.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["format", "lint", "flake8", "pylint", "pytest"]);
        assert_eq!(g[0].command, ["ruff", "format", "--check"]);
        assert!(g
            .iter()
            .any(|s| s.command.contains(&"--fail-under=10.0".to_string())));
    }

    #[test]
    fn undefined_gates_are_empty_for_now() {
        assert!(gate(Adapter::Go).is_empty());
    }

    #[test]
    fn failure_normalizes_to_an_error_finding() {
        let step = GateStep::new("pylint", &["pylint", "--fail-under=10.0"]);
        let f = failure_finding(Adapter::Python, &step, "src/app.py");
        assert_eq!(f.severity, Severity::Error);
        assert_eq!(f.rule, "test-python:pylint");
        assert_eq!(f.file, "src/app.py");
        assert!(f.message.contains("pylint"));
    }

    #[test]
    fn adapter_names_are_kebab_case() {
        assert_eq!(Adapter::Python.name(), "python");
        assert_eq!(Adapter::TypeScript.name(), "typescript");
    }
}
