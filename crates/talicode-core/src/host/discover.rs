// SPDX-License-Identifier: MIT
//! Skill discovery — the bundled defaults (embedded in the binary) merged with
//! repo-authored skills.
//!
//! Implements #12. The `code-*` defaults under `assets/skills/` are embedded at
//! compile time with `rust-embed`, so a single `tali` binary is self-contained.
//! A repo's on-disk `skills/<name>/` is read at runtime and **overrides** an
//! embedded default of the same name.

use crate::host::skill::{Skill, SkillError};
use rust_embed::RustEmbed;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(RustEmbed)]
#[folder = "assets/skills/"]
struct Bundled;

/// Where a discovered skill came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A bundled default embedded in the binary.
    Bundled,
    /// A skill authored in the repo's `skills/` directory.
    Repo,
}

/// A skill plus where it was found.
#[derive(Debug, Clone)]
pub struct Discovered {
    /// The parsed skill.
    pub skill: Skill,
    /// Bundled default or repo-authored.
    pub source: Source,
}

/// Errors from discovery / selection.
#[derive(Debug, thiserror::Error)]
pub enum DiscoverError {
    /// A skill folder failed to parse.
    #[error(transparent)]
    Skill(#[from] SkillError),
    /// A selected skill name isn't in the catalog.
    #[error("unknown skill `{0}`")]
    Unknown(String),
}

/// The merged set of available skills, keyed by name.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    skills: BTreeMap<String, Discovered>,
}

impl Catalog {
    /// Discover skills: embedded defaults merged with `repo_root/skills/`.
    pub fn discover(repo_root: &Path) -> Result<Self, DiscoverError> {
        Ok(merge(load_bundled()?, load_repo(repo_root)?))
    }

    /// Look up one skill by name.
    pub fn get(&self, name: &str) -> Option<&Discovered> {
        self.skills.get(name)
    }

    /// All discovered skills (bundled + repo), sorted by name.
    pub fn all(&self) -> impl Iterator<Item = &Discovered> {
        self.skills.values()
    }

    /// Only the repo-authored skills (what `tali skills` lists by default).
    pub fn repo_skills(&self) -> impl Iterator<Item = &Discovered> {
        self.skills
            .values()
            .filter(|d| d.source == Source::Repo)
    }

    /// Resolve selected skill names to their skills, erroring on any unknown.
    pub fn resolve<'a>(&'a self, names: &[String]) -> Result<Vec<&'a Skill>, DiscoverError> {
        names
            .iter()
            .map(|n| {
                self.get(n)
                    .map(|d| &d.skill)
                    .ok_or_else(|| DiscoverError::Unknown(n.clone()))
            })
            .collect()
    }

    /// Number of discovered skills.
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Whether the catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

/// Parse every embedded bundled skill.
pub fn load_bundled() -> Result<Vec<Skill>, SkillError> {
    // Group embedded file paths by their top-level directory (the skill name).
    let mut names = std::collections::BTreeSet::new();
    for path in Bundled::iter() {
        if let Some(dir) = path.split('/').next() {
            names.insert(dir.to_string());
        }
    }

    let mut skills = Vec::new();
    for name in names {
        let skill_md = Bundled::get(&format!("{name}/SKILL.md"))
            .map(|f| String::from_utf8_lossy(&f.data).into_owned());
        let Some(skill_md) = skill_md else { continue };
        let rules = Bundled::get(&format!("{name}/rules.yaml"))
            .map(|f| String::from_utf8_lossy(&f.data).into_owned());
        skills.push(Skill::parse(&name, &skill_md, rules.as_deref())?);
    }
    Ok(skills)
}

/// Parse repo-authored skills under `repo_root/skills/`. Missing dir ⇒ none.
fn load_repo(repo_root: &Path) -> Result<Vec<Skill>, SkillError> {
    let dir = repo_root.join("skills");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut skills = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("SKILL.md").is_file() {
            skills.push(Skill::load(&path)?);
        }
    }
    Ok(skills)
}

/// Merge bundled + repo skills; repo overrides bundled by name.
fn merge(bundled: Vec<Skill>, repo: Vec<Skill>) -> Catalog {
    let mut skills = BTreeMap::new();
    for skill in bundled {
        skills.insert(
            skill.name.clone(),
            Discovered {
                skill,
                source: Source::Bundled,
            },
        );
    }
    for skill in repo {
        skills.insert(
            skill.name.clone(),
            Discovered {
                skill,
                source: Source::Repo,
            },
        );
    }
    Catalog { skills }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::skill::SkillKind;

    fn lens(name: &str) -> Skill {
        Skill {
            name: name.into(),
            description: "d".into(),
            guidance: "g".into(),
            kind: SkillKind::Lens { rules: vec![] },
        }
    }

    #[test]
    fn all_bundled_skills_parse() {
        let skills = load_bundled().expect("every bundled skill must parse");
        assert_eq!(skills.len(), 22, "expected 21 lenses + code-review");
        let names: Vec<_> = skills.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"code-review"));
        assert!(names.contains(&"code-aviation"));
        assert!(names.contains(&"code-no-keys"));
        // code-review is the orchestrator; the rest are lenses.
        let review = skills.iter().find(|s| s.name == "code-review").unwrap();
        assert!(review.is_orchestrator());
    }

    #[test]
    fn repo_skill_overrides_bundled_by_name() {
        let cat = merge(vec![lens("code-kiss")], vec![lens("code-kiss")]);
        assert_eq!(cat.get("code-kiss").unwrap().source, Source::Repo);
        assert_eq!(cat.len(), 1);
    }

    #[test]
    fn repo_skills_iterator_excludes_bundled() {
        let cat = merge(vec![lens("code-kiss")], vec![lens("my-rule")]);
        let repo: Vec<_> = cat.repo_skills().map(|d| d.skill.name.as_str()).collect();
        assert_eq!(repo, vec!["my-rule"]);
    }

    #[test]
    fn resolve_errors_on_unknown_skill() {
        let cat = merge(vec![lens("code-kiss")], vec![]);
        assert!(matches!(
            cat.resolve(&["ghost".to_string()]),
            Err(DiscoverError::Unknown(_))
        ));
        assert!(cat.resolve(&["code-kiss".to_string()]).is_ok());
    }

    #[test]
    fn load_repo_reads_on_disk_skills() {
        let dir = tempfile::tempdir().unwrap();
        let sk = dir.path().join("skills/my-rule");
        std::fs::create_dir_all(&sk).unwrap();
        std::fs::write(
            sk.join("SKILL.md"),
            "---\nname: my-rule\ndescription: d\n---\nguidance",
        )
        .unwrap();
        std::fs::write(sk.join("rules.yaml"), "- id: r\n  message: m\n").unwrap();

        let cat = Catalog::discover(dir.path()).unwrap();
        assert_eq!(cat.get("my-rule").unwrap().source, Source::Repo);
        // bundled defaults are present too
        assert!(cat.get("code-review").is_some());
    }
}
