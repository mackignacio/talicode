# Phase 2 — Provider & Auditor

> Part of the [TaliCode MVP](./MVP.md). Every story below becomes a GitHub issue; every commit references its issue.

## Goal

The `Provider` trait seam (with a fake for tests), the Anthropic Messages-API implementation with forced tool-use output + `Usage`, and the Auditor that turns a file + rules into `Vec<Finding>`.

## User Stories

### US-2.1 — `Provider` trait + registry + fake
**As** a maintainer, **I want** a provider abstraction with a test double, **so that** the LLM is swappable and unit tests need no network.
- **Files:** `crates/talicode-core/src/provider/mod.rs`.
- **Acceptance criteria:**
  - [ ] Async `Provider` trait: one method taking messages + a JSON-schema tool def, returning validated structured output **and** `Usage`.
  - [ ] Registry resolves a provider by config `provider` name; errors on unknown.
  - [ ] An in-process `FakeProvider` returns canned structured output + usage for tests.
- **Tests:** registry resolves known / errors on unknown; fake returns expected output.

### US-2.2 — Anthropic provider (Messages API)
**As** a developer, **I want** the Anthropic provider, **so that** TaliCode can call Claude with a forced findings schema.
- **Files:** `crates/talicode-core/src/provider/anthropic.rs`.
- **Acceptance criteria:**
  - [ ] `reqwest` call to `POST /v1/messages` with tool-use forcing the findings schema.
  - [ ] Default model `claude-sonnet-5`, `output_config.effort` default `"medium"` (both overridable from config).
  - [ ] Parses `usage` (input/output/cache tokens) into `Usage`.
  - [ ] Missing `ANTHROPIC_API_KEY` → clear error.
- **Tests:** maps a mocked tool-use response → structured output (HTTP faked with `wiremock`); missing-key error path.

### US-2.3 — Auditor agent
**As** a developer, **I want** the Auditor, **so that** a file + rules yields line-anchored findings.
- **Files:** `crates/talicode-core/src/auditor.rs`, `crates/talicode-core/src/report.rs` (Finding type only, if not yet present).
- **Acceptance criteria:**
  - [ ] Input: file path, line-numbered content, agent `role` + rules.
  - [ ] Builds the prompt + tool schema; returns `Vec<Finding>` (`{file, line, severity, rule, message}`).
  - [ ] Prompt instructs "concrete, line-anchored violations; prefer silence over speculation."
- **Tests:** builds prompt/tool schema; returns findings from a fake provider; empty-findings path.
