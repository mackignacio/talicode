// SPDX-License-Identifier: MIT
//! `tali init` — scaffold a `config.tali` + `skills/` in the current repo.
//!
//! Implements #7. Writes a starter `config.tali` (refusing to overwrite an
//! existing one), creates an empty repo-level `skills/` directory, and ensures
//! `.talicode/` (the local usage-ledger dir) is git-ignored.

use anyhow::{bail, Context};
use std::path::Path;
use talicode_core::config::CONFIG_FILENAME;

/// The starter config written by `tali init`. Detect-only: an `auditor` agent
/// plus the bundled `code-review` orchestrator. (Healing agents/fallbacks are
/// roadmap, not the MVP — see docs/roadmaps/ROADMAP-HEAL.md.)
const STARTER_CONFIG: &str = r#"version: "1.0"
name: "TaliCode Local Sweep"

agents:
  auditor:
    provider: "anthropic"
    model: "claude-sonnet-5"
    effort: "medium"
    role: >
      Identify AI slop: hallucinated or unverified imports, dead boilerplate,
      and obvious type/security issues. Report only concrete, line-anchored
      violations; prefer silence over speculation.

execution_flow:
  - step: "slop_sweep"
    agent: "auditor"
    target: "./src/**/*.rs"

# Skills the sweep runs. Empty selects the bundled `code-review` orchestrator
# (all default lenses). List specific skills to narrow the sweep.
skills:
  - code-review

# Long-term memory (all fields optional; omit this block for the defaults).
# memory:
#   skill_retrieval: "search"        # "search" (native, token-saving) or "all"
#   experience_to_skill_threshold: 2 # recurring experiences → auto-created skills
"#;

/// Local-only memory files to git-ignore. Durable semantic memory
/// (`.talicode/memory/`) and the architecture map (`.talicode/architecture.json`)
/// are intentionally committed so the team shares the Auditor's context.
const GITIGNORE_LINES: [&str; 2] = [".talicode/usage.jsonl", ".talicode/episodes.jsonl"];

pub fn run() -> anyhow::Result<()> {
    let root = std::env::current_dir().context("cannot resolve the current directory")?;
    scaffold(&root)?;
    println!(
        "Initialized TaliCode: wrote {CONFIG_FILENAME}, created skills/, git-ignored the local usage + episode logs"
    );
    Ok(())
}

/// Scaffold TaliCode into `root`. Idempotent for `skills/` and `.gitignore`;
/// refuses to overwrite an existing `config.tali`.
fn scaffold(root: &Path) -> anyhow::Result<()> {
    let config_path = root.join(CONFIG_FILENAME);
    if config_path.exists() {
        bail!(
            "{CONFIG_FILENAME} already exists at {} — refusing to overwrite",
            config_path.display()
        );
    }
    std::fs::write(&config_path, STARTER_CONFIG)
        .with_context(|| format!("writing {}", config_path.display()))?;

    std::fs::create_dir_all(root.join("skills"))
        .with_context(|| format!("creating {}/skills", root.display()))?;

    ensure_gitignored(root)?;
    Ok(())
}

/// Append the local-only memory files to the repo `.gitignore` (idempotently).
fn ensure_gitignored(root: &Path) -> anyhow::Result<()> {
    let path = root.join(".gitignore");
    let mut updated = std::fs::read_to_string(&path).unwrap_or_default();
    let mut changed = false;
    for line in GITIGNORE_LINES {
        if updated.lines().any(|l| l.trim() == line) {
            continue;
        }
        if !updated.is_empty() && !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(line);
        updated.push('\n');
        changed = true;
    }
    if changed {
        std::fs::write(&path, updated).with_context(|| format!("updating {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use talicode_core::config::Config;

    #[test]
    fn scaffold_writes_a_valid_config_and_skills_dir() {
        let dir = tempfile::tempdir().unwrap();
        scaffold(dir.path()).unwrap();

        let cfg = std::fs::read_to_string(dir.path().join(CONFIG_FILENAME)).unwrap();
        Config::parse(&cfg).expect("starter config must be valid");
        assert!(dir.path().join("skills").is_dir());
    }

    #[test]
    fn scaffold_refuses_to_overwrite_existing_config() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(CONFIG_FILENAME),
            "version: \"9\"\nname: keep\n",
        )
        .unwrap();

        let err = scaffold(dir.path()).unwrap_err();
        assert!(err.to_string().contains("refusing to overwrite"));
        // original content is untouched
        let kept = std::fs::read_to_string(dir.path().join(CONFIG_FILENAME)).unwrap();
        assert!(kept.contains("keep"));
    }

    #[test]
    fn gitignore_gets_local_memory_lines_without_duplicating() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".gitignore"), "/target\n").unwrap();

        scaffold(dir.path()).unwrap();
        let gi = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        for line in GITIGNORE_LINES {
            assert_eq!(gi.matches(line).count(), 1, "{line} present once");
        }
        assert!(gi.contains("/target"));
        // Durable memory + the architecture map are NOT ignored (team-shared).
        assert!(!gi.contains(".talicode/memory/"));
        assert!(!gi.contains("architecture.json"));

        // second scaffold in a fresh config-less dir is idempotent on the lines
        std::fs::remove_file(dir.path().join(CONFIG_FILENAME)).unwrap();
        scaffold(dir.path()).unwrap();
        let gi2 = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        for line in GITIGNORE_LINES {
            assert_eq!(gi2.matches(line).count(), 1);
        }
    }
}
