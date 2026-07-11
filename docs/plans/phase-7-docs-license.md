# Phase 7 — Docs, License & Traceability

> Part of the [TaliCode MVP](./MVP.md). Every story below becomes a GitHub issue; every commit references its issue.

## Goal

The four roadmap docs, the README rewrite (positioning / Why TaliCode / Quick Start / MIT / logo), and the traceability matrix that closes the loop (`code-traceability` has real linkage to check).

## User Stories

### US-7.1 — Roadmap docs
**As** a maintainer, **I want** the deferred scope documented, **so that** contributors and users know what's coming and what's commercial.
- **Files:** `docs/roadmaps/ROADMAP-HEAL.md`, `ROADMAP-DEPLOYMENT.md`, `ROADMAP-TALICLOUD.md`, `ROADMAP-TALIAGENTICSERVER.md`.
- **Acceptance criteria:**
  - [ ] HEAL: Surgeon agent, Heal Preview (diff + approve) + Auto-Heal, `--heal` wiring, no-silent-overwrite rationale.
  - [ ] DEPLOYMENT: pre-commit hook + VS Code extension (auto-start on `onStartupFinished`, drives `tali watch`, `.tali`→YAML `contributes.languages`, diagnostics; MIT dev tools).
  - [ ] TALICLOUD: managed cloud platform; proprietary Commercial License text lives here.
  - [ ] TALIAGENTICSERVER: EC2 webhook/agentic daemon (PR Gatekeeper, Jira scaffolder, CI self-heal, team policy); proprietary Commercial License text lives here.

### US-7.2 — README rewrite
**As** a user, **I want** an accurate README, **so that** install and usage match the Rust/npm reality and the brand.
- **Files:** `README.md`.
- **Acceptance criteria:**
  - [ ] Header shows `assets/tali-logo.png`; intro uses the canonical AI-Slop-Gatekeeper positioning.
  - [ ] "Why TaliCode?" uses the refined Tagalog etymology (Tali/Talian, Talikod ×2).
  - [ ] Quick Start: `npm install -g @talicode/core`, `tali init` → `config.tali`, `tali sweep --staged`.
  - [ ] License section trimmed to MIT + pointer that commercial offerings are roadmapped.
  - [ ] Brand `TaliCode` (capital C) in prose; identifiers lowercase (`tali`, `config.tali`, `@talicode/*`).

### US-7.3 — Traceability matrix (`docs/TRACEABILITY.md`)
**As** a maintainer, **I want** a module → issue → phase → MVP map, **so that** every line of code is traceable (satisfies `code-traceability`).
- **Files:** `docs/TRACEABILITY.md`.
- **Acceptance criteria:**
  - [ ] A table mapping each source module to its implementing issue, phase doc, and MVP section.
  - [ ] Every source file carries `// SPDX-License-Identifier: MIT` + a module-level `//! Implements #<issue>` reference.
  - [ ] Kept current as issues are implemented.
