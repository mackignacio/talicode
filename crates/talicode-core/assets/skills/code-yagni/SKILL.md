---
name: code-yagni
description: >-
  Apply YAGNI ("You Aren't Gonna Need It") — the lazy-senior-dev discipline — BEFORE writing
  any code and as a review lens over every diff. MANDATORY for all coding work: invoke it for
  every fix, change, feature, or implementation so the answer is the smallest one that works.
  Always look for the minimal / 1-liner solution first by walking the decision ladder (does it
  need to exist → stdlib → native/platform → existing dependency → one line → only then minimal
  new code). Prefer deletion over addition; reject unrequested abstractions, speculative
  generality, and new dependencies. Never compromise validation, error handling, security, or
  explicitly requested features. Use whenever you add/modify code or the operator says "keep it
  minimal", "don't over-engineer", "yagni", "simplest fix", or "one-liner".
---

# YAGNI — the smallest change that works

Adapted from [ponytail](https://github.com/DietrichGebert/ponytail) ("lazy senior dev mode") and the [GeeksforGeeks YAGNI principle](https://www.geeksforgeeks.org/software-engineering/what-is-yagni-principle-you-arent-gonna-need-it/) for this repo.

> **The best code is the code you never wrote.** Every line is a liability someone else maintains. Before writing code, prove it has to exist. While reviewing a diff, prove it can't get shorter. *Lazy* here means **efficient, never careless** — you cut volume, not correctness.

## Why it pays off — the four costs of "we might need it"

Every speculative feature carries four costs YAGNI avoids — name them to justify *not* building something:

- **Build cost** — the time/effort/review spent creating a thing nobody asked for yet.
- **Delay cost** — what shipped late (or not at all) because that effort went to the speculative feature instead.
- **Carry cost** — the ongoing drag the extra code puts on *every other* change: more to read, test, and reason around.
- **Repair cost** — the bugs and technical debt the unused complexity introduces and that someone later has to fix.

A feature built "just in case" pays all four; a feature deferred until it's actually needed pays none. **Deferring a decision is cheaper than reversing one.**

This skill is **mandatory** for every coding task — bug fix, change, feature, user story, one-liner. Run the **ladder before writing** and the **review pass before committing**. It is a judgment lens like `/code-smells` and `/clean-code`; it sits on top of the `/lint-test` mechanical floor and is run by `/code-review`.

## The decision ladder — walk it BEFORE writing code

Evaluate each rung in order. **Stop at the first one that solves the problem** — don't descend further than you must.

1. **Does this need to exist at all?** Is the feature/file/function/branch/abstraction actually required by the request? If not — don't build it. Deletion beats addition. (Pure YAGNI.)
2. **Does the standard library solve it?** `itertools`, `collections`, `functools`, `pathlib`, `datetime`, `json`, `re`, `dataclasses` — reach here before writing logic or pulling a dependency.
3. **Does a native / platform feature already do it?** A MongoDB aggregation operator instead of in-Python post-processing; a Pydantic validator instead of a hand-rolled check; FastAPI dependency injection; a DB index/unique constraint instead of a manual dedup loop.
4. **Is it already an installed dependency?** Use what `requirements`/`pyproject` already pull in (motor, pydantic, redis, anthropic, …) before adding anything new. **Adding a dependency is a last resort** — it needs an explicit reason.
5. **Can it be one line?** A comprehension, a `dict.get(k, default)`, a single `$set`, a guard clause, a one-line helper call. Default to the 1-liner; expand only when the 1-liner is genuinely unreadable or unsafe.
6. **Only then, write minimal necessary code** — the shortest correct implementation. No speculative parameters, no "we might need it later" hooks, no config knobs nobody asked for.

The goal of every task is the **minimal or 1-liner** fix/change/implementation. Climb down the ladder only as far as the problem forces you.

## Hard rules

- **Prefer deletion and simplicity over clever solutions.** If you can remove code to fix it, do that first.
- **No unrequested abstractions.** A base class / interface / factory / strategy with a single implementation or one caller is over-engineering — inline it. Don't design for users that don't exist yet (this is `/code-smells`' *Speculative Generality*).
- **No new dependency** unless rungs 1–4 genuinely can't cover it; say why in the commit/PR.
- **Minimize file count.** Don't split into new modules/files unless size or a real reuse boundary demands it.
- **Don't gold-plate.** Build exactly what was asked — not the asked thing plus three anticipated variants.
- **Match the surrounding code.** The minimal change reuses the existing helper/pattern instead of introducing a parallel one.

## Never skip — YAGNI cuts volume, not safety

These are **non-negotiable** and are never what "minimal" trims:

- **Input validation at trust boundaries** — request bodies, webhook payloads, external API responses, anything client-supplied.
- **Error handling that prevents data loss or corruption** — DB writes, file/S3 ops, money/decision-board state, partial-batch failures.
- **Security** — auth/permission checks (`client_id` isolation, the role/permission gates), secret handling, injection-safe queries.
- **Explicitly requested features** — if the operator/ticket asked for it, it is needed by definition; don't YAGNI it away.
- **Accessibility** — for any UI-facing output.
- **One runnable check behind non-trivial logic** — per repo policy that's a `/py-test` at 100% coverage; for a truly trivial 1-liner the existing tests suffice. Never ship non-trivial logic with no test.

Repo-specific invariants YAGNI must respect (these are *requirements*, not gold-plating): no hardcoded indication/drug/company/trial names, no client state written to the shared lake, tenant routing via `get_client_collection`, the canonical signal schema, the no-suppression lint policy.

**Don't mistake YAGNI for these — it is none of them:**
- **Not "ignore scalability or design quality."** YAGNI defers *speculative* work; it never excuses a knowingly unscalable or sloppy design for a *current* requirement.
- **Not "abandon all planning."** It rejects premature *implementation*, not foresight — plan the direction, build only the must-haves now, defer the can-waits.
- **Not "strip every abstraction."** Removing an abstraction that supports a *current* requirement just to cut lines is over-correcting — keep what today's code genuinely needs (over-zealous simplification is its own pitfall).

## The `# yagni:` comment — mark deliberate shortcuts

When you intentionally take the simple path and there is a known ceiling, mark it so the next reader sees the tradeoff and the upgrade path — instead of silently shipping a limitation:

```python
# yagni: global in-process cache; switch to Redis if this runs in >1 worker
# yagni: linear scan, fine at <1k signals; add an index if the feed grows
```

One line: *what the shortcut is* + *when/how to upgrade*. Don't add a `# yagni:` to excuse missing validation or error handling — those aren't shortcuts, they're bugs.

## Review pass — run this over the diff before committing

`git diff HEAD` (and `--staged`), then flag every line that could be shorter or shouldn't exist. Five tags:

- **delete** — unused code, dead branches, speculative features with no current caller. Remove it.
- **stdlib** — hand-rolled logic the standard library already provides. Replace it.
- **native** — code/dependency duplicating a platform-native feature (Mongo operator, Pydantic, FastAPI, DB constraint). Replace it.
- **yagni** — an abstraction with a single implementation or a one-caller layer; a parameter/option nobody passes. Inline / drop it.
- **shrink** — identical behavior expressible in fewer lines. Compress it.

**Finding format:** `L<line>: <tag> <description>. <replacement>.` — multi-file: `<file>:L<line>: …`. Conclude with `net: -<N> lines possible.`, or **`Lean already. Ship.`** if there's nothing to cut.

Scope: this pass is about *volume and necessity only* — it does **not** judge correctness, security, or perf (those are `/bug-fix`, `/code-rules`, `/code-review`). A minimal self-check/smoke test is never flagged.

## Intensity

- **lite** — build it as requested; suggest the simpler alternative in one line, don't force it.
- **full (default)** — enforce the ladder; prefer stdlib/native; minimal new code; brief justification.
- **ultra** — YAGNI extremist; challenge the requirement itself; ship the 1-liner first, expand only if proven necessary.

Default to **full**. Use **lite** when the operator explicitly wants the literal request built as-is; **ultra** when they say "absolute minimum" / "challenge this".

## Relationship to other skills

- Invoked by `/coding` as a **mandatory** step of every coding task, and run **first** by `/code-review` — ahead of `/code-kiss` (simplicity), `/code-smells` (design), `/code-rules` (safety), and `/clean-code` (style) — since the cheapest line to review is the one you delete before judging its design.
- **Pairs with `/code-kiss`**: YAGNI cuts *what exists* (volume — delete, don't build); KISS simplifies *how the survivors are written* (shape — clearest, flattest). Run YAGNI first, then KISS.
- Overlaps `/code-smells` *Speculative Generality*, *Dead Code*, *Lazy/Middle Man class*, *Duplicate Code* — YAGNI is the **before-you-write** discipline; code-smells is the **catalog of what to remove**. Use both.
- After any YAGNI cut, re-run the `/lint-test` gate, then commit via `/atomic-commit`.
- Complements `/bug-fix`'s "smallest reversible change" — YAGNI is why that's the rule.
