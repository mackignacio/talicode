// SPDX-License-Identifier: MIT
//! TaliCode CLI (`tali`) — entry point.
//!
//! Implements #2 (workspace scaffold). The clap command surface and dispatch
//! arrive in #5; for now this proves the `tali` binary builds and links
//! against `talicode-core`.

fn main() {
    println!("TaliCode (tali) v{}", talicode_core::VERSION);
}
