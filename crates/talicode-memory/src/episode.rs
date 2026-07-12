// SPDX-License-Identifier: MIT
//! Episodic memory — TaliCode's long-term store of experiences and decisions.
//!
//! Implements #45. Time-stamped experiences (distinct from semantic timeless
//! facts) live as JSONL at `.talicode/episodes.jsonl` (local). Each carries a
//! `memory_type` (learning / mistake / experience / summary), a durable/scratch
//! tier, tags, and links (supersedes / contradicts / related_to). Retrieval is a
//! pure keyword+recency rank; `build_context` groups reuse/avoid/last-time for
//! injection. A SQLite + BM25 + embedding backend is roadmap.

use crate::memory::Tier;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::Path;
use talicode_core::report::Severity;
use talicode_core::usage::local_today;
use talicode_skills::host::invoke::SweepOutcome;

/// Repo-local dir holding the episodic ledger (git-ignored).
pub const EPISODE_DIR: &str = ".talicode";
/// The append-only episodic ledger.
pub const EPISODE_FILE: &str = "episodes.jsonl";

/// What kind of experience an episode records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    /// A good, reusable discovery.
    Learning,
    /// A bad outcome never to repeat.
    Mistake,
    /// A recurring scenario or preference (may promote to a skill).
    Experience,
    /// The compressed catch-all.
    Summary,
}

/// A typed relationship between episodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    /// This episode replaces the target.
    Supersedes,
    /// This episode contradicts the target.
    Contradicts,
    /// This episode is related to the target.
    RelatedTo,
}

/// A link from one episode to another (by id).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// The relationship.
    pub kind: LinkKind,
    /// The target episode id.
    pub target: u64,
}

/// Per-severity finding counts for a sweep summary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeverityCounts {
    /// Info-level findings.
    pub info: u64,
    /// Warning-level findings.
    pub warning: u64,
    /// Error-level findings.
    pub error: u64,
}

/// One recallable experience.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    /// Monotonic id (0 ⇒ assigned on record).
    pub id: u64,
    /// The experience text.
    pub content: String,
    /// Learning / mistake / experience / summary.
    pub memory_type: MemoryType,
    /// Durable or scratch.
    pub tier: Tier,
    /// Free-form tags.
    pub tags: Vec<String>,
    /// Local creation date (`YYYY-MM-DD`).
    pub created: String,
    /// Last-access date.
    #[serde(default)]
    pub accessed: String,
    /// Access frequency.
    #[serde(default)]
    pub access_count: u64,
    /// Expiry date for scratch entries (`YYYY-MM-DD`).
    #[serde(default)]
    pub expires_at: Option<String>,
    /// Links to other episodes.
    #[serde(default)]
    pub links: Vec<Link>,
    /// Files swept (summary episodes only).
    #[serde(default)]
    pub files: Option<Vec<String>>,
    /// Total findings (summary episodes only).
    #[serde(default)]
    pub findings_total: Option<u64>,
    /// Findings by severity (summary episodes only).
    #[serde(default)]
    pub by_severity: Option<SeverityCounts>,
    /// Most frequent rule ids (summary episodes only).
    #[serde(default)]
    pub top_rules: Option<Vec<String>>,
}

impl Episode {
    /// A minimal experience of the given kind, stamped with today's date.
    pub fn new(memory_type: MemoryType, content: &str, tags: Vec<String>) -> Self {
        let today = local_today();
        Episode {
            id: 0,
            content: content.trim().to_string(),
            memory_type,
            tier: Tier::Durable,
            tags,
            created: today.clone(),
            accessed: today,
            access_count: 0,
            expires_at: None,
            links: Vec::new(),
            files: None,
            findings_total: None,
            by_severity: None,
            top_rules: None,
        }
    }
}

/// Weights for the hybrid ranking (`w_fts·relevance + w_recency·decay`). The
/// vector term (`w_vec`) lands with the roadmap embedding backend.
#[derive(Debug, Clone, Copy)]
pub struct Weights {
    /// Keyword-relevance weight.
    pub w_fts: f64,
    /// Recency weight.
    pub w_recency: f64,
    /// Recency half-life in days.
    pub half_life_days: f64,
}

