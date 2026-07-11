// SPDX-License-Identifier: MIT
//! `tali heal` — run a sweep, then point at the healing roadmap.
//!
//! Implements #5 (command surface). The real behaviour (run sweep + print the
//! "healing not yet enabled" notice) lands in #23.

pub fn run() -> anyhow::Result<()> {
    println!("tali heal: not yet implemented (see #23)");
    Ok(())
}
