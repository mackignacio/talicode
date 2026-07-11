// SPDX-License-Identifier: MIT
//! TaliCode's skill runtime — "TaliCode acts as Claude".
//!
//! The host discovers skill folders (bundled defaults embedded in the binary
//! plus repo-authored skills), parses them ([`skill`]), and runs a selected
//! skill by feeding its guidance + rules to the Auditor over the provider.
//! Discovery lands in #12 and invocation in #13.

pub mod skill;
