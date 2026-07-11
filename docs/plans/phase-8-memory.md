# Phase 8 — Long-Term Memory (five-type architecture)

> Part of the [TaliCode MVP](./MVP.md). Every story below becomes a GitHub issue; every commit references its issue.

## Goal

Give the Auditor **long-term memory**, built natively in Rust (lean, ~zero new dependencies),
following a five-type cognitive architecture so the gatekeeper judges in context and stops starting
cold:

1. **Working memory** — the in-RAM per-turn assembler + a 250K-soft / 500K-hard conversation budget
   (over all inputs + output − semantic tokens) that, when the coding LLM finishes, compresses the
   conversation into episodic memory and fires the skill synthesizer — never interrupting work.
2. **Semantic memory** — durable, timeless project facts as markdown files, injected once per session
   / after compression, versioned as a content-addressed delta chain.
3. **Procedural memory** — the skill host as native, on-demand skill search: no resident skill index,
   only matching skills injected (or "no skill needed"), with an `always_run_skills` security floor.
4. **Episodic memory** — the long-term store of learnings / mistakes / experiences / summaries
   (durable/scratch tiers, supersedes/contradicts/related_to links, hybrid keyword+recency ranking,
   per-tier `build_context`); recurring experiences auto-promote into skills.
5. **Architectural memory** — a queryable map of the codebase (tree, symbols, imports) whose overview
   is injected into context and which is refreshed before each compression.

Heavy backends (vector DB / embeddings, SQLite+FTS5, knowledge graph, JIT/agentic skill pull-in,
LLM compression/synthesis, the `tali search` codebase-search command + `tali hook` Claude Code hook,
AST-accurate mapping) are deferred to `docs/roadmaps/ROADMAP-MEMORY.md`.

## User Stories

### US-8.1 — Semantic memory (`memory.rs`)
**As** the Auditor, **I want** durable project facts, **so that** I review with the team's context.
- **Files:** `crates/talicode-core/src/memory.rs`.
- **Acceptance criteria:**
  - [ ] Facts are markdown files (YAML frontmatter `{ slug, tier, created, tags }` + body) under
        `.talicode/memory/`, committed; tolerant `read` (missing dir ⇒ empty).
  - [ ] `store`/`forget` IO; pure `rank` (keyword + recency) and `build_section` (`""` when empty).
- **Tests:** store/read round-trip (tempdir); rank ordering; empty ⇒ no section.

### US-8.2 — Episodic memory (`episode.rs`)
**As** the Auditor, **I want** to recall past experiences, **so that** I stay consistent over time.
- **Files:** `crates/talicode-core/src/episode.rs`.
- **Acceptance criteria:**
  - [ ] `Episode { id, content, memory_type: learning|mistake|experience|summary, tier:
        durable|scratch, tags, created, accessed, access_count, expires_at, links }` at
        `.talicode/episodes.jsonl`.
  - [ ] Tiers (scratch `expires_at` TTL, hidden past expiry); links supersedes/contradicts/related_to
        + `supersede`; IO `record`/`read`/`forget`/`prune` (dry-run default).
  - [ ] Pure `summarize`, `rank` (`w_fts·sigmoid(−BM25) + w_recency·exp(−0.693·days/half_life)`),
        `build_context` (per-tier budgets + conflicts + timeline).
- **Tests:** round-trip; rank; expiry filter; supersede link; build_context grouping.

### US-8.3a — Episodic → procedural auto-promotion
**As** a developer, **I want** repeated preferences to become skills automatically, **so that** I'm
not re-flagged.
- **Files:** `crates/talicode-core/src/episode.rs` (promotion) + skill templating.
- **Acceptance criteria:**
  - [ ] Pure `due_for_skill(episodes, threshold)`; IO `promote(root, draft)` writes a templated
        `skills/<slug>/` (SKILL.md + rules.yaml) with no prompt.
