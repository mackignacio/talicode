# Roadmap — Healing (the Surgeon agent)

> Deferred design. This is a designed-but-not-built feature of the TaliCode MVP, which ships **detect-only**. See the [MVP plan](../plans/MVP.md) for the shipping scope.

## Overview

TaliCode's MVP stops at detection: the **Auditor** scans staged code and emits findings about likely AI-slop problems. Healing extends that pipeline with a second stage — a **Surgeon agent** that takes the Auditor's findings and proposes concrete fixes, then optionally applies them.

The Surgeon runs in one of two modes:

- **Heal Preview** (default) — propose fixes, render diffs, write nothing until the developer approves.
- **Auto-Heal** (opt-in) — write approved fixes in place, for trusted or CI flows.

The two stages stay cleanly separated: the Auditor decides *what is wrong*, the Surgeon decides *how to fix it*. This keeps detection trustworthy on its own and makes healing a strictly additive layer.

## Heal Preview mode

Heal Preview is the **default** healing mode and the one we recommend for everyday local use.

For each finding, the Surgeon proposes a fix and renders it as a **unified diff** against the current file. The developer reviews each diff and explicitly approves or rejects it. **Nothing is written to disk until approval is given** — a rejected proposal leaves the working tree untouched.

Approval is per-finding, so a developer can accept the three fixes they trust and skip the one they want to handle by hand. Preview output is designed to be skimmable: one diff hunk per finding, annotated with the rule and severity that motivated it.

## Auto-Heal mode

Auto-Heal writes approved fixes in place without an interactive prompt. It is **opt-in only** — gated behind an explicit flag (e.g. `tali sweep --heal --auto`) — and is intended for:

- Trusted, well-scoped repositories where the ruleset is tuned and false positives are rare.
- CI or pre-commit automation where no human is at the keyboard to approve diffs.

Even in Auto-Heal, the safety invariant holds: fixes are applied to the working tree for the developer/CI to inspect and commit, never silently folded into a commit behind the developer's back.

## The Auditor → Surgeon handoff

The handoff between the two agents is a **structured contract**, not a freeform prompt. For each finding, the Auditor passes:

| Field | Meaning |
| --- | --- |
| `file` | Path of the file the finding is in |
| `line` | 1-indexed anchor line |
| `rule` | The rule/check that fired |
| `severity` | error / warning / info |
| `message` | Human-readable description of the problem |
| _file content_ | The current source of `file` (or the relevant region) |

The Surgeon returns a **proposed patch** for that finding — a unified diff (or an edited region) — tied back to the originating finding by id. This binding matters: every proposed change is traceable to exactly one finding, which is what makes per-finding approval and post-hoc auditing possible.

One finding maps to at most one proposed patch. If the Surgeon cannot produce a fix it is confident in, it returns *no patch* for that finding rather than guessing.

## CLI wiring

Healing surfaces through two entry points:

- **`tali sweep --heal`** — run the normal sweep, then hand findings to the Surgeon. Add `--auto` to opt into Auto-Heal; without it, Heal Preview is used.
- **`tali heal`** — the existing command surface graduates from its current "not yet enabled" stub to invoking the Surgeon directly on an already-produced set of findings.

### Retry behavior

A proposed fix is not trusted on faith — after a patch is applied (in Preview, to a scratch copy; in Auto-Heal, in place), TaliCode **re-audits** to confirm the fix actually resolves the finding and does not introduce new ones.

- If the re-audit still reports the finding, or a fix breaks the re-audit by raising a new finding, the Surgeon retries with the updated context.
- Retries are bounded by **`max_retries`** (configured in `config.tali`).
- When retries are exhausted without a clean re-audit, the finding is reported as **unhealed** and left to the developer. TaliCode never ships a fix it could not verify.

## Trust & safety rationale

The core principle: **never silently overwrite a commit.**

The mandatory diff-preview + explicit-approval flow exists because:

- **Developer trust** — AI-proposed edits are suggestions, not authority. The developer stays the decision-maker; the tool earns trust by showing its work before touching code.
- **Auditability** — every applied change is a reviewed diff tied to a specific finding, so it is always answerable *why did this line change?*
- **Low blast radius** — per-finding approval means a bad proposal is rejected in isolation; it cannot cascade into a wide, unreviewed rewrite.

Auto-Heal is deliberately **opt-in**, never the default. Removing the human from the loop is a choice a team makes explicitly for a context where they have accepted the tradeoff (tuned rules, CI safety nets), not something TaliCode does on its own.

## Out of scope / open questions

- **Multi-file / cross-file fixes** — the MVP handoff is per-file; refactors that span files are not yet modeled.
- **Fix conflict resolution** — how to reconcile two patches that touch overlapping lines of the same file.
- **Learning from rejections** — whether rejected proposals should feed back to tune the Surgeon or the ruleset.
- **Partial-hunk approval** — approval is currently per-finding; accepting part of a single proposed diff is undecided.
- **Provenance in the commit** — whether/how to annotate commits that include Surgeon-applied fixes.
- **`max_retries` defaults** — the right default retry ceiling, and whether it should vary by severity.
