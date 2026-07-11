// SPDX-License-Identifier: MIT
//! TaliCode core — the AI Slop Gatekeeper engine.
//!
//! Implements #2 (workspace scaffold). Later phases add the provider seam,
//! skill host, auditor, reporting, usage, and watch modules.

pub mod config;

/// The crate version, surfaced by the CLI's `--version`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_nonempty() {
        assert!(!VERSION.is_empty());
    }
}
