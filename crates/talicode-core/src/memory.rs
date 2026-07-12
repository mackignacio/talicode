// SPDX-License-Identifier: MIT
//! Semantic memory — durable, timeless project facts as markdown files.
//!
//! Implements #44. Facts live as markdown files (YAML frontmatter + body) under
//! `.talicode/memory/`, committed so the team shares them — mirroring how this
//! project's own `MEMORY.md` memory works. Retrieval is a pure keyword+recency
//! rank fed into the Auditor's context; a vector/knowledge-graph backend is
//! roadmap. IO is best-effort; the pure ranking/formatting is unit-tested.

use crate::usage::local_today;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Repo-local directory holding semantic-memory markdown files (committed).
pub const MEMORY_DIR: &str = ".talicode/memory";
/// The generated one-line-per-memory index.
pub const INDEX_FILE: &str = "INDEX.md";

/// Lifecycle tier for a memory entry (shared with episodic memory).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// Lives forever.
    #[default]
    Durable,
    /// Transient; carries an expiry.
    Scratch,
}

/// A durable semantic fact.
#[derive(Debug, Clone, PartialEq)]
pub struct Memory {
    /// Stable file-name slug.
    pub slug: String,
    /// Lifecycle tier (durable for semantic facts).
    pub tier: Tier,
    /// Local creation date (`YYYY-MM-DD`).
    pub created: String,
    /// Free-form tags.
    pub tags: Vec<String>,
    /// The fact text (the markdown body).
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Frontmatter {
    slug: String,
    #[serde(default)]
    tier: Tier,
    #[serde(default)]
    created: String,
    #[serde(default)]
    tags: Vec<String>,
}

/// Store a fact as a markdown file, returning the created entry. Best-effort IO.
pub fn store(root: &Path, text: &str, tags: Vec<String>) -> std::io::Result<Memory> {
    let dir = root.join(MEMORY_DIR);
    std::fs::create_dir_all(&dir)?;
    let existing: Vec<String> = read(root).into_iter().map(|m| m.slug).collect();
    let slug = unique_slug(&slugify(text), &existing);
    let mem = Memory {
        slug: slug.clone(),
        tier: Tier::Durable,
        created: local_today(),
        tags,
        body: text.trim().to_string(),
    };
    std::fs::write(dir.join(format!("{slug}.md")), render_markdown(&mem))?;
    write_index(root)?;
    Ok(mem)
}

/// Read every semantic fact (skipping the index and unparseable files). Missing
/// dir ⇒ empty.
pub fn read(root: &Path) -> Vec<Memory> {
    let dir = root.join(MEMORY_DIR);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some(INDEX_FILE) {
            continue;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Some(mem) = parse(&text) {
                out.push(mem);
            }
        }
    }
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    out
}

/// Delete a fact by slug; returns whether a file was removed. Regenerates index.
pub fn forget(root: &Path, slug: &str) -> std::io::Result<bool> {
    let path = root.join(MEMORY_DIR).join(format!("{slug}.md"));
    if !path.exists() {
        return Ok(false);
    }
    std::fs::remove_file(path)?;
    write_index(root)?;
    Ok(true)
}

/// Rank facts against a query: keyword overlap + recency, most relevant first.
/// Pure — `today` is passed so it is deterministic in tests.
pub fn rank<'a>(
    mems: &'a [Memory],
    query: &str,
    today: NaiveDate,
    limit: usize,
) -> Vec<&'a Memory> {
    let terms = tokenize(query);
    let mut scored: Vec<(f64, &Memory)> =
        mems.iter().map(|m| (score(m, &terms, today), m)).collect();
    // Highest score first; stable tie-break by slug for determinism.
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.slug.cmp(&b.1.slug))
    });
    scored.into_iter().take(limit).map(|(_, m)| m).collect()
}

/// Build the "Project memory" prompt section from the top-ranked facts. Empty ⇒
/// `""` so nothing is injected.
pub fn build_section(mems: &[Memory], today: NaiveDate, limit: usize) -> String {
    let top = rank(mems, "", today, limit);
    if top.is_empty() {
        return String::new();
    }
    let mut out = String::from("Project memory (durable facts):\n");
    for m in top {
        out.push_str(&format!("- {}\n", m.body.replace('\n', " ")));
    }
    out
}

/// Regenerate `INDEX.md` from all facts (one line each).
fn write_index(root: &Path) -> std::io::Result<()> {
    let mems = read(root);
    let path = root.join(MEMORY_DIR).join(INDEX_FILE);
    std::fs::write(path, render_index(&mems))
}

fn render_index(mems: &[Memory]) -> String {
    let mut out = String::from("# Semantic memory\n\n");
    for m in mems {
        let first = m.body.lines().next().unwrap_or("").trim();
        out.push_str(&format!("- [{}]({}.md) — {}\n", m.slug, m.slug, first));
    }
    out
}

