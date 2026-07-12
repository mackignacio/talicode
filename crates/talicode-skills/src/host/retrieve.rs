// SPDX-License-Identifier: MIT
//! Procedural memory — native, on-demand skill retrieval.
//!
//! Implements #47. Rather than keep a skill index resident in the Auditor
//! context (which itself costs tokens), a native keyword search runs on each
//! trigger (the file/query under review) and returns only the matching skills —
//! or nothing ("no skill needed"). A configurable `always_run` floor guarantees
//! security lenses are never silently skipped. Pure and unit-tested; wired into
//! the sweep in #51.

use crate::host::skill::{Skill, SkillKind};
use std::collections::BTreeSet;

/// Search `candidates` for skills relevant to `query`, best match first, capped
/// at `limit`. Empty query or no overlap ⇒ empty ("no skill needed").
pub fn skill_search<'a>(candidates: &[&'a Skill], query: &str, limit: usize) -> Vec<&'a Skill> {
    let terms = tokenize(query);
    if terms.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(usize, &'a Skill)> = candidates
        .iter()
        .map(|s| (overlap(s, &terms), *s))
        .filter(|(n, _)| *n > 0)
        .collect();
    // Most overlap first; stable tie-break by name.
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    scored.into_iter().take(limit).map(|(_, s)| s).collect()
}

/// The skills to actually run: the `always_run` floor (guaranteed) unioned with
/// the search matches, order-preserving and de-duplicated. The floor makes e.g.
/// secret scanning run regardless of the query.
pub fn retrieve<'a>(
    candidates: &[&'a Skill],
    query: &str,
    limit: usize,
    always_run: &[String],
) -> Vec<&'a Skill> {
    let mut out: Vec<&'a Skill> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for s in candidates.iter().filter(|s| always_run.contains(&s.name)) {
        if seen.insert(s.name.clone()) {
            out.push(s);
        }
    }
    for s in skill_search(candidates, query, limit) {
        if seen.insert(s.name.clone()) {
            out.push(s);
        }
    }
    out
}

fn haystack(s: &Skill) -> String {
    let mut h = format!("{} {}", s.name, s.description);
    if let SkillKind::Lens { rules } = &s.kind {
        for r in rules {
            h.push(' ');
            h.push_str(&r.id);
            h.push(' ');
            h.push_str(&r.message);
        }
    }
    h
}

fn overlap(s: &Skill, terms: &[String]) -> usize {
    let hay = tokenize(&haystack(s));
    terms.iter().filter(|t| hay.contains(t)).count()
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| s.len() > 2)
        .map(|s| s.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::skill::Rule;

    fn lens(name: &str, desc: &str, rule_id: &str, rule_msg: &str) -> Skill {
        Skill {
            name: name.into(),
            description: desc.into(),
            guidance: "g".into(),
            kind: SkillKind::Lens {
                rules: vec![Rule {
                    id: rule_id.into(),
                    message: rule_msg.into(),
                    severity: Some("warning".into()),
                }],
            },
        }
    }

    fn catalog() -> Vec<Skill> {
        vec![
            lens(
                "code-no-keys",
                "no hardcoded secrets",
                "hardcoded-secret",
                "an api key or token",
            ),
            lens(
                "code-magic-numbers",
                "no unexplained numeric literals",
                "magic-number",
                "a threshold constant",
            ),
            lens(
                "code-kiss",
                "keep it simple",
                "needless-complexity",
                "overly clever convoluted code",
            ),
        ]
    }

    #[test]
    fn search_matches_by_keyword_and_ranks() {
        let c = catalog();
        let refs: Vec<&Skill> = c.iter().collect();
        let hits = skill_search(&refs, "this line embeds an api key token", 6);
        assert_eq!(hits[0].name, "code-no-keys");
    }

    #[test]
    fn search_empty_query_or_no_overlap_is_empty() {
        let c = catalog();
        let refs: Vec<&Skill> = c.iter().collect();
        assert!(skill_search(&refs, "", 6).is_empty());
        assert!(skill_search(&refs, "zzz qqq vvv", 6).is_empty());
    }

    #[test]
    fn search_respects_the_limit() {
        let c = catalog();
        let refs: Vec<&Skill> = c.iter().collect();
        // "code" appears in every name ⇒ all match; cap at 2.
        let hits = skill_search(&refs, "code review", 2);
        assert_eq!(hits.len(), 2);
    }

    #[test]
    fn retrieve_always_includes_the_floor() {
        let c = catalog();
        let refs: Vec<&Skill> = c.iter().collect();
        // Query only hits magic-numbers, but the floor forces no-keys in too.
        let out = retrieve(
            &refs,
            "a threshold constant",
            6,
            &["code-no-keys".to_string()],
        );
        let names: Vec<&str> = out.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"code-no-keys"), "floor lens present");
        assert!(
            names.contains(&"code-magic-numbers"),
            "search match present"
        );
        // No duplication when a floor lens also matches the search.
        let out2 = retrieve(&refs, "api key token", 6, &["code-no-keys".to_string()]);
        assert_eq!(out2.iter().filter(|s| s.name == "code-no-keys").count(), 1);
    }
}