impl Default for Weights {
    fn default() -> Self {
        Weights {
            w_fts: 0.4,
            w_recency: 0.2,
            half_life_days: 30.0,
        }
    }
}

/// Append an episode (assigning the next id when `id == 0`); returns the stored
/// episode. Best-effort — episodic accounting must never block a sweep.
pub fn record(root: &Path, episode: Episode) -> std::io::Result<Episode> {
    use std::io::Write;
    let dir = root.join(EPISODE_DIR);
    std::fs::create_dir_all(&dir)?;
    let id = if episode.id == 0 {
        next_id(root)
    } else {
        episode.id
    };
    let stored = Episode { id, ..episode };
    let line = serde_json::to_string(&stored).map_err(std::io::Error::other)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(EPISODE_FILE))?;
    writeln!(file, "{line}")?;
    Ok(stored)
}

/// Read all episodes (skipping unparseable lines). Missing file ⇒ empty.
pub fn read(root: &Path) -> Vec<Episode> {
    let path = root.join(EPISODE_DIR).join(EPISODE_FILE);
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// The most recently recorded episode, if any.
pub fn last(root: &Path) -> Option<Episode> {
    read(root).into_iter().max_by_key(|e| e.id)
}

/// Delete an episode by id; returns whether one was removed.
pub fn forget(root: &Path, id: u64) -> std::io::Result<bool> {
    let all = read(root);
    let kept: Vec<Episode> = all.iter().filter(|e| e.id != id).cloned().collect();
    if kept.len() == all.len() {
        return Ok(false);
    }
    write_all(root, &kept)?;
    Ok(true)
}

/// Record a new episode that supersedes `old_id` (writes the entry + a
/// `supersedes` link in one op, so old knowledge stays queryable with lineage).
pub fn supersede(
    root: &Path,
    old_id: u64,
    memory_type: MemoryType,
    content: &str,
    tags: Vec<String>,
) -> std::io::Result<Episode> {
    let mut ep = Episode::new(memory_type, content, tags);
    ep.links.push(Link {
        kind: LinkKind::Supersedes,
        target: old_id,
    });
    record(root, ep)
}

/// Prune expired scratch episodes. `apply=false` ⇒ dry run (returns the
/// candidates without deleting); `apply=true` ⇒ rewrite the ledger without them.
pub fn prune(root: &Path, today: NaiveDate, apply: bool) -> std::io::Result<Vec<Episode>> {
    let all = read(root);
    let expired: Vec<Episode> = all
        .iter()
        .filter(|e| is_expired(e, today))
        .cloned()
        .collect();
    if apply && !expired.is_empty() {
        let kept: Vec<Episode> = all.into_iter().filter(|e| !is_expired(e, today)).collect();
        write_all(root, &kept)?;
    }
    Ok(expired)
}

/// Episodes still live today (expired scratch dropped). Pure.
pub fn active(episodes: &[Episode], today: NaiveDate) -> Vec<&Episode> {
    episodes.iter().filter(|e| !is_expired(e, today)).collect()
}

/// Rank episodes against a query (keyword relevance + recency decay). Pure.
pub fn rank<'a>(
    episodes: &'a [Episode],
    query: &str,
    today: NaiveDate,
    weights: Weights,
    limit: usize,
) -> Vec<&'a Episode> {
    let terms = tokenize(query);
    let mut scored: Vec<(f64, &Episode)> = active(episodes, today)
        .into_iter()
        .map(|e| (score(e, &terms, today, weights), e))
        .collect();
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.1.id.cmp(&a.1.id))
    });
    scored.into_iter().take(limit).map(|(_, e)| e).collect()
}

