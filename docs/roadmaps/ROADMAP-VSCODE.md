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
- First-class `config.tali` editing and visibility into memory/skills.

**Non-goals**
- Re-implementing detection in TypeScript.
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

### Phase V3 — Memory & skills surfacing

- **Skills view** — a tree of the active lenses (`tali skills --all`), showing which are bundled vs.
  repo-authored and which the search would inject for the current file.
- **Memory view** — browse semantic facts and the episodic timeline (`tali memory list/timeline
  --json`); add a fact from a selection (`tali memory add`).
- **"Why flagged" hover/detail** — render the finding's rule guidance and any relevant memory context
  the Auditor used, so the verdict is explainable in-editor.

### Phase V4 — Team & platform integration

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

## Relationship to other roadmaps

- The **pre-commit git hook** — the other local, MIT dev-tool integration — lives in
  [ROADMAP-DEPLOYMENT](./ROADMAP-DEPLOYMENT.md).
- The **Heal / Surgeon** mechanism the Quick-Fix action ties into is
  [ROADMAP-HEAL](./ROADMAP-HEAL.md).
- The commercial server products are [ROADMAP-TALICLOUD](./ROADMAP-TALICLOUD.md) and
  [ROADMAP-TALIAGENTICSERVER](./ROADMAP-TALIAGENTICSERVER.md).
