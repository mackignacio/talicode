// SPDX-License-Identifier: MIT
//! TaliCode skills — the skill runtime host.
//!
//! Hosts the [`host`] module, which discovers skill folders (bundled defaults
//! embedded in the binary plus repo-authored skills), parses them, and runs a
//! selected skill by feeding its guidance and rules to the Auditor.

pub mod host;