- **Tests:** threshold boundary; generated skill parses through `host::skill`.

### US-8.3 — Procedural native skill retrieval (`host/retrieve.rs`)
**As** the host, **I want** to inject only relevant skills, **so that** token spend drops.
- **Files:** `crates/talicode-core/src/host/retrieve.rs`, `host/invoke.rs`.
- **Acceptance criteria:**
  - [ ] `skill_search(catalog, query, limit)` keyword/grep, ranked, empty ⇒ "no skill needed".
  - [ ] `compose_guidance` over matches only; `always_run_skills` floor; `skill_retrieval: search|all`.
- **Tests:** match/no-match; floor always present.

### US-8.3b — Architectural memory (`architecture.rs`)
**As** the Auditor, **I want** a codebase map, **so that** I look up structure instead of grepping.
- **Files:** `crates/talicode-core/src/architecture.rs`.
- **Acceptance criteria:**
  - [ ] IO `scan`/`save`/`load` (`.talicode/architecture.json`, committed); pure `build`,
        `arch_lookup`, `overview`; heuristic per-extension extraction (no AST).
- **Tests:** build map; lookup; overview.

### US-8.4 — Working memory (`context.rs`)
**As** TaliCode, **I want** budgeted context assembly + compression, **so that** memory scales.
- **Files:** `crates/talicode-core/src/context.rs`.
- **Acceptance criteria:**
  - [ ] `WorkingContext::assemble` under `context_budget_tokens`; 250K-soft/500K-hard budget over
        `all_inputs + output − semantic_tokens`.
  - [ ] Pure `should_compress(spent, soft, hard, llm_active)` (defer while active); `compress`
        (semantic stripped); content-addressed semantic `diff`/`resolve` chain (FNV-1a, deltas only).
- **Tests:** soft/hard/active boundaries; semantic-exclusion; delta-only + chain resolve.

### US-8.5 — `MemoryConfig` (`config.rs`)
- **Files:** `crates/talicode-core/src/config.rs`.
- **Acceptance criteria:**
  - [ ] Optional `memory:` section, all fields defaulted; existing configs load unchanged.
- **Tests:** defaults; back-compat parse.

### US-8.6 — Wire memory into the Auditor
- **Files:** `auditor.rs`, `host/invoke.rs`, `commands/sweep.rs`.
- **Acceptance criteria:**
  - [ ] `memory: String` through `AuditRequest` → `system_prompt` → `invoke_file`/`invoke_files`;
        `sweep::execute` assembles context + records the episode. Empty stores ⇒ zero behavior change.
- **Tests:** `system_prompt` includes the block; invoke passes it through (fake provider).

### US-8.7 — `tali memory` + `tali map` commands
- **Files:** `crates/talicode-cli/src/commands/memory.rs`, `commands/map.rs`, `main.rs`.
- **Acceptance criteria:**
  - [ ] Semantic `add`/`list`/`search`/`forget`; episodic `remember --type …`/`recall`/`timeline`/
        `supersede`/`prune`; `tali map [--rebuild]`; `--json`.
- **Tests:** pure render + search/rank.

### US-8.8 — `init` seeds `memory:` + git-ignore granularity
- **Files:** `crates/talicode-cli/src/commands/init.rs`.
- **Acceptance criteria:**
  - [ ] Starter config gains a commented `memory:` block; `.gitignore` ignores `.talicode/usage.jsonl`
        + `.talicode/episodes.jsonl` (local) so `.talicode/memory/` (semantic) is committed.
- **Tests:** update init tests.

### US-8.9 — Docs
- **Files:** `docs/roadmaps/ROADMAP-MEMORY.md`, `README.md`, `docs/TRACEABILITY.md`.
- **Acceptance criteria:**
  - [ ] `ROADMAP-MEMORY.md` (per-type deferred upgrades incl. `tali search`/`tali hook`); README
        "Memory" subsection; TRACEABILITY rows for the new modules.
