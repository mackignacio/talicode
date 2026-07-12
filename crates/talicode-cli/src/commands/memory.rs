// SPDX-License-Identifier: MIT
//! `tali memory` — manage semantic facts and episodic experiences.
//!
//! Implements #52. Semantic: `add`/`list`/`search`/`forget` durable facts.
//! Episodic: `remember`/`recall`/`timeline`/`supersede`/`prune` — and recording
//! a recurring `experience` auto-promotes it to a skill (episodic → procedural).
//! The renderers are pure and unit-tested; `run` does the IO.

use anyhow::{anyhow, Context};
use clap::{Args as ClapArgs, Subcommand};
use talicode_core::config::{Config, MemoryConfig};
use talicode_core::usage;
use talicode_memory::episode::{self, Episode, MemoryType};
use talicode_memory::memory::{self, Memory, Tier};
use talicode_skills::host::discover::Catalog;

#[derive(ClapArgs, Debug)]
pub struct Args {
    #[command(subcommand)]
    cmd: MemoryCmd,
}

#[derive(Subcommand, Debug)]
enum MemoryCmd {
    /// Add a durable semantic fact.
    Add {
        /// The fact text.
        text: String,
        /// Tags (repeatable).
        #[arg(long)]
        tag: Vec<String>,
    },
    /// List semantic facts.
    List {
        #[arg(long)]
        json: bool,
    },
    /// Search semantic facts by keyword.
    Search {
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Forget a semantic fact by slug.
    Forget { slug: String },
    /// Record an episodic experience.
    Remember {
        /// One of: learning | mistake | experience | summary.
        #[arg(long = "type", value_name = "TYPE")]
        kind: String,
        /// The experience text.
        text: String,
        /// Tags (repeatable).
        #[arg(long)]
        tag: Vec<String>,
        /// Store as a transient scratch entry (auto-expires).
        #[arg(long)]
        scratch: bool,
    },
    /// Recall past episodes by query.
    Recall {
        query: String,
        #[arg(long)]
        json: bool,
    },
    /// Show recent episodic history.
    Timeline {
        #[arg(long)]
        json: bool,
    },
    /// Replace an episode with new text, linking the old one.
    Supersede { id: u64, text: String },
    /// Prune expired scratch episodes (dry-run unless `--apply`).
    Prune {
        #[arg(long)]
        apply: bool,
    },
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    match args.cmd {
        MemoryCmd::Add { text, tag } => {
            let m = memory::store(&root, &text, tag)?;
            println!("added semantic fact `{}`", m.slug);
        }
        MemoryCmd::List { json } => {
            let mems = memory::read(&root);
            emit(json, render_mems_json(&mems), render_mems_human(&mems));
        }
        MemoryCmd::Search { query, json } => {
            let mems = memory::read(&root);
            let hits: Vec<Memory> = memory::rank(&mems, &query, usage::local_today_naive(), 20)
                .into_iter()
                .cloned()
                .collect();
            emit(json, render_mems_json(&hits), render_mems_human(&hits));
        }
        MemoryCmd::Forget { slug } => {
            let removed = memory::forget(&root, &slug)?;
            println!(
                "{}",
                if removed {
                    format!("forgot `{slug}`")
                } else {
                    format!("no fact `{slug}`")
                }
            );
        }
        MemoryCmd::Remember {
            kind,
            text,
            tag,
            scratch,
        } => {
            let mut ep = Episode::new(parse_kind(&kind)?, &text, tag);
            let cfg = load_memory_config(&root);
            if scratch {
                ep.tier = Tier::Scratch;
                ep.expires_at = Some(episode::scratch_expiry(cfg.episodic_scratch_ttl_hours));
            }
            let stored = episode::record(&root, ep)?;
            println!("remembered episode #{} ({kind})", stored.id);
            maybe_promote(&root, &cfg);
        }
        MemoryCmd::Recall { query, json } => {
            let eps = episode::read(&root);
            let hits: Vec<Episode> = episode::rank(
                &eps,
                &query,
                usage::local_today_naive(),
                Default::default(),
                20,
            )
            .into_iter()
            .cloned()
            .collect();
            emit(json, render_eps_json(&hits), render_eps_human(&hits));
        }
        MemoryCmd::Timeline { json } => {
            let mut eps = episode::read(&root);
            eps.sort_by_key(|e| std::cmp::Reverse(e.id));
            emit(json, render_eps_json(&eps), render_eps_human(&eps));
        }
        MemoryCmd::Supersede { id, text } => {
            let old = episode::read(&root)
                .into_iter()
                .find(|e| e.id == id)
                .ok_or_else(|| anyhow!("no episode #{id}"))?;
            let new = episode::supersede(&root, id, old.memory_type, &text, old.tags)?;
            println!("episode #{} supersedes #{id}", new.id);
        }
        MemoryCmd::Prune { apply } => {
            let pruned = episode::prune(&root, usage::local_today_naive(), apply)?;
            let verb = if apply { "pruned" } else { "would prune" };
            println!("{verb} {} expired scratch episode(s)", pruned.len());
        }
    }
    Ok(())
}

/// Auto-promote recurring experiences to skills (episodic → procedural), unless
/// disabled. Best-effort — never fails the command.
fn maybe_promote(root: &std::path::Path, cfg: &MemoryConfig) {
    if !cfg.auto_promote_skills {
        return;
    }
    let existing: Vec<String> = Catalog::discover(root)
        .map(|c| c.all().map(|d| d.skill.name.clone()).collect())
        .unwrap_or_default();
    let eps = episode::read(root);
    for draft in episode::due_for_skill(&eps, cfg.experience_to_skill_threshold, &existing) {
        if episode::promote(root, &draft).is_ok() {
            println!("promoted experience → skills/{}/", draft.slug);
        }
    }
}

fn load_memory_config(root: &std::path::Path) -> MemoryConfig {
    Config::load(root).map(|c| c.memory).unwrap_or_default()
}

fn parse_kind(kind: &str) -> anyhow::Result<MemoryType> {
    match kind {
        "learning" => Ok(MemoryType::Learning),
        "mistake" => Ok(MemoryType::Mistake),
        "experience" => Ok(MemoryType::Experience),
        "summary" => Ok(MemoryType::Summary),
        other => Err(anyhow!(
            "unknown --type `{other}` (expected learning|mistake|experience|summary)"
        )),
    }
    .context("parsing --type")
}

fn emit(json: bool, as_json: String, as_human: String) {
    if json {
        println!("{as_json}");
    } else {
        print!("{as_human}");
    }
}

fn render_mems_human(mems: &[Memory]) -> String {
    if mems.is_empty() {
        return "No semantic facts yet.\n".to_string();
    }
    let mut out = String::new();
    for m in mems {
        out.push_str(&format!("  {:<28} {}\n", m.slug, m.body.replace('\n', " ")));
    }
    out
}

fn render_mems_json(mems: &[Memory]) -> String {
    let rows: Vec<_> = mems
        .iter()
        .map(|m| serde_json::json!({"slug": m.slug, "created": m.created, "tags": m.tags, "body": m.body}))
        .collect();
    serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".to_string())
}

fn render_eps_human(eps: &[Episode]) -> String {
    if eps.is_empty() {
        return "No episodes yet.\n".to_string();
    }
    let mut out = String::new();
    for e in eps {
        out.push_str(&format!(
            "  #{:<4} {:<11} {}  {}\n",
            e.id,
            format!("{:?}", e.memory_type).to_lowercase(),
            e.created,
            e.content.replace('\n', " ")
        ));
    }
    out
}

fn render_eps_json(eps: &[Episode]) -> String {
    serde_json::to_string_pretty(eps).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ep(id: u64, kind: MemoryType, content: &str) -> Episode {
        let mut e = Episode::new(kind, content, vec![]);
        e.id = id;
        e.created = "2026-07-12".into();
        e
    }

    #[test]
    fn parse_kind_maps_known_types_and_rejects_others() {
        assert_eq!(parse_kind("learning").unwrap(), MemoryType::Learning);
        assert_eq!(parse_kind("mistake").unwrap(), MemoryType::Mistake);
        assert!(parse_kind("nope").is_err());
    }

    #[test]
    fn render_mems_human_lists_or_reports_empty() {
        assert!(render_mems_human(&[]).contains("No semantic facts"));
        let mems = vec![Memory {
            slug: "raw-sql".into(),
            tier: Tier::Durable,
            created: "2026-07-12".into(),
            tags: vec![],
            body: "uses raw SQL".into(),
        }];
        let out = render_mems_human(&mems);
        assert!(out.contains("raw-sql"));
        assert!(out.contains("uses raw SQL"));
    }

    #[test]
    fn render_eps_human_shows_type_and_content() {
        let out = render_eps_human(&[ep(3, MemoryType::Learning, "use alias+version")]);
        assert!(out.contains("#3"));
        assert!(out.contains("learning"));
        assert!(out.contains("use alias+version"));
        assert!(render_eps_human(&[]).contains("No episodes"));
    }

    #[test]
    fn render_eps_json_is_valid_json() {
        let j = render_eps_json(&[ep(1, MemoryType::Mistake, "oops")]);
        assert!(j.contains("\"mistake\""));
        assert!(j.contains("oops"));
    }
}
