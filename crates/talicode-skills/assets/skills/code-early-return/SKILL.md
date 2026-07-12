---
name: code-early-return
description: >-
  Apply the early-return (guard-clause) principle — exit a function the moment an invalid state,
  error, or base case is met, so the happy path stays flat and unindented. MANDATORY whenever a
  change writes or modifies a function with conditional logic (a no-op otherwise): check failure
  conditions first and `return`/`raise`/`continue` early, instead of wrapping the success path in
  deep `if`/`else` nesting. Balance with readability — keep functions
  short so the multiple exits stay obvious, and don't strand resource cleanup. Use whenever you
  write or change a function with conditional logic, or the operator says "early return", "guard
  clause", "return early", "flatten this", "too nested", or "reduce nesting".
---

# Early return — guard clauses over deep nesting

Sources: the operator's early-return brief · [The Return Early Pattern (softwarecraft-mastery)](https://medium.com/softwarecraft-mastery/the-return-early-pattern-c554f47fe58a) · [Clean Coding Tip: Early Return (dev.to/shameel)](https://dev.to/shameel/-clean-coding-tip-early-return-principle-47kj) · [Improve your code with early return (felipeelia.com)](https://felipeelia.com/improve-your-code-with-early-return/) · [Return Early Pattern (swlh)](https://medium.com/swlh/return-early-pattern-3d18a41bba8).

> A function exits **immediately** as soon as an invalid state, error, or base condition is met. Handle these **guard clauses** at the top, and the main "happy path" logic stays unindented, sequential, and highly readable — no scrolling through layers of nested brackets to find what the function actually does.

