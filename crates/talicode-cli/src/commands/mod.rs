// SPDX-License-Identifier: MIT
//! CLI command handlers.
//!
//! Implements #5. Each submodule owns one `tali` subcommand: its clap `Args`
//! (where it takes flags) and a `run` entry point.

pub mod heal;
pub mod init;
pub mod map;
pub mod memory;
pub mod skills;
pub mod sweep;
pub mod usage;
pub mod watch;