/// Build the episodic context block: reuse (learnings), avoid (mistakes),
/// relevant experiences, and a "Last sweep" timeline line. `""` when empty.
pub fn build_context(episodes: &[Episode], today: NaiveDate, limit: usize) -> String {
    let live = active(episodes, today);
    let of = |t: MemoryType| -> Vec<&Episode> {
        live.iter()
            .filter(|e| e.memory_type == t)
            .take(limit)
            .copied()
            .collect()
    };
    let learnings = of(MemoryType::Learning);
    let mistakes = of(MemoryType::Mistake);
    let experiences = of(MemoryType::Experience);
    let last_summary = live
        .iter()
        .filter(|e| e.memory_type == MemoryType::Summary)
        .max_by_key(|e| e.id);

    let mut out = String::new();
    let mut section = |title: &str, eps: &[&Episode]| {
        if eps.is_empty() {
            return;
        }
        out.push_str(title);
        out.push('\n');
        for e in eps {
            out.push_str(&format!("- {}\n", e.content.replace('\n', " ")));
        }
    };
    section("Reuse (past learnings):", &learnings);
    section("Avoid (past mistakes):", &mistakes);
    section("Recurring experiences:", &experiences);
    if let Some(s) = last_summary {
        out.push_str(&format!(
            "Last sweep ({}): {}\n",
            s.created,
            s.content.replace('\n', " ")
        ));
    }
    out
}

/// Summarize a sweep into a compressed `summary` episode (id assigned on record).
pub fn summarize(outcome: &SweepOutcome, files: &[String]) -> Episode {
    let mut counts = SeverityCounts::default();
    let mut rule_freq: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for f in &outcome.findings {
        match f.severity {
            Severity::Info => counts.info += 1,
            Severity::Warning => counts.warning += 1,
            Severity::Error => counts.error += 1,
        }
        *rule_freq.entry(f.rule.clone()).or_default() += 1;
    }
    let mut rules: Vec<(String, u64)> = rule_freq.into_iter().collect();
    rules.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let top_rules: Vec<String> = rules.into_iter().take(3).map(|(r, _)| r).collect();
    let total = outcome.findings.len() as u64;
    let content = format!(
        "{} finding(s) across {} file(s) (e{} / w{} / i{}); top: {}",
        total,
        files.len(),
        counts.error,
        counts.warning,
        counts.info,
        if top_rules.is_empty() {
            "none".to_string()
        } else {
            top_rules.join(", ")
        }
    );

    let mut ep = Episode::new(MemoryType::Summary, &content, vec![]);
    ep.files = Some(files.to_vec());
    ep.findings_total = Some(total);
    ep.by_severity = Some(counts);
    ep.top_rules = Some(top_rules);
    ep
}

/// The expiry date (`YYYY-MM-DD`) for a scratch entry created today with the
/// given TTL in hours (rounded up to whole days, minimum one).
pub fn scratch_expiry(ttl_hours: u64) -> String {
    let days = (ttl_hours.div_ceil(24)).max(1) as i64;
    (chrono::Local::now().date_naive() + chrono::Duration::days(days)).to_string()
}

fn next_id(root: &Path) -> u64 {
    read(root).iter().map(|e| e.id).max().unwrap_or(0) + 1
}

fn write_all(root: &Path, episodes: &[Episode]) -> std::io::Result<()> {
    let dir = root.join(EPISODE_DIR);
    std::fs::create_dir_all(&dir)?;
    let mut buf = String::new();
    for e in episodes {
        buf.push_str(&serde_json::to_string(e).map_err(std::io::Error::other)?);
        buf.push('\n');
    }
    std::fs::write(dir.join(EPISODE_FILE), buf)
}

// --- Episodic → procedural auto-promotion (#46) ------------------------------

/// A candidate skill synthesized from a recurring experience.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDraft {
    /// Skill folder name.
    pub slug: String,
    /// One-line description.
    pub description: String,
    /// The rule message the generated lens flags on.
    pub rule_message: String,
}

/// Words dropped when deriving a skill slug from a preference sentence.
const STOPWORDS: [&str; 20] = [
    "the", "a", "an", "and", "or", "of", "to", "in", "on", "user", "users", "i", "we", "you",
    "code", "codebase", "always", "never", "this", "that",
];