This skill is **mandatory** for any change that **writes or modifies a function with conditional logic**. Run the **early-return check while shaping control flow** and the **review pass before committing**. It is a judgment lens like `/code-yagni`, `/code-kiss`, `/code-solid`, `/code-dry`, and `/code-composition`; it sits on top of the `/lint-test` mechanical floor and is run by `/code-review`. (For a change with no branching control flow, it's a no-op — note that and move on.)

## Order — runs right after KISS

**`/code-early-return` runs after `/code-yagni` and `/code-kiss`** (before SOLID/DRY/composition): first prove the code should exist (YAGNI) and write it simply (KISS), *then* flatten its control flow with guard clauses. Early return is the concrete control-flow technique behind KISS's "flat over nested" rule — it specializes that rule, so it runs immediately after KISS and before the OO-design lenses shape the surviving types.

## Invert the logic — check failures first

Instead of wrapping the successful path inside a deep `if`/`else` pyramid, **invert it**: check each failure condition up front and exit, so the core logic runs sequentially at the bottom, one indent deep.

**🚫 Deep nesting — the happy path is buried:**
```python
def process_order(order):
    if order is not None:
        if order.is_paid:
            if order.items:
                ship_order(order)        # the real work, 3 indents deep
                return "Success"
            else:
                return "Error: No items"
        else:
            return "Error: Not paid"
    else:
        return "Error: Invalid order"
```

**✅ Early returns — guard clauses on top, happy path flat:**
```python
def process_order(order):
    if order is None:
        return "Error: Invalid order"
    if not order.is_paid:
        return "Error: Not paid"
    if not order.items:
        return "Error: No items"

    ship_order(order)                    # the real work, flat
    return "Success"
```

The guard form reads top-to-bottom as "reject these, then do the thing." In Python the early exit is `return`, `raise` (validation that should error, not return a sentinel), or `continue`/`break` inside a loop to skip an iteration.

## Why it pays off

- **Readability** — removes the layers of nested brackets; the happy path is at one indent, easy to find and follow.
- **Fail-fast** — invalid inputs are discarded immediately; no wasted work computing down a doomed path.
- **Easier debugging** — clean separation between the validation block (top) and the main algorithm (bottom).

## Balance — keep the exits obvious (the guardrails)

Early return isn't license to scatter exits everywhere. Heed the well-known caveats:

- **Keep functions short.** Multiple exit points are only a readability win when the function is small enough that every exit is visible at a glance. A 200-line function with eight scattered `return`s is *worse* than nesting — if you're tempted to sprinkle returns through a long function, the real fix is to split it (`/code-kiss` one-thing-per-unit, `/code-solid` SRP). The classic "single entry, single exit" (SESE) objection dissolves once functions are short.
- **Don't strand cleanup.** When a path acquires a resource (file handle, DB connection, lock), a bare early `return` can skip the release. Use the language's scope-bound cleanup so every exit is safe — in Python a `with` block / context manager (or `try/finally`), **not** repeated manual close-before-each-return. Guard clauses placed *before* the resource is acquired are always safe; the hazard is only returns *after* acquisition.
- **Guards are for the abnormal, the tail is for the normal.** Reserve the top guards for invalid/edge/error states; keep the genuine business logic as the flat happy path at the bottom. Don't invert so far that the normal case becomes a guard.
- **Don't drop behavior to flatten.** Flattening must preserve every branch's effect (the right error/sentinel/exception per condition). Simpler control flow, identical semantics — this is the `/code-kiss` "simple is not broken" rule applied to control flow.

## Review pass — run this over the diff before committing

`git diff HEAD` (and `--staged`); for each new/changed function with conditional logic, check whether the happy path is buried. Tags:

- **guard** — a failure/edge check wrapping the body in `if cond: <whole body>` → invert to `if not cond: return/raise` up front and unindent the body.
- **nest** — `if`/`else` nested ≥2 deep where inner branches just early-exit → flatten into sequential guard clauses.
- **else** — an `else` that only exists because the `if` didn't return → drop the `else`, return in the guard, de-indent the rest.
- **arrow** — "arrow-shaped" code (indentation marching right then back) → collapse with guards.
- **split** — too many exits to keep obvious → the function is too long; split it (defer to `/code-kiss`/`/code-solid`), don't just add more returns.
- **cleanup** — an early return that could skip resource release → wrap acquisition in `with`/`try-finally` so every exit is safe.
- **keep** — note nesting you deliberately did NOT flatten (e.g. it would strand cleanup, or a single shallow `if/else` is already clearer) so a reviewer doesn't push a worse inversion.

**Finding format:** `L<line>: <tag> <description>. <guard-clause fix>.` — multi-file: `<file>:L<line>: …`. Conclude with **`Early return. Ship.`** if there's no violation (including diffs with no branching control flow).

Scope: this pass judges *control-flow shape only* — not necessity (`/code-yagni`), broader simplicity/naming (`/code-kiss`), OO design (`/code-solid`), duplication (`/code-dry`), has-a-vs-is-a (`/code-composition`), correctness (`/bug-fix`), safety (`/code-rules`), or the broader smell catalog (`/code-smells`).

## Intensity

- **lite** — write it as-is; note the guard-clause improvement in one line, don't force the inversion.
- **full (default)** — invert obvious nested-validation pyramids into guard clauses; flatten arrow code; keep functions short enough that the exits stay visible; flag clear cases.
- **ultra** — guard-clause hard-liner; no `else`-after-return, no body buried under a condition, happy path always flat at one indent — but still bound by short-function and cleanup-safety guardrails (split rather than scatter; never strand a resource release).

Default to **full**.

## Relationship to other skills

- Invoked by `/coding` as a **mandatory** step for any function-with-branching change, and run by `/code-review` as a judgment lens alongside `/code-yagni`, `/code-kiss`, `/code-no-nested-loop`, `/code-solid`, `/code-dry`, `/code-composition`, `/code-smells`, `/code-rules`, and `/clean-code`.
- **Runs after `/code-yagni` and `/code-kiss`** (before SOLID/DRY/composition): it specializes KISS's "flat over nested" into a concrete control-flow technique, so it follows KISS directly and precedes the OO-design lenses.
- **Pairs with `/code-kiss`**: short functions are what make multiple exits safe; when guards multiply, the fix is to split (KISS/SRP), not to keep adding returns.
- Overlaps `/code-smells` (deeply nested conditionals, arrow code) and `/clean-code` (which applies guard clauses at the style level) — `/code-early-return` is the dedicated control-flow lens; those are the catalog and the style floor.
- After any guard-clause refactor, re-run the `/lint-test` gate, then commit via `/atomic-commit`.
