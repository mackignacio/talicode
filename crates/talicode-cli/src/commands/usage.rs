// SPDX-License-Identifier: MIT
//! `tali usage` — show token spend (today's total + recent daily history).
//!
//! Implements #25. Reads the repo-local ledger and rolls it up by day. The
//! rendering is a pure function so it is unit-tested; `run` does the IO.

use clap::Args as ClapArgs;
use std::collections::BTreeMap;
use talicode_core::usage::{self, DailyTotal};

/// How many recent days to show by default.
const RECENT_DAYS: usize = 14;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Emit the usage summary as JSON.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let days = usage::roll_up(&usage::read(&root));
    if args.json {
        println!("{}", render_json(&days));
    } else {
        print!("{}", render_human(&days, &usage::local_today(), RECENT_DAYS));
    }
    Ok(())
}

fn render_json(days: &BTreeMap<String, DailyTotal>) -> String {
    let rows: Vec<_> = days
        .iter()
        .map(|(date, t)| serde_json::json!({"date": date, "input": t.input, "output": t.output}))
        .collect();
    serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".to_string())
}

fn render_human(days: &BTreeMap<String, DailyTotal>, today: &str, recent: usize) -> String {
    if days.is_empty() {
        return "No usage recorded yet.\n".to_string();
    }
    let mut out = String::new();
    let today_total = days.get(today).copied().unwrap_or_default();
    out.push_str(&format!(
        "Today ({today}): in {} / out {}\n\nRecent:\n",
        today_total.input, today_total.output
    ));
    // Most-recent `recent` days, newest first.
    for (date, t) in days.iter().rev().take(recent) {
        out.push_str(&format!("  {date}  in {} / out {}\n", t.input, t.output));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn days() -> BTreeMap<String, DailyTotal> {
        let mut m = BTreeMap::new();
        m.insert("2026-07-10".to_string(), DailyTotal { input: 1, output: 1 });
        m.insert("2026-07-11".to_string(), DailyTotal { input: 5, output: 3 });
        m
    }

    #[test]
    fn human_shows_today_and_history() {
        let out = render_human(&days(), "2026-07-11", 14);
        assert!(out.contains("Today (2026-07-11): in 5 / out 3"));
        assert!(out.contains("2026-07-10  in 1 / out 1"));
    }

    #[test]
    fn human_empty_ledger() {
        assert_eq!(render_human(&BTreeMap::new(), "2026-07-11", 14), "No usage recorded yet.\n");
    }

    #[test]
    fn today_absent_shows_zero() {
        let out = render_human(&days(), "2026-07-12", 14);
        assert!(out.contains("Today (2026-07-12): in 0 / out 0"));
    }

    #[test]
    fn json_lists_days() {
        let j = render_json(&days());
        assert!(j.contains("2026-07-11"));
        assert!(j.contains("\"input\": 5"));
    }
}
