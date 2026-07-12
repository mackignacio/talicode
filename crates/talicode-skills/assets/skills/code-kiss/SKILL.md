---
name: code-kiss
description: >-
  Apply KISS ("Keep It Simple, Stupid") — the simplicity discipline — BEFORE writing any code
  and as a review lens over every diff. MANDATORY for all coding work: invoke it for every fix,
  change, feature, or implementation so the result is the simplest thing that reads clearly and
  solves exactly the problem at hand. Prefer the obvious, boring, readable solution over the
  clever one; the simplest control flow; the fewest moving parts. Reject needless cleverness,
  premature optimization, deep nesting, and speculative complexity. Never let "simple" become
  "broken" — keep validation, error handling, security, and necessary structure. Use whenever
  you add/modify code or the operator says "keep it simple", "make it readable", "kiss",
  "too clever", or "simplify this".
---

# KISS — keep it simple

Sources: [GeeksforGeeks — KISS principle](https://www.geeksforgeeks.org/software-engineering/kiss-principle-in-software-development/) · [dev.to — KISS in programming](https://dev.to/kwereutosu/the-k-i-s-s-principle-in-programming-1jfg) · [freeCodeCamp — how to use KISS](https://www.freecodecamp.org/news/keep-it-simple-stupid-how-to-use-the-kiss-principle-in-design/).

> **Most systems work best when kept simple rather than made complicated.** *"We neither want just less nor more — only as much as is required."* Simple ≠ easy and ≠ fewer features; it means the **clearest path** to exactly what's needed, with the fewest moving parts. As M.A. Jackson warned: *"Programmers often take refuge in an understandable, but disastrous, inclination towards complexity."* Write the boring, obvious code — the next reader (often you) will thank you.

This skill is **mandatory** for every coding task — bug fix, change, feature, user story. Run the **simplicity check before writing** and the **review pass before committing**. It is a judgment lens like `/code-yagni`, `/code-smells`, and `/clean-code`; it sits on top of the `/lint-test` mechanical floor and is run by `/code-review`.

## KISS vs YAGNI — the split

They're complementary and both mandatory. **`/code-yagni` decides *whether* code should exist** (the smallest *amount* — delete, don't build). **`/code-kiss` decides *how* the code that must exist is written** (the simplest *shape* — clearest, most readable). Run YAGNI first (cut volume), then KISS (simplify what remains).

## The simplicity check — BEFORE and WHILE writing

Ask, in order:

1. **What is the actual problem?** Name the one thing this code must do. Solve *that*, not a generalized version of it. (*"An `if` statement often beats a machine-learning model."*)
2. **What is the most obvious solution a competent reader would expect?** Default to it. Clever only earns its place when the obvious way is genuinely inadequate — and then a `# why:` comment explains the need.
3. **Can the control flow be flatter?** Prefer guard clauses / early returns over nested `if/else`; a straight-line sequence over a state machine; a comprehension or `dict` dispatch over a branching ladder — *when that reads more clearly*, not as a golf trick.
4. **Is each unit doing one thing?** One function = one job; one class = one responsibility. Split a function that needs "and" to describe it.
5. **Are the names self-explanatory?** Descriptive variable/function names that state intent kill the need for comments-explaining-what. A reader should follow the code without a decoder ring.
6. **Did I add complexity nobody asked for?** No premature optimization, no abstraction for a single caller, no config knob/flag/hook for a hypothetical future. Solve today's requirement.

The goal of every task is the **simplest correct, readable** implementation — not the shortest at the cost of clarity, and not the most flexible at the cost of comprehension.

## Hard rules

- **Boring beats clever.** If two solutions work, ship the one that's easier to read. Cleverness is a cost, not a feature.
- **Flat beats nested.** Reduce nesting depth with guard clauses and early returns; deeply nested logic is where bugs hide.
- **Obvious beats implicit.** No magic, no surprising side effects, no relying on subtle language quirks. Code should do what it looks like it does.
- **One responsibility per unit.** Functions and classes that do one thing are simple to read, test, and change.
- **No premature optimization.** Don't tune performance until a real measurement demands it (`/code-yagni` rung 6 + this). Simple first; optimize the proven hot path later, with a `# why:` note.
- **Remove what's unused** — dead code, parameters nobody passes, leftover branches. (Overlaps `/code-yagni` `delete` and `/code-smells` *Dead Code*.)
- **Match the surrounding code.** The simplest change reuses the existing idiom instead of introducing a second, parallel way to do the same thing (avoid the inconsistency that itself breeds complexity).

## Simple is not broken — keep what's necessary

KISS removes *needless* complexity, never *essential* structure. These stay:

- **Input validation at trust boundaries**, **error handling that prevents data loss/corruption**, **security/permission checks**, **accessibility** — same non-negotiables as `/code-yagni`. "Simpler" never means dropping these.
- **Necessary abstractions.** Don't strip a justified abstraction just to cut line count — confusing simplicity with inadequate structure is itself a pitfall. The test is *clarity*, not size: an extracted helper that makes the call site obvious is *simpler* even though it adds a function.
- **Explicitly requested features and real requirements** — including the repo invariants (no hardcoded indications/drugs, no client state in the shared lake, tenant routing via `get_client_collection`, the canonical signal schema, the no-suppression lint policy). These are the problem, not complexity to trim.

> Pitfall to avoid: *over-simplifying critical logic* (e.g. collapsing distinct error cases into one silent `except`) reads as simple but is a bug. Simple means clear, not careless.

## Review pass — run this over the diff before committing

`git diff HEAD` (and `--staged`), then flag every spot that's more complex than the problem requires. Tags:

- **clever** — a cute/implicit construction where the obvious one reads better. Rewrite plainly.
- **nest** — deep `if/else`/loop nesting that a guard clause or early return would flatten. Flatten it.
- **split** — a function/class doing more than one thing. Decompose it.
- **name** — an unclear name forcing the reader to decode intent. Rename it.
- **premature** — speculative flexibility / optimization / abstraction for one caller. Drop or inline it.
- **magic** — surprising side effect, hidden coupling, or a relied-upon language quirk. Make it explicit.

**Finding format:** `L<line>: <tag> <description>. <simpler form>.` — multi-file: `<file>:L<line>: …`. Conclude with **`Simple enough. Ship.`** if there's nothing to flatten.

Scope: this pass judges *simplicity and readability only* — not necessity/volume (`/code-yagni`), correctness (`/bug-fix`), safety (`/code-rules`), or design smells (`/code-smells`). A necessary self-check/smoke test is never flagged.

## Intensity

- **lite** — build it as written; note the simpler form in one line, don't force it.
- **full (default)** — actively prefer the obvious/flat/boring solution; brief justification when you must go non-obvious.
- **ultra** — simplicity hard-liner; challenge any cleverness, demand the flattest control flow, refuse implicit magic outright.

Default to **full**.

## Relationship to other skills

- Invoked by `/coding` as a **mandatory** step of every coding task, and run by `/code-review` as a judgment lens alongside `/code-yagni` (necessity), `/code-smells` (design), `/code-rules` (safety), and `/clean-code` (style).
- **Pairs with `/code-yagni`**: YAGNI cuts *what exists* (volume), KISS simplifies *how it's written* (shape). Run YAGNI, then KISS.
- Overlaps `/clean-code` (naming, one-thing functions, single responsibility) and `/code-smells` (*Long Method*, *Switch Statements*, *Speculative Generality*) — KISS is the **before-you-write simplicity mindset**; those are the catalogs of specific fixes.
- After any KISS simplification, re-run the `/lint-test` gate, then commit via `/atomic-commit`.
