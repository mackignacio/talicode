// SPDX-License-Identifier: MIT
//! `tali skills` — list user-authored repo skills (bundled hidden by default).
//!
//! Implements #27. By default this lists only the skills a user authored under
//! the repo's `skills/` directory — the bundled `code-*` defaults are the
//! built-in harness, not the user's work, so they're hidden. `--all` reveals
//! them, labeled by source. The rendering is a pure function so it is
//! unit-tested; `run` does the discovery IO.

use clap::Args as ClapArgs;
use talicode_core::host::discover::{Catalog, Discovered, Source};

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Include the bundled `code-*` defaults, labeled by source.
    #[arg(long)]
    pub all: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let catalog = Catalog::discover(&root)?;
    let mut rows: Vec<&Discovered> = if args.all {
        catalog.all().collect()
    } else {
        catalog.repo_skills().collect()
    };
    rows.sort_by(|a, b| a.skill.name.cmp(&b.skill.name));
    print!("{}", render(&rows, args.all));
    Ok(())
}

/// Render the skill listing. With `show_source`, each row is tagged
/// `[bundled]`/`[repo]`; otherwise (repo-only view) the source is omitted.
fn render(rows: &[&Discovered], show_source: bool) -> String {
    if rows.is_empty() {
        return if show_source {
            "No skills found.\n".to_string()
        } else {
            "No user skills yet. Author one under skills/, or run with --all to see bundled defaults.\n"
                .to_string()
        };
    }
    let mut out = String::new();
    for d in rows {
        if show_source {
            out.push_str(&format!(
                "  {:<28} {:<9} {}\n",
                d.skill.name,
                source_label(d.source),
                d.skill.description
            ));
        } else {
            out.push_str(&format!("  {:<28} {}\n", d.skill.name, d.skill.description));
        }
    }
    out
}

fn source_label(source: Source) -> &'static str {
    match source {
        Source::Bundled => "[bundled]",
        Source::Repo => "[repo]",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talicode_core::host::skill::{Skill, SkillKind};

    fn discovered(name: &str, desc: &str, source: Source) -> Discovered {
        Discovered {
            skill: Skill {
                name: name.into(),
                description: desc.into(),
                guidance: "g".into(),
                kind: SkillKind::Lens { rules: vec![] },
            },
            source,
        }
    }

    #[test]
    fn repo_only_view_omits_source_and_lists_names() {
        let mine = discovered("my-rule", "my custom lens", Source::Repo);
        let out = render(&[&mine], false);
        assert!(out.contains("my-rule"));
        assert!(out.contains("my custom lens"));
        assert!(!out.contains("[repo]"));
    }

    #[test]
    fn all_view_labels_each_row_by_source() {
        let bundled = discovered("code-kiss", "simplicity", Source::Bundled);
        let mine = discovered("my-rule", "custom", Source::Repo);
        let out = render(&[&bundled, &mine], true);
        assert!(out.contains("code-kiss"));
        assert!(out.contains("[bundled]"));
        assert!(out.contains("my-rule"));
        assert!(out.contains("[repo]"));
    }

    #[test]
    fn empty_repo_view_hints_at_all_flag() {
        let out = render(&[], false);
        assert!(out.contains("No user skills yet"));
        assert!(out.contains("--all"));
    }

    #[test]
    fn empty_all_view_reports_none() {
        assert_eq!(render(&[], true), "No skills found.\n");
    }
}
