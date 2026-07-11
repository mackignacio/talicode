// SPDX-License-Identifier: MIT
//! `tali skills` — list user-authored repo skills (bundled hidden by default).
//!
//! Implements #5 (command surface). The listing logic lands in #27.

use clap::Args as ClapArgs;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Include the bundled `code-*` defaults, labeled by source.
    #[arg(long)]
    pub all: bool,
}

pub fn run(_args: Args) -> anyhow::Result<()> {
    println!("tali skills: not yet implemented (see #27)");
    Ok(())
}