/// Words that mark a *negative* preference (⇒ the slug is prefixed `no-`).
const NEGATIVE: [&str; 9] = [
    "no", "not", "dont", "doesnt", "dislike", "dislikes", "hate", "hates", "avoid",
];

/// Which recurring experiences have crossed `threshold` and aren't already
/// skills — the candidates to auto-promote. Pure: `existing` is the set of skill
/// slugs to exclude. Experiences are grouped by their derived slug.
pub fn due_for_skill(
    episodes: &[Episode],
    threshold: usize,
    existing: &[String],
) -> Vec<SkillDraft> {
    let mut by_slug: std::collections::BTreeMap<String, Vec<&Episode>> =
        std::collections::BTreeMap::new();
    for e in episodes
        .iter()
        .filter(|e| e.memory_type == MemoryType::Experience)
    {
        by_slug.entry(derive_slug(&e.content)).or_default().push(e);
    }
    by_slug
        .into_iter()
        .filter(|(slug, group)| group.len() >= threshold && !existing.contains(slug))
        .map(|(slug, group)| draft_from(slug, group[0]))
        .collect()
}

/// Write a templated `skills/<slug>/` (SKILL.md + rules.yaml) for a draft. The
/// frontmatter and rules are serialized with `serde_yaml` so text containing
/// colons/quotes stays valid YAML — the generated skill parses through the
/// host's skill loader like an authored one.
pub fn promote(root: &Path, draft: &SkillDraft) -> std::io::Result<()> {
    #[derive(Serialize)]
    struct Frontmatter<'a> {
        name: &'a str,
        description: &'a str,
    }
    #[derive(Serialize)]
    struct PromotedRule<'a> {
        id: String,
        message: &'a str,
        severity: &'a str,
    }

    let dir = root.join("skills").join(&draft.slug);
    std::fs::create_dir_all(&dir)?;
    let fm = serde_yaml::to_string(&Frontmatter {
        name: &draft.slug,
        description: &draft.description,
    })
    .map_err(std::io::Error::other)?;
    let skill_md = format!("---\n{fm}---\n{}\n", draft.description);
    let rules = serde_yaml::to_string(&[PromotedRule {
        id: format!("{}-rule", draft.slug),
        message: &draft.rule_message,
        severity: "warning",
    }])
    .map_err(std::io::Error::other)?;
    std::fs::write(dir.join("SKILL.md"), skill_md)?;
    std::fs::write(dir.join("rules.yaml"), rules)
}

fn draft_from(slug: String, source: &Episode) -> SkillDraft {
    SkillDraft {
        description: format!(
            "Auto-generated from a recurring team preference: {}",
            source.content
        ),
        rule_message: format!("Recorded team preference: {}", source.content),
        slug,
    }
}

/// Derive a skill slug from a preference sentence: keep meaningful words, and
/// prefix `no-` when the sentence expresses a negative preference. E.g.
/// "user dislikes while loops" → `no-while-loops`.
fn derive_slug(content: &str) -> String {
    let words = tokenize(content);
    let negative = words.iter().any(|w| NEGATIVE.contains(&w.as_str()));
    let kept: Vec<String> = words
        .into_iter()
        .filter(|w| !STOPWORDS.contains(&w.as_str()) && !NEGATIVE.contains(&w.as_str()))
        .collect();
    let core = if kept.is_empty() {
        "preference".to_string()
    } else {
        kept.join("-")
    };
    if negative {
        format!("no-{core}")
    } else {
        core
    }
}

fn is_expired(e: &Episode, today: NaiveDate) -> bool {
    if e.tier != Tier::Scratch {
        return false;
    }
    match e
        .expires_at
        .as_deref()
        .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
    {
        Some(exp) => today > exp,
        None => false,
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| s.len() > 2)
        .map(|s| s.to_ascii_lowercase())
        .collect()
}

/// `w_fts · keyword_relevance + w_recency · recency_decay`. The keyword term
/// approximates FTS BM25 for the MVP (roadmap swaps in real BM25 + a vector term).
fn score(e: &Episode, terms: &[String], today: NaiveDate, w: Weights) -> f64 {
    let hay = tokenize(&format!("{} {}", e.content, e.tags.join(" ")));
    let relevance = if terms.is_empty() {
        0.0
    } else {
        terms.iter().filter(|t| hay.contains(t)).count() as f64 / terms.len() as f64
    };
    w.w_fts * relevance + w.w_recency * recency_decay(&e.created, today, w.half_life_days)
}

