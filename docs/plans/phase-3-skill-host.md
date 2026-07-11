# Phase 3 — Skill Host & Bundled Skills

> Part of the [TaliCode MVP](./MVP.md). Every story below becomes a GitHub issue; every commit references its issue.

## Goal

The skill runtime (parse, discover, invoke — "TaliCode acts as Claude") and the 22 bundled `code-*` skills (21 lenses + the `code-review` orchestrator), embedded into the binary. Each skill = `SKILL.md` (frontmatter + guidance) + `rules.yaml`.

## Host stories

### US-3.1 — Skill model & parser (`skill.rs`)
**As** a maintainer, **I want** to parse a skill folder, **so that** SKILL.md + rules.yaml become a typed `Skill`.
- **Files:** `crates/talicode-core/src/host/skill.rs`, `crates/talicode-core/src/host/mod.rs`.
- **Acceptance criteria:**
  - [ ] `Skill` serde model; parse `SKILL.md` frontmatter (`serde_yaml`) + load `rules.yaml`.
  - [ ] A skill is a **lens** (has `rules.yaml`) or an **orchestrator** (SKILL.md declares a `runs:` list, no rules).
  - [ ] Malformed skills rejected with a clear error.
- **Tests:** parses valid lens + orchestrator; rejects malformed.

### US-3.2 — Discovery + embed + override (`discover.rs`)
**As** a developer, **I want** bundled defaults embedded and repo skills to override them, **so that** the binary is self-contained but customizable.
- **Files:** `crates/talicode-core/src/host/discover.rs`.
- **Acceptance criteria:**
  - [ ] Embed `crates/talicode-core/assets/skills/` via `rust-embed`.
  - [ ] Discover from embedded defaults + repo-root `skills/`; repo overrides embedded by `name`.
  - [ ] Unknown selected skill → clear error.
- **Tests:** finds embedded + repo; override on name; unknown errors.

### US-3.3 — Invoke + orchestrator expansion (`invoke.rs`)
**As** a developer, **I want** selected skills invoked and aggregated, **so that** a sweep runs the right lenses and renders one verdict.
- **Files:** `crates/talicode-core/src/host/invoke.rs`.
- **Acceptance criteria:**
  - [ ] Expand orchestrator skills to their `runs:` lenses.
  - [ ] Compose resolved lenses' guidance/rules into the Auditor call; return `Vec<Finding>` tagged with skill/rule id; aggregate one verdict for an orchestrator run.
  - [ ] Findings de-duplicated by file+line.
- **Tests:** composes via fake provider; expands `code-review` `runs:`; `code-clear-exit`+`code-early-return` compose.

## Bundled-skill stories (author `SKILL.md` + `rules.yaml`, each validated by a data test)

### US-3.4 — Foundational lenses
`code-think-twice`, `code-kiss`, `code-yagni`, `code-dry`.

### US-3.5 — Control-flow lenses
`code-early-return`, `code-clear-exit`, `code-no-nested-loop`, `code-bounded-loops`, `code-bounded-recursion`, `code-deterministic-concurrency`. (Definitions per MVP.md — bounded-loops/recursion allow the construct but require a finite/verifiable bound; deterministic-concurrency forbids sleep/setTimeout as sync; clear-exit composes with early-return.)

### US-3.6 — Design lenses
`code-composition`, `code-solid`, `code-smells`, `code-nitpick`, `code-rules` (NASA Power-of-10, language-agnostic).

### US-3.7 — Literal & secret lenses
`code-magic-strings`, `code-magic-numbers` (with the "not magic" exceptions), `code-no-keys`, `code-no-credentials`.

### US-3.8 — Traceability + aviation
`code-traceability` (best-effort, no cert overclaim), `code-aviation` (opt-in strict DO-178C superset; not in default runs).

### US-3.9 — `code-review` orchestrator
`code-review/` with a `runs:` list = all lenses **except** `code-aviation`. Default sweep target.

**Common acceptance criteria (US-3.4–3.9):** each skill folder has a valid `SKILL.md` (frontmatter `name`+`description` + guidance) and `rules.yaml`; each is validated by a data test loading it through `host/skill.rs`; all 22 covered by the end of the phase.
