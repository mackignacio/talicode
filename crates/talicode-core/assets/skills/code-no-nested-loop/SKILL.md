---
name: code-no-nested-loop
description: >-
  Apply the no-nested-loop principle — avoid a loop inside a loop where it costs quadratic time
  or buries logic under deep indentation. MANDATORY whenever a change writes or modifies loops (a
  no-op otherwise): replace an O(n²) inner scan with a dict/set lookup (O(n)), extract a required
  inner loop into a named function, or reach for a built-in iterator/comprehension. Not dogmatic —
  nesting is fine for genuinely multi-dimensional data (grids, matrices, image pixels) and
  strictly-bounded inputs (12 months × 31 days). Use whenever you add/modify a loop, or the
  operator says "nested loop", "O(n^2)", "this is quadratic", "loop in a loop", or "this is slow".
---

# No nested loops — kill quadratic scans and arrow-shaped iteration

Sources: the operator's no-nested-loop brief · [Avoiding nested for-loops (Stack Overflow)](https://stackoverflow.com/questions/43173168/avoiding-nested-for-loops-python) · [Why are nested loops bad practice (Software Engineering SE)](https://softwareengineering.stackexchange.com/questions/199196/why-are-nested-loops-considered-bad-practice) · [The evil nested for-loop (Koronci, Medium)](https://juliuskoronci.medium.com/the-evil-nested-for-loop-9fbc2f999ec1) · [How to avoid nested for-loops (Salesforce SE)](https://salesforce.stackexchange.com/questions/376786/how-to-avoid-nested-for-loop-condition-for-code-optimization-reusability).

> A guideline, not an absolute law: avoid putting a loop inside a loop where it isn't needed. Two reasons. **Performance** — nesting *multiplies* operations: an outer loop of `n` around an inner loop of `n` runs `n × n = n²` times. At 10 items that's 100 ops (negligible); at 100,000 items it's 10,000,000,000 — apps freeze or crash. **Readability** — deep indentation (the "arrow" anti-pattern) raises cognitive load: the reader must track `i`, `j`, `k` at once to follow the logic.

This skill is **mandatory** whenever a change **writes or modifies a loop** (a no-op for diffs with no loops). Run the **loop check while writing** and the **review pass before committing**. It is a judgment lens like `/code-yagni`, `/code-kiss`, `/code-early-return`, `/code-solid`, `/code-dry`, and `/code-composition`; it sits on top of the `/lint-test` mechanical floor and is run by `/code-review`.

## Order — runs after early-return

**`/code-no-nested-loop` runs after `/code-yagni`, `/code-kiss`, and `/code-early-return`** (before SOLID/DRY/composition): first prove the code should exist (YAGNI), write it simply (KISS), and flatten its control flow with guard clauses (early-return), *then* attack quadratic / arrow-shaped iteration. It pairs naturally with early-return — both fight deep nesting; early-return flattens *conditionals*, this flattens *loops*. Settle iteration shape before the OO-design lenses shape the surviving types.

## Strategies — how to remove a nested loop

### 1. Map / hash lookup — turns O(n²) into O(n)
The most common win. Instead of re-scanning a second list inside the first, build a dict/set once and look up in O(1).

**🚫 Nested scan — O(n²):**
```python
for user in users:
    for profile in profiles:          # rescans every profile per user
        if user["id"] == profile["user_id"]:
            ...
```
**✅ Pre-indexed lookup — O(n):**
```python
profile_map = {p["user_id"]: p for p in profiles}   # build once
for user in users:
    profile = profile_map.get(user["id"])
    if profile is not None:
        ...
```
This is the dominant pattern in this codebase: signal/entity reconciliation, dismissal left-joins, and overlay application should index one side into a dict/set keyed by `entity_id` / `(entity_id, source_id)` / `normalized_name` rather than nest-scanning two collections.

### 2. Extract the inner loop into a named function
When the inner loop is genuinely required, abstracting it doesn't change complexity but removes the indentation and names the intent.

**🚫** nested accumulation inline → **✅**
```python
def total_score(student):
    return sum(student.grades)

for student in classroom:
    print(total_score(student))
```

