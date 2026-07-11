// SPDX-License-Identifier: MIT
//! `tali watch` — monitor the current folder/repo and sweep on change.
//!
//! Implements #5 (command surface). The `notify` watcher + debounce lands in #26.

use clap::Args as ClapArgs;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Emit findings as newline-delimited JSON.
    #[arg(long)]
    pub json: bool,
}

pub fn run(_args: Args) -> anyhow::Result<()> {
    println!("tali watch: not yet implemented (see #26)");
    Ok(())
}
