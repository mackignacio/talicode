# Roadmap — VS Code Extension

> Deferred design, part of the TaliCode MVP (see [../plans/MVP.md](../plans/MVP.md)). The extension
> is a **thin presentation layer over the shipped `tali` CLI** — it launches the modes the MVP
> already provides (`tali watch`, `tali sweep --staged --json`) and renders their output. It does
> **not** re-implement any detection, and it never re-derives policy: the CLI's findings, severity
> gate, and exit codes are the single source of truth.

TaliCode is an "AI Slop Gatekeeper": a compiled Rust CLI (`tali`), distributed via npm as the
unscoped `talicode` package (a launcher whose postinstall downloads the matching native binary — see
[ROADMAP-MEMORY](./ROADMAP-MEMORY.md) and the README for the distribution model). This document is the
full design for the **MIT-licensed** VS Code extension that wraps it.

---

## Principles

- **Thin client, zero duplicated logic.** Every finding, every block/allow decision, and every fix
  comes from the CLI. The extension only *starts* the CLI and *renders* what it returns.
- **One requirement: install the extension.** Everything else degrades gracefully (missing binary,
  missing API key, watcher crash) into a dismissible notice — never a hard failure on window open.
- **Per-window isolation.** Each VS Code window runs its own extension host, so opening N windows
  monitors N workspaces independently — one watcher per workspace, no shared global daemon.

## Goals / non-goals

**Goals**
- Surface TaliCode findings inline as you type/save, with the same verdict a `tali sweep` would give.
- Make the healing flow (see [ROADMAP-HEAL](./ROADMAP-HEAL.md)) reachable from a diagnostic.
- Provide an **in-editor chat** — a Claude-Code-style conversational agent, driven by TaliCode's own
  agents, memory, and skills.
- Give users **cost control** — a settings panel to turn subsystems (skills, agent, memory) on/off, a
  token-usage dashboard, and per-component model routing — all editing `config.tali`.
- First-class `config.tali` editing and visibility into memory/skills.

**Non-goals**
- Re-implementing detection — or the agent loop, or model integration — in TypeScript. The chat
  drives TaliCode's agents through the CLI; it does not embed its own provider client.
- Editors other than VS Code (JetBrains, Neovim) — separate future work.
- CI / server-side gating — that lives with the commercial server roadmaps
  ([ROADMAP-TALICLOUD](./ROADMAP-TALICLOUD.md), [ROADMAP-TALIAGENTICSERVER](./ROADMAP-TALIAGENTICSERVER.md)).

---

## Phased plan

### Phase V1 — MVP extension (diagnostics)

The smallest useful extension: watch the workspace and paint findings.

- **Activation** — `activationEvents: ["onStartupFinished"]`; no command to run.
- **Binary resolution** — resolve `tali` in order: (1) the `talicode` npm package's launcher
  (`bin/tali.js`, which execs the downloaded native binary), (2) a workspace-local install, (3) `tali`
  on `PATH`. A configurable `talicode.path` override wins over all.
- **Start `tali watch`** for each workspace folder and stream its `--json` output.
- **Findings → diagnostics** — map each `{ file, line, severity, rule, message }` to a VS Code
  `Diagnostic` (severity-mapped squiggle) in the editor and the Problems panel; the `rule` id becomes
  the diagnostic code, linking back to the lens.
- **`config.tali` support** — register the `.tali` extension as **YAML** (syntax highlighting +
  indentation) and contribute a **file icon** so it's recognizable in the Explorer.
- **Failure posture** — missing binary / missing API key / watcher failure each surface a single
  dismissible notice; the extension otherwise stays quietly inactive.

### Phase V2 — Interaction (commands, status, healing)

- **Quick-Fix "Heal" `CodeAction`** on each diagnostic, wired to the healing flow in
  [ROADMAP-HEAL](./ROADMAP-HEAL.md) (Heal Preview → diff + approve). The extension offers the action;
  the heal roadmap owns the mechanism. TaliCode never silently overwrites code.
- **Command palette** — `TaliCode: Sweep Staged`, `Sweep File`, `Rebuild Architecture Map`
  (`tali map --rebuild`), `Open config.tali`, `Restart Watcher`.
- **Status-bar item** — sweep state (clean / N findings / running) plus today's token spend from
  `tali usage --json`; click to open the Problems panel or the usage summary.
- **Save-gated sweep option** — optionally run `tali sweep` on save (in addition to the watcher) for
  immediate feedback on the just-saved file.

### Phase V3 — Chat (in-editor conversational agent)

A **Claude-Code-style chat panel** in the sidebar — an interactive way to ask TaliCode about the
codebase, request reviews, and apply fixes conversationally. It is a *front-end to TaliCode's own
agent*, not a second assistant: the extension does not embed a provider client or a model loop, it
talks to an interactive `tali` agent mode (see the note below) so the CLI stays the single source of
truth for prompts, provider seam, memory, and skills.

- **Conversational agent** — a webview chat backed by the `config.tali` agent(s) over the existing
  provider seam (Anthropic Messages API, streamed responses).
- **Grounded in TaliCode's memory** — answers draw on **semantic** facts, **episodic** history, and
  the **architectural map** (`tali map`) so the agent reasons from the project's own context and
  looks up structure instead of re-grepping; the conversation is bounded by the working-memory budget
  and compressed into episodic memory when it ends (so "what did we do last time?" carries over).
- **Skill-aware** — the procedural search injects only the relevant `code-*` lenses for what's being
  discussed; the chat can explain a finding ("why was this flagged?") from the rule guidance.
- **Agentic actions with safe apply** — ask it to sweep, explain, refactor, or fix; proposed edits
  are applied through the **Heal** flow (diff + approve), never written silently. This depends on the
  agentic tool-loop upgrade shared with [ROADMAP-HEAL](./ROADMAP-HEAL.md).
