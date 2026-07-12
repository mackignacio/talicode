// SPDX-License-Identifier: MIT
//! TaliCode core — the AI Slop Gatekeeper engine.
//!
//! Implements #2 (workspace scaffold). Provides the shared foundation — the
//! provider seam, configuration, git integration, reporting, usage, and watch
//! modules — depended on by the agent, skills, and memory crates.

pub mod config;
pub mod git;
pub mod provider;
pub mod report;
pub mod usage;
pub mod watch;

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
