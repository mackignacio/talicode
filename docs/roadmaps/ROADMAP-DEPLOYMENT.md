# Roadmap — Local Deployment (pre-commit hook + VS Code extension)

> Deferred design, part of the TaliCode MVP (see [../plans/MVP.md](../plans/MVP.md)). These are local developer integrations that are **designed but not yet built** — they wrap the CLI the MVP already ships (`tali sweep --staged`, `tali watch`) rather than re-implementing any detection.

TaliCode is an "AI Slop Gatekeeper": a compiled Rust CLI (`tali`), distributed via npm as `@talicode/core`, that detects AI-generated code problems before they reach a commit. This document covers the two local dev-tool integrations that sit on top of that CLI.

---

## Pre-commit git hook

A git `pre-commit` hook that wraps `tali sweep --staged` and **gates the commit on the CLI's exit code**. This reuses the exact exit-code contract the CLI already exposes — no new detection logic, no duplicated policy.

- **Behavior**: on `git commit`, the hook runs `tali sweep --staged` against the staged changes. When `tali` exits **non-zero** (a finding at or above the severity threshold configured in `config.tali`), the hook aborts the commit; on a clean pass (exit `0`) the commit proceeds.
- **Severity gate**: the block/allow decision is entirely the CLI's — the hook only forwards the exit code. Raising or lowering the threshold is a `config.tali` change, not a hook change.
- **Opt-in per repo**: the hook is never installed globally or silently. A developer (or repo maintainer) enables it deliberately for a given working copy.
- **Installation** — two supported paths, both documented:
  - A `tali` subcommand (e.g. `tali hook install`) that writes the `pre-commit` script into `.git/hooks/` (or wires up a `core.hooksPath` entry so it composes with existing hook managers).
  - A documented, copy-pasteable hook script for teams that manage hooks with an existing tool (Husky, pre-commit, lefthook, etc.), which simply shells out to `tali sweep --staged`.
- **Bypass**: standard git escape hatches still apply (`git commit --no-verify`), so the gate is advisory-by-consent, not a lock.

---

## VS Code extension

A **thin TypeScript client over the CLI's `tali watch` mode**. The extension is a presentation layer: it launches the watcher the MVP already ships and renders its findings. It does **not** re-implement detection.

### Activation

- **Auto-starts on window open** via `activationEvents: ["onStartupFinished"]`. There is no command the user has to run.
- Because each VS Code window runs its own extension host, **opening N windows monitors N folders/repos independently** — one watcher per workspace, no shared global daemon to coordinate.

### On activate

1. **Resolve the `tali` binary**: first the bundled npm platform package (the platform-specific optional dependency shipped alongside `@talicode/core`), then fall back to `tali` on `PATH`.
2. **Start `tali watch`** for the active workspace folder.
3. **Map findings to VS Code diagnostics (squiggles)** — each finding becomes a `Diagnostic` on the relevant file/range, surfaced in the editor and the Problems panel.
4. **Quick-Fix / "Heal" action**: a `CodeAction` on each diagnostic ties into the healing flow (see [./ROADMAP-HEAL.md](./ROADMAP-HEAL.md)). The extension offers the action; the heal roadmap owns the mechanism.

### Config file language support

Register the custom `.tali` config extension as **YAML** so `config.tali` gets full YAML syntax highlighting and indentation behavior:

```jsonc
// package.json — contributes
"languages": [
  {
    "id": "yaml",
    "extensions": [".tali"],
    "aliases": ["TaliCode Configuration"]
  }
]
```

Plus a `contributes.icons` entry providing a **file icon for `.tali`**, so the config file is recognizable in the Explorer.

### Failure posture

**The only user requirement is installing the extension.** Everything else degrades gracefully:

- A **missing binary** (no bundled package and nothing on `PATH`) surfaces a **dismissible notice** with a hint on how to install `@talicode/core` — never a hard fail.
- A **missing API key** likewise surfaces a dismissible notice, not a blocking error.
- If the watcher can't start, the extension stays quietly inactive rather than throwing on window open.

---

## Licensing

These are local, **MIT-licensed** developer tools — free to install, inspect, and fork. This is distinct from the commercial server products, which have their own roadmaps: [./ROADMAP-TALICLOUD.md](./ROADMAP-TALICLOUD.md) and [./ROADMAP-TALIAGENTICSERVER.md](./ROADMAP-TALIAGENTICSERVER.md).

---

## Out of scope / open questions

**Out of scope (for this deferred design)**

- Re-implementing any detection in TypeScript or in the hook — both integrations delegate to the shipped `tali` CLI.
- Editors other than VS Code (JetBrains, Neovim, etc.) — future work, not designed here.
- CI / server-side gating — that lives with the commercial server roadmaps, not this local tooling.
- Auto-installing the git hook without explicit opt-in.

**Open questions**

- Hook install UX: prefer a `tali hook install` subcommand, `core.hooksPath` wiring, or documented snippets only — and how to compose cleanly with Husky/lefthook/pre-commit already present in a repo.
- Multi-root workspaces: one `tali watch` per workspace folder vs. one per window, and how diagnostics are attributed when folders overlap.
- Watcher lifecycle: restart/backoff policy when `tali watch` exits unexpectedly, and how (or whether) to surface repeated crashes without being noisy.
- Binary/version skew: how the extension should react when the resolved `tali` version differs from the extension's expected protocol.
- Notice fatigue: how often to re-surface the dismissible missing-binary / missing-key notices before staying silent.
