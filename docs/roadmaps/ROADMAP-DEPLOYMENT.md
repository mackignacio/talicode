# Roadmap — Local Deployment (pre-commit git hook)

> Deferred design, part of the TaliCode MVP (see [../plans/MVP.md](../plans/MVP.md)). This is a local developer integration that is **designed but not yet built** — it wraps the CLI the MVP already ships (`tali sweep --staged`) rather than re-implementing any detection.

TaliCode is an "AI Slop Gatekeeper": a compiled Rust CLI (`tali`), distributed via npm as the unscoped `talicode` package, that detects AI-generated code problems before they reach a commit. This document covers the pre-commit git hook that sits on top of that CLI. The other local, MIT-licensed integration — the **VS Code extension** — now has its own dedicated design in [./ROADMAP-VSCODE.md](./ROADMAP-VSCODE.md).

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

The VS Code extension — the other local, MIT-licensed integration that wraps the CLI (a thin
presentation layer over `tali watch`, rendering findings as diagnostics with a Quick-Fix "Heal"
action) — now has its own full, phased design in **[./ROADMAP-VSCODE.md](./ROADMAP-VSCODE.md)**.

---

## Licensing

These are local, **MIT-licensed** developer tools — free to install, inspect, and fork. This is distinct from the commercial server products, which have their own roadmaps: [./ROADMAP-TALICLOUD.md](./ROADMAP-TALICLOUD.md) and [./ROADMAP-TALIAGENTICSERVER.md](./ROADMAP-TALIAGENTICSERVER.md).

---

## Out of scope / open questions

**Out of scope (for this deferred design)**

- Re-implementing any detection in the hook — it delegates to the shipped `tali` CLI.
- CI / server-side gating — that lives with the commercial server roadmaps, not this local tooling.
- Auto-installing the git hook without explicit opt-in.

**Open questions**

- Hook install UX: prefer a `tali hook install` subcommand, `core.hooksPath` wiring, or documented snippets only — and how to compose cleanly with Husky/lefthook/pre-commit already present in a repo.

*(VS Code extension open questions have moved to [./ROADMAP-VSCODE.md](./ROADMAP-VSCODE.md).)*
