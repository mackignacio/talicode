# Phase 0 — Scaffold

> Part of the [TaliCode MVP](./MVP.md). Every story below becomes a GitHub issue; every commit references its issue. Traceability: each source file carries an SPDX header + `//! Implements #<issue>`, and a row is added to `docs/TRACEABILITY.md`.

## Goal

Stand up the Cargo workspace, repo hygiene, and CI so later phases have a compiling, linted, tested foundation. No product logic yet — just the skeleton that `cargo build`/`clippy`/`test` pass on.

## User Stories

### US-0.1 — Cargo workspace + crate skeleton
**As** a maintainer, **I want** a two-crate Cargo workspace, **so that** the CLI and core logic are separated and unit-testable.
- **Files:** `Cargo.toml` (workspace), `rust-toolchain.toml`, `crates/talicode-cli/Cargo.toml`, `crates/talicode-cli/src/main.rs` (hello stub), `crates/talicode-core/Cargo.toml`, `crates/talicode-core/src/lib.rs`.
- **Acceptance criteria:**
  - [ ] `talicode-cli` builds a binary named `tali` (`[[bin]] name = "tali"`).
  - [ ] `talicode-cli` depends on `talicode-core`.
  - [ ] Both crates set `license = "MIT"`.
  - [ ] `cargo build` succeeds.
- **Tests:** a trivial `talicode-core` unit test (`lib.rs`) proving the test harness runs.

### US-0.2 — Repo hygiene + logo asset
**As** a maintainer, **I want** `.gitignore`, SPDX header convention, and the logo in `assets/`, **so that** the repo is clean and branded.
- **Files:** `.gitignore` (Rust `target/`, `.talicode/`, node_modules), `assets/tali-logo.png` (moved from repo root), SPDX header on every `.rs` (`// SPDX-License-Identifier: MIT`).
- **Acceptance criteria:**
  - [ ] `tali-logo.png` moved to `assets/tali-logo.png` (tracked).
  - [ ] `.talicode/`, `target/`, `node_modules/` git-ignored.
  - [ ] Every committed `.rs` file starts with the SPDX line.
- **Tests:** n/a (verified by a repo lint / manual check).

### US-0.3 — CI gate (fmt / clippy / test / coverage)
**As** a maintainer, **I want** a CI workflow, **so that** every PR is gated on formatting, lints, tests, and coverage.
- **Files:** `.github/workflows/ci.yml`.
- **Acceptance criteria:**
  - [ ] CI runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.
  - [ ] CI runs `cargo llvm-cov` with a coverage gate (lines + functions).
  - [ ] CI is green on the scaffold.
- **Tests:** the workflow itself (green run).
