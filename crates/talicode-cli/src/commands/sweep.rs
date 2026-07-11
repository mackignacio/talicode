// SPDX-License-Identifier: MIT
//! `tali sweep` — detect AI slop / violations in staged or target files.
//!
//! Implements #22. Loads `config.tali`, selects skills and files, reads
//! sources, invokes the skill host over them, renders findings, and exits
//! non-zero when the gate trips. The IO wiring lives here; the detection engine
//! is `talicode_core`. Pure selection helpers are unit-tested.

use anyhow::{anyhow, Context};
use clap::Args as ClapArgs;
use talicode_core::config::{Agent, Config};
use talicode_core::git::{self, SkipReason};
use talicode_core::host::{discover::Catalog, invoke};
use talicode_core::provider;
use talicode_core::report;
use talicode_core::usage;

/// Default target glob when the config's first step declares none.
const DEFAULT_TARGET: &str = "./**/*.rs";
/// Default skill selection when the config lists none.
const DEFAULT_SKILL: &str = "code-review";

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Only audit files staged in git.
    #[arg(long)]
    pub staged: bool,
    /// Run only this skill, overriding the config selection.
    #[arg(long)]
    pub skill: Option<String>,
    /// Emit findings as machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let code = execute(args.staged, args.skill, args.json).await?;
    std::process::exit(code);
}

/// Run the full detect path and render. Returns the process exit code without
/// exiting, so `heal` can reuse it. `#[cfg(test)]` code drives the helpers
/// below; this async entry point is exercised via the binary.
pub async fn execute(staged: bool, skill: Option<String>, json: bool) -> anyhow::Result<i32> {
    let root = std::env::current_dir().context("resolving current directory")?;
    let config = Config::load(&root)?;
    let agent = pick_auditor(&config)?;
    let selection = select_skills(skill.as_deref(), &config.skills);

    let paths = if staged {
        git::staged_files(&root)?
    } else {
        git::target_files(&root, &resolve_target(&config))?
    };
    let (sources, skipped) = git::read_sources(&root, &paths);
    for s in &skipped {
        eprintln!("skipped {} ({})", s.path, skip_label(s.reason));
    }

    let provider = provider::build_provider(&agent.provider)?;
    let catalog = Catalog::discover(&root)?;
    let pairs: Vec<(String, String)> = sources.into_iter().map(|s| (s.path, s.content)).collect();

    let outcome = invoke::invoke_files(
        provider.as_ref(),
        &catalog,
        &selection,
        &pairs,
        &agent.model,
        &agent.effort,
        &agent.role,
    )
    .await?;

    // Record spend to the ledger (best-effort — never blocks a sweep).
    let entry = usage::LedgerEntry::today(outcome.usage, &agent.model, "sweep");
    if let Err(e) = usage::append(&root, &entry) {
        eprintln!("warning: could not write usage ledger: {e}");
    }

    if json {
        println!("{}", report::render_json(&outcome.findings));
    } else {
        print!("{}", report::render_human(&outcome.findings));
        eprintln!("{}", usage::footer(outcome.usage, &agent.model));
        if let Some(today) = usage::roll_up(&usage::read(&root)).get(&usage::local_today()) {
            eprintln!("today: in {} / out {}", today.input, today.output);
        }
    }

    Ok(report::exit_code(&outcome.findings, report::DEFAULT_GATE))
}

/// Choose the auditor agent: the first step's agent, else the first defined agent.
fn pick_auditor(config: &Config) -> anyhow::Result<Agent> {
    if let Some(step) = config.execution_flow.first() {
        return config.agents.get(&step.agent).cloned().ok_or_else(|| {
            anyhow!(
                "step `{}` references undefined agent `{}`",
                step.step,
                step.agent
            )
        });
    }
    config
        .agents
        .values()
        .next()
        .cloned()
        .ok_or_else(|| anyhow!("config.tali defines no agents"))
}

/// `--skill` wins; else the config's `skills:`; else the default orchestrator.
fn select_skills(skill_arg: Option<&str>, config_skills: &[String]) -> Vec<String> {
    match skill_arg {
        Some(s) => vec![s.to_string()],
        None if !config_skills.is_empty() => config_skills.to_vec(),
        None => vec![DEFAULT_SKILL.to_string()],
    }
}

/// The target glob from the first step, or the default.
fn resolve_target(config: &Config) -> String {
    config
        .execution_flow
        .first()
        .and_then(|s| s.target.clone())
        .unwrap_or_else(|| DEFAULT_TARGET.to_string())
}

fn skip_label(reason: SkipReason) -> &'static str {
    match reason {
        SkipReason::TooLarge => "too large",
        SkipReason::Binary => "binary",
        SkipReason::Unreadable => "unreadable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use talicode_core::config::Step;

    fn config_with(agents: &[(&str, &str)], steps: &[(&str, &str, Option<&str>)]) -> Config {
        let mut map = BTreeMap::new();
        for (name, model) in agents {
            map.insert(
                name.to_string(),
                Agent {
                    provider: "anthropic".into(),
                    model: model.to_string(),
                    role: "r".into(),
                    effort: "medium".into(),
                },
            );
        }
        Config {
            version: "1.0".into(),
            name: "t".into(),
            agents: map,
            execution_flow: steps
                .iter()
                .map(|(step, agent, target)| Step {
                    step: step.to_string(),
                    agent: agent.to_string(),
                    target: target.map(String::from),
                })
                .collect(),
            skills: vec![],
        }
    }

    #[test]
    fn select_skills_precedence() {
        assert_eq!(
            select_skills(Some("code-no-keys"), &[]),
            vec!["code-no-keys"]
        );
        assert_eq!(
            select_skills(None, &["code-solid".to_string()]),
            vec!["code-solid"]
        );
        assert_eq!(select_skills(None, &[]), vec![DEFAULT_SKILL]);
    }

    #[test]
    fn resolve_target_uses_step_or_default() {
        let c = config_with(
            &[("auditor", "m")],
            &[("sweep", "auditor", Some("./x/**/*.py"))],
        );
        assert_eq!(resolve_target(&c), "./x/**/*.py");
        let c2 = config_with(&[("auditor", "m")], &[]);
        assert_eq!(resolve_target(&c2), DEFAULT_TARGET);
    }

    #[test]
    fn pick_auditor_follows_step_then_falls_back() {
        let c = config_with(
            &[("auditor", "claude-sonnet-5")],
            &[("sweep", "auditor", None)],
        );
        assert_eq!(pick_auditor(&c).unwrap().model, "claude-sonnet-5");

        let c2 = config_with(&[("only", "m2")], &[]);
        assert_eq!(pick_auditor(&c2).unwrap().model, "m2");

        let bad = config_with(&[("auditor", "m")], &[("sweep", "ghost", None)]);
        assert!(pick_auditor(&bad).is_err());
    }
}