### 3. Built-in iterators / comprehensions
Reach for the language's flattening tools: `sum()`/`any()`/`all()`/`max()` over a generator, a comprehension, `itertools.product`/`chain`/`groupby`, `collections.defaultdict`/`Counter`. They express cross-referencing without manual index-tracking.

### 4. Sort first, then single-pass
For search/compare problems, sorting once (`O(n log n)`) then walking with two pointers or a single pass beats an `O(n²)` pairwise scan.

## When nesting is actually fine (don't be dogmatic)

Nested loops are correct and necessary — do **not** contort the code to remove them — when:
- **The data is genuinely multi-dimensional** — a fixed 2-D grid, image pixels, matrix math.
- **The input is strictly bounded** — iterating a fixed calendar (12 months × 31 days), a small fixed config, an enum × enum table: the upper bound is tiny, so `n²` is a constant. Forcing a dict here would be a `/code-yagni` / `/code-kiss` violation (more machinery, no real gain).
- The "inner loop" is actually an O(1)-bounded operation, or the collections are provably small and won't grow with load.

The test: *does the inner iteration grow with input size, over data that can get large at runtime?* If yes, de-nest. If it's bounded or multi-dimensional, keep it and (if it reads poorly) just extract a named function.

## Review pass — run this over the diff before committing

`git diff HEAD` (and `--staged`); for each new/changed loop, check for an avoidable nested loop. Tags:

- **quadratic** — an inner loop re-scanning a second collection that grows with input → index that collection into a dict/set and look up (O(n²) → O(n)).
- **lookup** — a linear `in list` / `for … if x == …` membership test inside a loop → use a `set`/`dict` membership instead.
- **extract** — a structurally-required inner loop hurting readability → pull it into a named function.
- **builtin** — a manual nested accumulation/cross-reference → `sum`/`any`/comprehension/`itertools`/`Counter`.
- **arrow** — ≥3 levels of loop/branch indentation → flatten (combine with `/code-early-return` guard clauses).
- **keep** — note nesting you deliberately kept (multi-dimensional data, strictly-bounded input) so a reviewer doesn't push a pointless de-nesting.

**Finding format:** `L<line>: <tag> <description>. <de-nesting fix + resulting complexity>.` — multi-file: `<file>:L<line>: …`. Conclude with **`No nested loop. Ship.`** if there's no violation (including diffs with no loops, or only justified/bounded nesting).

Scope: this pass judges *loop shape and complexity only* — not necessity (`/code-yagni`), general simplicity/naming (`/code-kiss`), conditional flattening (`/code-early-return`), OO design (`/code-solid`), duplication (`/code-dry`), has-a-vs-is-a (`/code-composition`), correctness (`/bug-fix`), safety (`/code-rules`), or the broader smell catalog (`/code-smells`).

## Intensity

- **lite** — write it as-is; note the de-nesting / complexity improvement in one line, don't force the refactor.
- **full (default)** — replace growth-bearing O(n²) scans with lookups; extract or flatten arrow-shaped loops; leave genuinely multi-dimensional / bounded nesting alone (note it as `keep`).
- **ultra** — no avoidable nested loop survives; every cross-reference is a hash lookup, every required inner loop is a named function or built-in — but still bound by KISS/YAGNI (don't add a dict for a 12×31 bounded loop).

Default to **full**.

## Relationship to other skills

- Invoked by `/coding` as a **mandatory** step for any loop-touching change, and run by `/code-review` as a judgment lens alongside `/code-yagni`, `/code-kiss`, `/code-early-return`, `/code-solid`, `/code-dry`, `/code-composition`, `/code-smells`, `/code-rules`, and `/clean-code`.
- **Runs after `/code-early-return`** (before SOLID/DRY/composition): both fight deep nesting — early-return flattens conditionals, this flattens loops — so they sit adjacent; the OO-design lenses follow.
- **Pairs with `/code-early-return`** for the arrow anti-pattern (a deeply-indented block is usually loops *and* conditionals); with `/code-kiss`/`/code-yagni` for the don't-over-engineer-bounded-loops guardrail; and overlaps `/code-smells` (*Loops* / complex iteration) and `/clean-code`.
- After any de-nesting refactor, re-run the `/lint-test` gate (a complexity change can shift behavior — keep the tests green), then commit via `/atomic-commit`.