- **Context attach** — `@`-mention files/symbols (resolved via the architectural map), add the current
  selection, or drag a diagnostic into the chat as context.
- **Session history** — conversations persist (and summarize) into episodic memory, so past chats are
  recallable and inform later reviews.

> **CLI dependency:** the chat needs an interactive/agent surface on the CLI (e.g. a `tali agent` /
> `tali chat` mode exposing a streamed, tool-enabled loop over stdio or a local socket). Designing
> that surface is a prerequisite and is tracked alongside the agentic upgrade in
> [ROADMAP-HEAL](./ROADMAP-HEAL.md); the extension is purely its UI.

### Phase V4 — Control panel: settings, cost control & model routing

A settings surface for managing what TaliCode runs and what it costs. Every toggle is a **projection
of `config.tali`** — the panel reads and writes that file, so the CLI stays authoritative and the
choices are committable and shareable.

- **Settings UI — turn subsystems on/off** (to bound token spend):
  - **Skills** — enable/disable individual `code-*` lenses, switch `skill_retrieval` between `search`
    (only matched lenses injected) and `all`, and edit the always-run security floor.
  - **Agent** — enable/disable the auditor and set its effort level.
  - **Memory** — toggle the whole `memory:` block or individual types (working, semantic, procedural,
    episodic, architectural), with the budgets (`context_budget_tokens`, soft/hard) editable inline.
- **Token-usage dashboard** — visualize the `tali usage --json` ledger: today's spend, daily history,
  and a breakdown by command and model with the cost estimate, so the toggles above are informed by
  what's actually driving spend.
- **Per-component model routing** — a tab to choose the inference model **per agent, per skill, and
  per memory operation** (e.g. a cheap model for memory summarization, a stronger one for the
  auditor), written as per-component `model` overrides in `config.tali`. *(Requires a config-schema
  extension to accept a `model` override at the skill and memory-operation level, not just the
  agent.)*
- **Browse views** (explainability, alongside the controls):
  - **Skills view** — a tree of the active lenses (`tali skills --all`), bundled vs. repo-authored,
    and which the search would inject for the current file.
  - **Memory view** — browse semantic facts and the episodic timeline (`tali memory list/timeline
    --json`); add a fact from a selection (`tali memory add`).
  - **"Why flagged" hover/detail** — the finding's rule guidance plus the memory context the Auditor
    used, so the verdict is explainable in-editor.

### Phase V5 — Team & platform integration

- **Settings sync & multi-root** — coherent behavior across multi-root workspaces and synced settings.
- **Cloud hooks** — optional integration points for the commercial products
  ([TaliCloud](./ROADMAP-TALICLOUD.md), [TaliAgenticServer](./ROADMAP-TALIAGENTICSERVER.md)) — e.g.
  showing server-side gate results alongside local ones. These remain opt-in and separate from the
  MIT extension core.

---

## Architecture

- **Language:** TypeScript, bundled (esbuild) into a single extension.
- **Process model:** one long-lived `tali watch` child per workspace folder, communicating over its
  `--json` stream; short-lived `tali sweep`/`tali map`/`tali usage` invocations for on-demand actions.
- **Protocol:** the CLI's existing `--json` output is the contract. A `tali --version` handshake lets
  the extension detect binary/version skew and degrade rather than misparse.
- **No detection state in the extension** — it holds only a diagnostics cache keyed by file, rebuilt
  from CLI output.

## Packaging & distribution

- Published to the **VS Code Marketplace** (and Open VSX), **MIT-licensed** — free to install,
  inspect, and fork.
- The extension does **not** bundle the Rust binary; it depends on the `talicode` CLI being installed
  (and points the user at `npm install -g talicode` when it isn't). This keeps the extension small and
  the binary distribution in one place.

---

## Open questions

- **Binary/version skew** — how the extension should react when the resolved `tali` version differs
  from the protocol it expects (warn, disable, or best-effort parse).
- **Watcher lifecycle** — restart/backoff policy when `tali watch` exits unexpectedly, and how (or
  whether) to surface repeated crashes without being noisy.
- **Multi-root attribution** — one watcher per folder vs. per window, and how diagnostics are
  attributed when folders overlap.
- **Notice fatigue** — how often to re-surface the dismissible missing-binary / missing-key notices
  before staying silent.
- **Diagnostics for unsaved buffers** — whether to sweep dirty editors (via a temp file) or only
  saved files.
- **Chat transport** — how the extension drives the interactive agent: a long-lived `tali agent`
  process over stdio, a local socket, or JSON-RPC — and how streaming tokens and tool calls are
  framed on that channel.
- **Chat ↔ episodic memory** — when a conversation is compressed into episodic memory, how much is
  kept vs. summarized, and how the user browses/prunes past chat sessions.
- **Edit application** — whether chat-proposed edits always route through Heal Preview (diff +
  approve) or allow a trusted auto-apply mode, and how multi-file edits are staged.
- **Per-component model schema** — how `config.tali` should express a model override per skill and
  per memory operation (not just per agent), and sensible defaults so a skill/memory without an
  explicit model inherits the agent's.

## Relationship to other roadmaps

- The **pre-commit git hook** — the other local, MIT dev-tool integration — lives in
  [ROADMAP-DEPLOYMENT](./ROADMAP-DEPLOYMENT.md).
- The **Heal / Surgeon** mechanism the Quick-Fix action ties into is
  [ROADMAP-HEAL](./ROADMAP-HEAL.md).
- The commercial server products are [ROADMAP-TALICLOUD](./ROADMAP-TALICLOUD.md) and
  [ROADMAP-TALIAGENTICSERVER](./ROADMAP-TALIAGENTICSERVER.md).
