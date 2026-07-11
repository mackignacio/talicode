// SPDX-License-Identifier: MIT
//! `tali heal` — run a sweep, then point at the healing roadmap.
//!
//! Implements #23. Healing (the Surgeon agent that rewrites sloppy code) is
//! roadmap, not the MVP. `heal` runs a staged detect-only sweep so the command
//! is useful today, then prints where healing is designed. The command exists
//! so the CLI surface is stable ahead of the healing work.

const HEAL_NOTICE: &str = "\nHealing is not yet enabled — the findings above are detect-only.\n\
     The Surgeon (auto-fix) agent is designed in docs/roadmaps/ROADMAP-HEAL.md.";

pub async fn run() -> anyhow::Result<()> {
    let code = crate::commands::sweep::execute(true, None, false).await?;
    println!("{HEAL_NOTICE}");
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notice_points_at_the_healing_roadmap() {
        assert!(HEAL_NOTICE.contains("ROADMAP-HEAL.md"));
        assert!(HEAL_NOTICE.contains("not yet enabled"));
    }
}
