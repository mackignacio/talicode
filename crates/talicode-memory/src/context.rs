// SPDX-License-Identifier: MIT
//! Working memory — the in-RAM context assembler and compression trigger.
//!
//! Implements #49. Per turn, working memory assembles the procedural / semantic
//! / architectural / episodic sections into the Auditor prompt within a token
//! budget (dropping the lowest-priority content first). Across the session it
//! bounds the running conversation with a 250K-soft / 500K-hard budget over
//! `all_inputs + output − semantic_tokens`; when the coding LLM finishes past the
//! soft budget (or at the hard cap) the conversation is compressed into an
//! episodic summary — with the semantic context stripped and versioned as a
//! content-addressed delta chain so unchanged context is never re-stored. All
//! logic here is pure and unit-tested. LLM-based compression is roadmap.

use crate::episode::{Episode, MemoryType};

/// The per-turn context sections, highest priority first when trimming.
#[derive(Debug, Clone, Default)]
pub struct ContextParts {
    /// Matched skills' guidance (procedural) — highest priority.
    pub procedural: String,
    /// Durable facts (semantic).
    pub semantic: String,
    /// Codebase overview (architectural).
    pub architecture: String,
    /// Reuse/avoid/last-time (episodic) — lowest priority.
    pub episodic: String,
}

/// Compose the sections into one context block within `budget_tokens`, keeping
/// higher-priority sections and dropping the lowest-priority tail once full. The
/// highest-priority section is always included (even if it alone exceeds budget).
pub fn assemble(parts: &ContextParts, budget_tokens: usize) -> String {
    let sections = [
        &parts.procedural,
        &parts.semantic,
        &parts.architecture,
        &parts.episodic,
    ];
    let mut out = String::new();
    let mut used = 0usize;
    for sec in sections {
        if sec.trim().is_empty() {
            continue;
        }
        let cost = est_tokens(sec);
        if !out.is_empty() && used + cost > budget_tokens {
            break; // drop this and every lower-priority section
        }
        out.push_str(sec.trim_end());
        out.push_str("\n\n");
        used += cost;
    }
    out.trim_end().to_string()
}

/// Cheap token estimate (~4 chars/token) for MVP budgeting.
pub fn est_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// A running session's token tally (working memory's persistent counters).
#[derive(Debug, Clone, Default)]
pub struct Session {
    /// Turns taken.
    pub turns: u64,
    /// Cumulative input tokens (includes semantic).
    pub input_tokens: u64,
    /// Cumulative output tokens.
    pub output_tokens: u64,
    /// Cumulative semantic-block tokens (excluded from the budget).
    pub semantic_tokens: u64,
    /// Notes to fold into the compressed summary.
    pub notes: Vec<String>,
}

impl Session {
    /// Budget spend: all inputs + output **minus** the semantic block.
    pub fn spent(&self) -> u64 {
        self.input_tokens
            .saturating_add(self.output_tokens)
            .saturating_sub(self.semantic_tokens)
    }
}

/// Whether to compress now: force at the hard ceiling, else only once the coding
/// LLM is idle past the soft budget (never interrupt work mid-task).
pub fn should_compress(spent: u64, soft: u64, hard: u64, llm_active: bool) -> bool {
    spent >= hard || (spent >= soft && !llm_active)
}

/// Compress a session into an episodic `summary` (semantic already excluded from
/// the counters). MVP is a structured summary; LLM narrative compression is
/// roadmap.
pub fn compress(session: &Session) -> Episode {
    let mut content = format!(
        "Session: {} turns, {} in / {} out tokens ({} counted toward budget)",
        session.turns,
        session.input_tokens,
        session.output_tokens,
        session.spent()
    );
    if !session.notes.is_empty() {
        content.push_str(". Notes: ");
        content.push_str(&session.notes.join("; "));
    }
    Episode::new(MemoryType::Summary, &content, vec![])
}

// --- Semantic delta chain ("a blockchain for LLM memory") --------------------

/// The change between two semantic-context snapshots.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Delta {
    /// Facts (slugs) added vs. the parent.
    pub added: Vec<String>,
    /// Facts (slugs) removed vs. the parent.
    pub removed: Vec<String>,
}

/// A content-addressed snapshot: the delta vs. its parent, hash-linked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Content hash (FNV-1a of parent + delta).
    pub id: String,
    /// Parent snapshot id (None ⇒ genesis).
    pub parent: Option<String>,
    /// Facts added at this snapshot.
    pub added: Vec<String>,
    /// Facts removed at this snapshot.
    pub removed: Vec<String>,
}

/// Facts in `current` not in `prev` (added) and in `prev` not in `current`
/// (removed), each sorted.
pub fn diff(prev: &[String], current: &[String]) -> Delta {
    let mut added: Vec<String> = current
        .iter()
        .filter(|c| !prev.contains(c))
        .cloned()
        .collect();
    let mut removed: Vec<String> = prev
        .iter()
        .filter(|p| !current.contains(p))
        .cloned()
        .collect();
    added.sort();
    removed.sort();
    Delta { added, removed }
}

