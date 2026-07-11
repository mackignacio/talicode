// SPDX-License-Identifier: MIT
//! `tali sweep` — detect AI slop / violations in staged or target files.
//!
//! Implements #5 (command surface). The detect path (git → skill host →
//! auditor → report) is wired in #22.

use clap::Args as ClapArgs;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Only audit files staged in git.
    #[arg(long)]
    pub staged: bool,
    /// Run only this skill, overriding the config selection.
    #[arg(long)]
    pub skill: Option<String>,
    /// Emit findings as machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

pub fn run(_args: Args) -> anyhow::Result<()> {
    println!("tali sweep: not yet implemented (see #22)");
    Ok(())
}