fn recency_decay(created: &str, today: NaiveDate, half_life_days: f64) -> f64 {
    match NaiveDate::parse_from_str(created, "%Y-%m-%d") {
        Ok(d) => {
            let age = (today - d).num_days().max(0) as f64;
            (-0.693 * age / half_life_days).exp()
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

    fn ep(id: u64, t: MemoryType, created: &str, content: &str) -> Episode {
        let mut e = Episode::new(t, content, vec![]);
        e.id = id;
        e.created = created.into();
        e
    }

    #[test]
    fn record_read_round_trips_and_assigns_ids() {
        let dir = tempfile::tempdir().unwrap();
        let a = record(
            dir.path(),
            Episode::new(MemoryType::Learning, "use alias+version", vec![]),
        )
        .unwrap();
        let b = record(
            dir.path(),
            Episode::new(MemoryType::Mistake, "deleted an env var", vec![]),
        )
        .unwrap();
        assert_eq!(a.id, 1);
        assert_eq!(b.id, 2);
        let all = read(dir.path());
        assert_eq!(all.len(), 2);
        assert_eq!(last(dir.path()).unwrap().id, 2);
    }

    #[test]
    fn supersede_writes_new_plus_link() {
        let dir = tempfile::tempdir().unwrap();
        let old = record(
            dir.path(),
            Episode::new(MemoryType::Learning, "old way", vec![]),
        )
        .unwrap();
        let new = supersede(dir.path(), old.id, MemoryType::Learning, "new way", vec![]).unwrap();
        assert_eq!(
            new.links,
            vec![Link {
                kind: LinkKind::Supersedes,
                target: old.id
            }]
        );
        assert_eq!(read(dir.path()).len(), 2);
    }

    #[test]
    fn prune_drops_expired_scratch_only_when_applied() {
        let dir = tempfile::tempdir().unwrap();
        let mut scratch = Episode::new(MemoryType::Experience, "transient", vec![]);
        scratch.tier = Tier::Scratch;
        scratch.expires_at = Some("2026-07-01".into());
        record(dir.path(), scratch).unwrap();
        record(
            dir.path(),
            Episode::new(MemoryType::Learning, "durable", vec![]),
        )
        .unwrap();

        let today = day("2026-07-12");
        let dry = prune(dir.path(), today, false).unwrap();
        assert_eq!(dry.len(), 1, "dry run finds the expired scratch");
        assert_eq!(read(dir.path()).len(), 2, "but does not delete it");

        prune(dir.path(), today, true).unwrap();
        assert_eq!(read(dir.path()).len(), 1, "apply removes expired scratch");
    }

    #[test]
    fn active_hides_expired_scratch() {
        let mut scratch = ep(1, MemoryType::Experience, "2026-06-01", "x");
        scratch.tier = Tier::Scratch;
        scratch.expires_at = Some("2026-06-30".into());
        let durable = ep(2, MemoryType::Learning, "2026-06-01", "y");
        let eps = vec![scratch, durable];
        let live = active(&eps, day("2026-07-12"));
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].id, 2);
    }

    #[test]
    fn rank_matches_keyword_over_recency() {
        let eps = vec![
            ep(
                1,
                MemoryType::Learning,
                "2026-01-01",
                "lambda alias version deploy",
            ),
            ep(
                2,
                MemoryType::Learning,
                "2026-07-11",
                "unrelated widget note",
            ),
        ];
        let ranked = rank(&eps, "lambda", day("2026-07-12"), Weights::default(), 10);
        assert_eq!(ranked[0].id, 1);
    }

    #[test]
    fn build_context_groups_reuse_avoid_and_last_sweep() {
        let eps = vec![
            ep(1, MemoryType::Learning, "2026-07-10", "reuse alias+version"),
            ep(
                2,
                MemoryType::Mistake,
                "2026-07-10",
                "never delete env blindly",
            ),
            ep(
                3,
                MemoryType::Summary,
                "2026-07-11",
                "2 findings across 1 file",
            ),
        ];
        let ctx = build_context(&eps, day("2026-07-12"), 8);
        assert!(ctx.contains("Reuse (past learnings):"));
        assert!(ctx.contains("Avoid (past mistakes):"));
        assert!(ctx.contains("Last sweep (2026-07-11):"));
    }

    #[test]
    fn build_context_empty_is_blank() {
        assert_eq!(build_context(&[], day("2026-07-12"), 8), "");
    }

    #[test]
    fn summarize_counts_severities_and_top_rules() {
        use talicode_core::report::Finding;
        let outcome = SweepOutcome {
            findings: vec![
                Finding {
                    file: "a.rs".into(),
                    line: 1,
                    severity: Severity::Error,
                    rule: "code-no-keys".into(),
                    message: "m".into(),
                },
                Finding {
                    file: "a.rs".into(),
                    line: 2,
                    severity: Severity::Warning,
                    rule: "code-kiss".into(),
                    message: "m".into(),
                },
                Finding {
                    file: "b.rs".into(),
                    line: 3,
                    severity: Severity::Error,
                    rule: "code-no-keys".into(),
                    message: "m".into(),
                },
            ],
            usage: Default::default(),
            approved: false,
        };
        let e = summarize(&outcome, &["a.rs".into(), "b.rs".into()]);
        assert_eq!(e.memory_type, MemoryType::Summary);
        assert_eq!(e.findings_total, Some(3));
        assert_eq!(
            e.by_severity,
            Some(SeverityCounts {
                info: 0,
                warning: 1,
                error: 2
            })
        );
        assert_eq!(e.top_rules.as_ref().unwrap()[0], "code-no-keys");
        assert!(e.content.contains("3 finding(s) across 2 file(s)"));
    }

    #[test]
    fn derive_slug_prefixes_no_for_negative_preference() {
        assert_eq!(derive_slug("user dislikes while loops"), "no-while-loops");
        assert_eq!(derive_slug("prefer guard clauses"), "prefer-guard-clauses");
    }

    #[test]
    fn due_for_skill_respects_threshold_and_existing() {
        let eps = vec![
            ep(
                1,
                MemoryType::Experience,
                "2026-07-10",
                "user dislikes while loops",
            ),
            ep(
                2,
                MemoryType::Experience,
                "2026-07-11",
                "user dislikes while loops",
            ),
            ep(3, MemoryType::Learning, "2026-07-11", "unrelated learning"),
        ];
        let drafts = due_for_skill(&eps, 2, &[]);
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].slug, "no-while-loops");
        // Already a skill ⇒ excluded; below threshold ⇒ none.
        assert!(due_for_skill(&eps, 2, &["no-while-loops".to_string()]).is_empty());
        assert!(due_for_skill(&eps[..1], 2, &[]).is_empty());
    }

    #[test]
    fn promote_writes_a_parseable_skill() {
        let dir = tempfile::tempdir().unwrap();
        // Use the REAL generated draft — its description/message contain a colon,
        // which must not break the YAML frontmatter/rules.
        let eps = vec![
            ep(
                1,
                MemoryType::Experience,
                "2026-07-10",
                "user dislikes while loops",
            ),
            ep(
                2,
                MemoryType::Experience,
                "2026-07-11",
                "user dislikes while loops",
            ),
        ];
        let draft = due_for_skill(&eps, 2, &[]).remove(0);
        assert!(
            draft.description.contains(':'),
            "realistic draft text has a colon"
        );

        promote(dir.path(), &draft).unwrap();
        let skill_dir = dir.path().join("skills/no-while-loops");
        assert!(skill_dir.join("SKILL.md").is_file());
        let s = talicode_skills::host::skill::Skill::load(&skill_dir).unwrap();
        assert_eq!(s.name, "no-while-loops");
        assert!(!s.is_orchestrator());
    }
}
