// SPDX-License-Identifier: MIT
//! TaliCode agent — the Auditor.
//!
//! Hosts the [`auditor`] module, which turns a file plus a set of rules into
//! line-anchored findings over a [`talicode_core::provider`] backend.

pub mod auditor;
