// SPDX-License-Identifier: MIT
//! TaliCode's skill runtime — "TaliCode acts as Claude".
//!
//! The host discovers skill folders (bundled defaults embedded in the binary
//! plus repo-authored skills), parses them ([`skill`]), and runs a selected
//! skill by feeding its guidance + rules to the Auditor over the provider.
//!
//! Implements #11 (the host module root): parsing lands in [`skill`] (#11),
//! discovery in [`discover`] (#12), and invocation in [`invoke`] (#13).

pub mod discover;
pub mod invoke;
pub mod retrieve;
pub mod skill;
