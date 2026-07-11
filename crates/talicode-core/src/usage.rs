// SPDX-License-Identifier: MIT
//! Token-spend reporting — the per-execution footer and the daily ledger.
//!
//! Implements #24. Each sweep sums provider [`Usage`] and prints a one-line
//! footer, then appends a row to `.talicode/usage.jsonl` (repo-local). Tokens
//! are the source of truth; the dollar figure is a clearly-labelled estimate
//! from a small per-model price table. Ledger writes are best-effort — usage
//! accounting must never block a sweep.

use crate::provider::Usage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Repo-local directory holding the usage ledger (git-ignored).
pub const LEDGER_DIR: &str = ".talicode";
/// The append-only ledger file.
pub const LEDGER_FILE: &str = "usage.jsonl";

/// Per-1M-token input/output prices for cost estimation.
#[derive(Debug, Clone, Copy)]
pub struct Price {
    /// USD per 1M input tokens.
    pub input_per_m: f64,
    /// USD per 1M output tokens.
    pub output_per_m: f64,
}

/// Estimated price for a model (defaults to Sonnet-5 rates for unknown models).
pub fn price_for(model: &str) -> Price {
    match model {
        "claude-opus-4-8" | "claude-opus-4-7" => Price { input_per_m: 5.0, output_per_m: 25.0 },
        "claude-haiku-4-5" => Price { input_per_m: 1.0, output_per_m: 5.0 },
        // claude-sonnet-5 and anything unrecognised.
        _ => Price { input_per_m: 3.0, output_per_m: 15.0 },
    }
}

/// Estimated USD cost of `usage` at `price` (uncached input + output only).
pub fn cost_estimate(usage: Usage, price: Price) -> f64 {
    let per = 1_000_000.0;
    (usage.input_tokens as f64 / per) * price.input_per_m
        + (usage.output_tokens as f64 / per) * price.output_per_m
}

/// The one-line footer printed after a sweep.
pub fn footer(usage: Usage, model: &str) -> String {
    let cost = cost_estimate(usage, price_for(model));
    format!(
        "tokens: in {} / out {} (cached {}) · est. ${:.4}",
        usage.input_tokens, usage.output_tokens, usage.cache_read_input_tokens, cost
    )
}

/// A single ledger row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Local date (`YYYY-MM-DD`).
    pub date: String,
    /// Model used.
    pub model: String,
    /// Command that produced the spend (e.g. `sweep`).
    pub command: String,
    /// Uncached input tokens.
    pub input: u64,
    /// Output tokens.
    pub output: u64,
    /// Cache-read tokens.
    pub cache_read: u64,
    /// Cache-write tokens.
    pub cache_creation: u64,
}

impl LedgerEntry {
    /// Build a ledger row for `usage`, stamped with today's local date.
    pub fn today(usage: Usage, model: &str, command: &str) -> Self {
        Self::on(&local_today(), usage, model, command)
    }

    /// Build a ledger row for an explicit date (used by tests).
    pub fn on(date: &str, usage: Usage, model: &str, command: &str) -> Self {
        LedgerEntry {
            date: date.to_string(),
            model: model.to_string(),
            command: command.to_string(),
            input: usage.input_tokens,
            output: usage.output_tokens,
            cache_read: usage.cache_read_input_tokens,
            cache_creation: usage.cache_creation_input_tokens,
        }
    }
}

/// Today's local date as `YYYY-MM-DD`.
pub fn local_today() -> String {
    chrono::Local::now().date_naive().to_string()
}

/// Append a row to `<root>/.talicode/usage.jsonl`. Best-effort: returns Err on
/// failure so the caller can warn, but the caller must not abort a sweep on it.
pub fn append(root: &Path, entry: &LedgerEntry) -> std::io::Result<()> {
    use std::io::Write;
    let dir = root.join(LEDGER_DIR);
    std::fs::create_dir_all(&dir)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(LEDGER_FILE))?;
    let line = serde_json::to_string(entry).map_err(std::io::Error::other)?;
    writeln!(file, "{line}")
}

/// Read all ledger rows (skipping unparseable lines). Missing file ⇒ empty.
pub fn read(root: &Path) -> Vec<LedgerEntry> {
    let path = root.join(LEDGER_DIR).join(LEDGER_FILE);
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// A day's summed totals.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DailyTotal {
    /// Input tokens that day.
    pub input: u64,
    /// Output tokens that day.
    pub output: u64,
}

/// Roll ledger rows up into per-day totals (sorted by date).
pub fn roll_up(entries: &[LedgerEntry]) -> BTreeMap<String, DailyTotal> {
    let mut days: BTreeMap<String, DailyTotal> = BTreeMap::new();
    for e in entries {
        let d = days.entry(e.date.clone()).or_default();
        d.input += e.input;
        d.output += e.output;
    }
    days
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(i: u64, o: u64) -> Usage {
        Usage {
            input_tokens: i,
            output_tokens: o,
            cache_read_input_tokens: 0,
            cache_creation_input_tokens: 0,
        }
    }

    #[test]
    fn cost_estimate_uses_the_price_table() {
        // Sonnet-5: $3/1M in, $15/1M out.
        let c = cost_estimate(usage(1_000_000, 1_000_000), price_for("claude-sonnet-5"));
        assert!((c - 18.0).abs() < 1e-9);
    }

    #[test]
    fn footer_reports_tokens_and_estimate() {
        let f = footer(usage(10, 2), "claude-sonnet-5");
        assert!(f.contains("in 10 / out 2 (cached 0)"));
        assert!(f.contains("est. $"));
    }

    #[test]
    fn append_then_read_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        append(dir.path(), &LedgerEntry::on("2026-07-11", usage(5, 1), "claude-sonnet-5", "sweep"))
            .unwrap();
        append(dir.path(), &LedgerEntry::on("2026-07-11", usage(7, 2), "claude-sonnet-5", "sweep"))
            .unwrap();
        let rows = read(dir.path());
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].input, 7);
    }

    #[test]
    fn roll_up_sums_by_day() {
        let rows = vec![
            LedgerEntry::on("2026-07-10", usage(1, 1), "m", "sweep"),
            LedgerEntry::on("2026-07-11", usage(2, 2), "m", "sweep"),
            LedgerEntry::on("2026-07-11", usage(3, 3), "m", "watch"),
        ];
        let days = roll_up(&rows);
        assert_eq!(days["2026-07-10"], DailyTotal { input: 1, output: 1 });
        assert_eq!(days["2026-07-11"], DailyTotal { input: 5, output: 5 });
    }

    #[test]
    fn read_missing_ledger_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read(dir.path()).is_empty());
    }
}
