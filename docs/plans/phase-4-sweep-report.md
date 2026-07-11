# Phase 4 — Sweep, Git & Report

> Part of the [TaliCode MVP](./MVP.md). Every story below becomes a GitHub issue; every commit references its issue.

## Goal

Wire the end-to-end detect path: read staged/target files, invoke the skill host + Auditor, render findings with a gating exit code. The `sweep` and `heal` commands become functional.

## User Stories

### US-4.1 — Git staged reader (`git.rs`)
**As** a developer, **I want** staged/target files read, **so that** the sweep knows what to audit.
- **Files:** `crates/talicode-core/src/git.rs`.
- **Acceptance criteria:**
  - [ ] `--staged` → `git diff --cached --name-only --diff-filter=ACM` (via `std::process::Command`).
  - [ ] Without `--staged` → expand the step's `target` glob against the working tree (`globset`).
  - [ ] Skip binary/oversized files (cap ~200 KB) and log what was skipped — no silent truncation.
- **Tests:** staged-file listing; binary/oversize skip.

### US-4.2 — Report renderer (`report.rs`)
**As** a developer, **I want** findings rendered and exit codes set, **so that** output is readable and gate-able.
- **Files:** `crates/talicode-core/src/report.rs`.
- **Acceptance criteria:**
  - [ ] `Finding` + `Severity`; group by file; print `file:line severity rule — message`.
  - [ ] Non-zero exit when any finding at/above threshold; `--json` machine output.
- **Tests:** rendering + exit-code threshold logic.

### US-4.3 — `sweep` command
**As** a developer, **I want** `tali sweep`, **so that** I can audit my staged code.
- **Files:** `crates/talicode-cli/src/commands/sweep.rs`.
- **Acceptance criteria:**
  - [ ] `sweep [--staged] [--skill <name>] [--json]` resolves selected skills via the host, invokes over target files, renders findings.
  - [ ] Bare sweep invokes the `code-review` orchestrator (all default lenses, one verdict).
- **Tests:** command wiring against faked git + provider.

### US-4.4 — `heal` command (surface only)
**As** a developer, **I want** `tali heal` present, **so that** the CLI shape is stable ahead of the healing roadmap.
- **Files:** `crates/talicode-cli/src/commands/heal.rs`.
- **Acceptance criteria:**
  - [ ] Runs `sweep`, then prints "healing not yet enabled" pointing to `docs/roadmaps/ROADMAP-HEAL.md`.
- **Tests:** prints the roadmap notice.
