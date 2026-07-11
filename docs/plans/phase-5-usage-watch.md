# Phase 5 — Usage & Watch

> Part of the [TaliCode MVP](./MVP.md). Every story below becomes a GitHub issue; every commit references its issue.

## Goal

Token-spend reporting (per-execution footer + daily ledger), the `usage` command, the `watch` monitor mode, and the `skills` listing.

## User Stories

### US-5.1 — Usage ledger + footer (`usage.rs`)
**As** a developer, **I want** to see token spend, **so that** I can track cost per run and per day.
- **Files:** `crates/talicode-core/src/usage.rs`.
- **Acceptance criteria:**
  - [ ] `Usage` parsed from provider responses (input/output/cache tokens).
  - [ ] Per-execution footer: `tokens: in <n> / out <n> (cached <n>) · est. $<x>` (cost estimate from a per-model price table, overridable in config).
  - [ ] Appends `{date, model, input, output, cache_read, cache_creation, command}` to `.talicode/usage.jsonl`; prints today's cumulative total (local-date bucketed).
  - [ ] Ledger-write failure is non-fatal (warn, continue).
- **Tests:** parse Usage; per-execution sum; daily rollup by date; non-fatal write failure.

### US-5.2 — `usage` command
**As** a developer, **I want** `tali usage`, **so that** I can view today + daily history.
- **Files:** `crates/talicode-cli/src/commands/usage.rs`.
- **Acceptance criteria:**
  - [ ] Prints today's total + last N days from the ledger; `--json` output.
- **Tests:** renders today + history from a seeded ledger.

### US-5.3 — `watch` monitor mode (`watch.rs` + command)
**As** a developer, **I want** `tali watch`, **so that** my current folder/repo is swept on change.
- **Files:** `crates/talicode-core/src/watch.rs`, `crates/talicode-cli/src/commands/watch.rs`.
- **Acceptance criteria:**
  - [ ] `notify` filesystem watcher on CWD + git staged-change detection, debounced (burst → one sweep).
  - [ ] Reuses the `sweep` path; prints findings (NDJSON with `--json`); records usage; `Ctrl-C` exits cleanly.
  - [ ] Scoped to the current folder/repo only.
- **Tests:** debounce collapses a burst into one sweep; change → exactly one sweep (fake watcher + fake provider).

### US-5.4 — `skills` command
**As** a developer, **I want** `tali skills`, **so that** I see my authored skills (bundled hidden).
- **Files:** `crates/talicode-cli/src/commands/skills.rs`.
- **Acceptance criteria:**
  - [ ] Lists only repo `skills/` by default (bundled `code-*` hidden).
  - [ ] `--all` includes bundled, labeled by source.
- **Tests:** repo-only by default; `--all` includes bundled with source labels.