fn render_markdown(m: &Memory) -> String {
    let fm = Frontmatter {
        slug: m.slug.clone(),
        tier: m.tier,
        created: m.created.clone(),
        tags: m.tags.clone(),
    };
    let fm_yaml = serde_yaml::to_string(&fm).unwrap_or_default();
    format!("---\n{fm_yaml}---\n{}\n", m.body)
}

fn parse(text: &str) -> Option<Memory> {
    let (fm_text, body) = split_frontmatter(text)?;
    let fm: Frontmatter = serde_yaml::from_str(fm_text).ok()?;
    Some(Memory {
        slug: fm.slug,
        tier: fm.tier,
        created: fm.created,
        tags: fm.tags,
        body: body.trim().to_string(),
    })
}

/// Split a leading `--- ... ---` YAML frontmatter block from the body.
fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    Some((&rest[..end], &rest[end + "\n---\n".len()..]))
}

/// A stable, file-safe slug from the first words of a fact.
fn slugify(text: &str) -> String {
    let slug: String = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let parts: Vec<&str> = slug.split('-').filter(|s| !s.is_empty()).take(6).collect();
    let s = parts.join("-");
    if s.is_empty() {
        "fact".to_string()
    } else {
        s
    }
}

/// Disambiguate a slug against existing ones (`base`, `base-2`, `base-3`, …).
fn unique_slug(base: &str, existing: &[String]) -> String {
    if !existing.iter().any(|e| e == base) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base}-{n}"))
        .find(|c| !existing.iter().any(|e| e == c))
        .unwrap_or_else(|| base.to_string())
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| s.len() > 2)
        .map(|s| s.to_ascii_lowercase())
        .collect()
}

fn score(m: &Memory, terms: &[String], today: NaiveDate) -> f64 {
    let hay = tokenize(&format!("{} {}", m.body, m.tags.join(" ")));
    let overlap = terms.iter().filter(|t| hay.contains(t)).count() as f64;
    overlap + recency(&m.created, today)
}

/// Recency weight: `1 / (1 + age_days)`, 0 when the date is unparseable.
fn recency(created: &str, today: NaiveDate) -> f64 {
    match NaiveDate::parse_from_str(created, "%Y-%m-%d") {
        Ok(d) => {
            let age = (today - d).num_days().max(0) as f64;
            1.0 / (1.0 + age)
        }
        Err(_) => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn mem(slug: &str, created: &str, body: &str, tags: &[&str]) -> Memory {
        Memory {
            slug: slug.into(),
            tier: Tier::Durable,
            created: created.into(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
            body: body.into(),
        }
    }

    #[test]
    fn store_read_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let m = store(
            dir.path(),
            "this repo uses raw SQL on purpose",
            vec!["sql".into()],
        )
        .unwrap();
        assert_eq!(m.tier, Tier::Durable);
        let all = read(dir.path());
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].slug, m.slug);
        assert!(all[0].body.contains("raw SQL"));
        assert_eq!(all[0].tags, vec!["sql"]);
    }

    #[test]
    fn slug_collisions_disambiguate() {
        assert_eq!(unique_slug("foo", &[]), "foo");
        assert_eq!(unique_slug("foo", &["foo".into()]), "foo-2");
        assert_eq!(unique_slug("foo", &["foo".into(), "foo-2".into()]), "foo-3");
    }

    #[test]
    fn forget_removes_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let m = store(dir.path(), "a durable fact here", vec![]).unwrap();
        assert!(forget(dir.path(), &m.slug).unwrap());
        assert!(read(dir.path()).is_empty());
        assert!(!forget(dir.path(), "nope").unwrap());
    }

    #[test]
    fn rank_prefers_keyword_then_recency() {
        let mems = vec![
            mem("old-sql", "2026-01-01", "raw SQL is fine here", &["sql"]),
            mem(
                "recent-noise",
                "2026-07-10",
                "unrelated note about widgets",
                &[],
            ),
        ];
        let ranked = rank(&mems, "sql", day("2026-07-12"), 10);
        assert_eq!(ranked[0].slug, "old-sql", "keyword match wins over recency");
    }

    #[test]
    fn build_section_empty_is_blank_else_lists() {
        assert_eq!(build_section(&[], day("2026-07-12"), 8), "");
        let mems = vec![mem("f", "2026-07-11", "prefer composition", &[])];
        let s = build_section(&mems, day("2026-07-12"), 8);
        assert!(s.contains("Project memory"));
        assert!(s.contains("prefer composition"));
    }

    #[test]
    fn read_missing_dir_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read(dir.path()).is_empty());
    }
}
