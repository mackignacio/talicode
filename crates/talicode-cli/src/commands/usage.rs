// SPDX-License-Identifier: MIT
//! `tali usage` — show token spend (today's total + recent daily history).
//!
//! Implements #5 (command surface). The ledger rollup lands in #25.

use clap::Args as ClapArgs;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Emit the usage summary as JSON.
    #[arg(long)]
    pub json: bool,
}

pub fn run(_args: Args) -> anyhow::Result<()> {
    println!("tali usage: not yet implemented (see #25)");
    Ok(())
}