/// Append a snapshot for `current` to the chain, storing **only the delta** vs.
/// the current head. Returns `false` (no new snapshot) when nothing changed — so
/// unchanged semantic context is never re-stored.
pub fn extend_chain(chain: &mut Vec<Snapshot>, current: &[String]) -> bool {
    let head_id = chain.last().map(|h| h.id.clone());
    let prev = match &head_id {
        Some(id) => resolve(chain, id),
        None => Vec::new(),
    };
    let d = diff(&prev, current);
    if d.added.is_empty() && d.removed.is_empty() {
        return false;
    }
    let id = fnv1a(&format!("{head_id:?}|{:?}|{:?}", d.added, d.removed));
    chain.push(Snapshot {
        id,
        parent: head_id,
        added: d.added,
        removed: d.removed,
    });
    true
}

/// Reconstruct the full semantic-fact set at `head_id` by walking the parent
/// chain from the genesis and applying each delta. Sorted.
pub fn resolve(chain: &[Snapshot], head_id: &str) -> Vec<String> {
    // Walk head → genesis, collecting the path.
    let by_id = |id: &str| chain.iter().find(|s| s.id == id);
    let mut path: Vec<&Snapshot> = Vec::new();
    let mut cursor = by_id(head_id);
    while let Some(snap) = cursor {
        path.push(snap);
        cursor = snap.parent.as_deref().and_then(by_id);
    }
    // Apply genesis → head.
    let mut set: Vec<String> = Vec::new();
    for snap in path.into_iter().rev() {
        for a in &snap.added {
            if !set.contains(a) {
                set.push(a.clone());
            }
        }
        set.retain(|x| !snap.removed.contains(x));
    }
    set.sort();
    set
}

/// FNV-1a (64-bit) hex — a stable content hash, no dependencies. A Merkle/CID
/// store is roadmap.
pub fn fnv1a(s: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(len: usize) -> String {
        "x".repeat(len)
    }

    #[test]
    fn assemble_keeps_priority_and_drops_low_tail() {
        let parts = ContextParts {
            procedural: part(40),   // ~10 tokens
            semantic: part(40),     // ~10
            architecture: part(40), // ~10
            episodic: part(40),     // ~10
        };
        // Budget for ~2 sections: procedural + semantic kept, rest dropped.
        let out = assemble(&parts, 20);
        let blocks = out.split("\n\n").filter(|b| !b.is_empty()).count();
        assert_eq!(blocks, 2);
    }

    #[test]
    fn assemble_always_includes_highest_priority() {
        let parts = ContextParts {
            procedural: part(400),
            ..Default::default()
        };
        let out = assemble(&parts, 1);
        assert!(
            !out.is_empty(),
            "procedural always included even over budget"
        );
    }

    #[test]
    fn should_compress_defers_while_active() {
        // Below soft: never.
        assert!(!should_compress(100, 250, 500, false));
        // Past soft but LLM still working: hold off.
        assert!(!should_compress(300, 250, 500, true));
        // Past soft and idle: compress.
        assert!(should_compress(300, 250, 500, false));
        // Hard cap: compress even while active.
        assert!(should_compress(500, 250, 500, true));
    }

    #[test]
    fn spent_excludes_semantic_tokens() {
        let s = Session {
            turns: 3,
            input_tokens: 200,
            output_tokens: 100,
            semantic_tokens: 50,
            notes: vec![],
        };
        assert_eq!(s.spent(), 250);
        let ep = compress(&s);
        assert_eq!(ep.memory_type, MemoryType::Summary);
        assert!(ep.content.contains("3 turns"));
        assert!(ep.content.contains("250 counted"));
    }

    #[test]
    fn diff_reports_added_and_removed() {
        let d = diff(&["a".into(), "b".into()], &["b".into(), "c".into()]);
        assert_eq!(d.added, vec!["c"]);
        assert_eq!(d.removed, vec!["a"]);
    }

    #[test]
    fn chain_stores_deltas_and_resolves() {
        let mut chain: Vec<Snapshot> = Vec::new();
        assert!(extend_chain(&mut chain, &["a".into(), "b".into()]));
        assert!(extend_chain(&mut chain, &["b".into(), "c".into()]));
        assert_eq!(chain.len(), 2);
        // Second snapshot stored only the delta (add c, remove a).
        assert_eq!(chain[1].added, vec!["c"]);
        assert_eq!(chain[1].removed, vec!["a"]);
        // Resolving the head reconstructs the current set.
        let head = chain.last().unwrap().id.clone();
        assert_eq!(
            resolve(&chain, &head),
            vec!["b".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn chain_skips_unchanged_context() {
        let mut chain: Vec<Snapshot> = Vec::new();
        assert!(extend_chain(&mut chain, &["a".into()]));
        // Same set again ⇒ no new snapshot.
        assert!(!extend_chain(&mut chain, &["a".into()]));
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn fnv1a_is_stable_and_distinct() {
        assert_eq!(fnv1a("abc"), fnv1a("abc"));
        assert_ne!(fnv1a("abc"), fnv1a("abd"));
        assert_eq!(fnv1a("abc").len(), 16);
    }
}
