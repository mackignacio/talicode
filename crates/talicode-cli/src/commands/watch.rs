// SPDX-License-Identifier: MIT
//! `tali watch` — monitor the current folder/repo and sweep on change.
//!
//! Implements #26. Starts a debounced filesystem watcher over the current
//! directory (the detection engine + debounce live in `talicode_core::watch`),
//! runs an initial sweep, then re-sweeps once per settled burst of edits until
//! `Ctrl-C`. Each sweep reuses the exact `sweep` path — same findings, same
//! usage ledger — so watch is a one-shot on a loop.

use clap::Args as ClapArgs;
use std::time::Duration;
use talicode_core::watch::{self, FsChangeStream, DEFAULT_DEBOUNCE_MS};

use super::sweep;

#[derive(ClapArgs, Debug)]
pub struct Args {
    /// Emit findings as newline-delimited JSON.
    #[arg(long)]
    pub json: bool,
}

pub async fn run(args: Args) -> anyhow::Result<()> {
    let root = std::env::current_dir()?;
    let window = Duration::from_millis(DEFAULT_DEBOUNCE_MS);
    let stream = FsChangeStream::start(&root, window)?;

    eprintln!("tali watch: monitoring {} (Ctrl-C to stop)", root.display());
    sweep_once(args.json).await;

    watch::run(stream, || sweep_once(args.json)).await;
    Ok(())
}

/// Run one sweep over the working tree, printing findings but never exiting the
/// process (watch keeps running). Errors are reported and swallowed so a
/// transient failure doesn't kill the monitor.
async fn sweep_once(json: bool) {
    if let Err(e) = sweep::execute(false, None, json).await {
        eprintln!("watch: sweep error: {e}");
    }
}
