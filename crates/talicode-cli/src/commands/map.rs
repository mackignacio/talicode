// SPDX-License-Identifier: MIT
//! `tali map` — build/refresh and show the codebase architectural map.
//!
//! Implements #52. Scans the repo into `.talicode/architecture.json` (or loads
//! the existing map) and prints the overview the Auditor consults instead of
//! re-grepping. The `tali search` query command and the Claude Code hook that
//! routes agent greps here are roadmap (see docs/roadmaps/ROADMAP-MEMORY.md).

use clap::Args as ClapArgs;
use talicode_memory::architecture;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Rebuild the map from a fresh scan (otherwise reuse the saved one).
    #[arg(long)]
    pub rebuild: bool,
}

pub fn run(args: Args) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let map = match architecture::load(&root) {
        Some(m) if !args.rebuild => m,
        _ => {
            let m = architecture::scan(&root);
            architecture::save(&root, &m)?;
            m
        }
    };
    print!("{}", architecture::overview(&map));
    Ok(())
}
