// SPDX-License-Identifier: MIT
//! TaliCode memory — the memory subsystem.
//!
//! Hosts semantic memory ([`memory`]), episodic memory ([`episode`]), working
//! memory ([`context`]), and architectural memory ([`architecture`]) — the
//! stores that let TaliCode recall facts, experiences, and codebase structure.

pub mod architecture;
pub mod context;
pub mod episode;
pub mod memory;
