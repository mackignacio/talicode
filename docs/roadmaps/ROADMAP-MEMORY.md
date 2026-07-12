# Roadmap — Long-Term Memory (deferred upgrades)

> Deferred design, part of the [TaliCode MVP](../plans/MVP.md) — see also the
> [phase-8 plan](../plans/phase-8-memory.md). The MVP ships a lean, zero-dependency version of the
> five-type memory architecture (JSONL/markdown stores, keyword+recency ranking, a heuristic
> codebase scan, FNV-1a content addressing, templated skill synthesis). This document collects the
> heavier upgrades intentionally left out of the MVP.

TaliCode's memory has five types — **working**, **semantic**, **procedural**, **episodic**, and
**architectural**. Each has a deferred, richer form:

## Episodic memory — full-fidelity store

The MVP stores episodes as JSONL and ranks with the keyword + recency half of the hybrid formula
(`score = w_fts·relevance + w_recency·decay`). The roadmap completes it:

- **SQLite + FTS5 (BM25)** storage — real full-text ranking instead of the keyword approximation.
- **Local ONNX embeddings + hybrid vector/keyword ranking** — the `w_vec·(1 − cosine)` term, so
  recall matches *intent*, not just shared words.
- **Introspection tools** — similar / topics / timeline / conflicts / health over the store.
- **Scratch-TTL garbage collection** on start, and **LLM-compressed narrative episodes** (the MVP
  writes structured summaries).

## Semantic memory — richer backends

- **Knowledge graph + vector DB** backends beyond the markdown files, for relationship-aware recall.
- A robust **Merkle / CID content-addressed snapshot store**, hardening the MVP's FNV-1a delta chain
  (the "blockchain for LLM memory") into a tamper-evident, dedup-perfect history.

## Procedural memory — smarter retrieval

- **Semantic / embedding skill search** — match a skill to a file by meaning, not just keywords.
- **JIT / agentic "pull in as necessary"** — the Auditor requests a skill's full body on demand via
  a tool loop (needs the agentic upgrade noted in [ROADMAP-HEAL](./ROADMAP-HEAL.md)).

## Working memory — LLM compression & synthesis

- **LLM conversation compression** at the 250K soft / 500K hard boundary (the MVP writes a
  structured summary).
- **Summarize-to-fit** context assembly, and **LLM pattern-synthesis of skills** — detecting
  recurring preferences from the conversation itself (the MVP promotes only explicitly-recorded
  recurring experiences via a template) and authoring **rich** promoted skills.

## Architectural memory — codebase search & the Claude Code hook

The MVP builds the map (`tali map`) and injects a compact overview. The roadmap adds the query
surface and integrations:

- **`tali search "<query>"`** — a codebase-search command answering file/symbol/module lookups from
  architectural memory instead of grepping the filesystem (the token-saving, fast grep replacement).
- **`tali hook install | uninstall | status`** — a Claude Code `PreToolUse` hook (in
  `.claude/settings.json`, matching `Grep`/`Glob`) that routes Claude Code / coding-agent codebase
  searches to `tali search` automatically.
- **AST-accurate symbols**, a **call/dependency graph**, **semantic (embedding) code search**, and
  **incremental updates** on file change (via `tali watch`) — beyond the MVP's heuristic scan.

## Out of scope entirely (for now)

Finding-suppression baselines and any hard dependency on SQLite/embeddings in the MVP build — all
of the above are additive and keep the single self-contained `tali` binary until a deliberate
opt-in.
