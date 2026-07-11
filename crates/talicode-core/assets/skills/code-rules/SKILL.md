---
name: code-rules
description: >-
  Apply this repo's NASA Power-of-10 safety-critical coding rules (Python edition) when
  writing or reviewing code. Use whenever you add or change non-trivial code, design a
  function/loop/worker, or review a diff for control-flow, bounds, scope, dereferencing,
  or input/return-value safety. Covers the 10 rules with this codebase's concrete
  shapes (bounded loops, batch caps, guard clauses, no code-as-string, Law of Demeter),
  the compliance posture, and how to document a justified deviation. Rule 10 (zero
  static-analysis warnings) chains the /lint-test gate.
---

# Code rules — NASA Power of 10

Reference: <https://gist.githubusercontent.com/SZanlongo/c5e3a9157e1496b77b17/raw/22c8f0a3d0174c8294465bea0b3c5e22b299ae49/NASA10commandments.md>

The 10 rules NASA's JPL applies to safety-critical C (flight software). We aren't writing flight software — but pharma clients make multi-million-dollar competitive-intel and trial-strategy decisions on this platform's output. A missed signal, a silently-truncated trial list, a worker that hangs forever have client-level consequences. **Follow these rules as closely as Python allows.** Deviating is a decision that needs justification in the PR, not a default you drift into.

## The 10 rules — Python edition

Each restates the NASA original in Python terms; the parenthetical names the C-language original.

1. **Simple, linear control flow. No recursion.** Prefer iteration with an explicit work-list (`list` / `collections.deque`) over recursive helpers. No exception-as-flow-control (don't `raise`/`except` through normal paths). The only acceptable recursion is provably bounded by data shape (e.g. a tree whose depth is capped by indication scope), with the bound enforced by a guard clause at entry and a comment naming it. **Always use early returns / guard clauses** — handle the negative / edge / invalid / failure state first and `return`/`raise` early; the happy path stays un-nested at the lowest indentation at the end, never buried in an `else` after the success branch. **Max 2 levels of `if`/`else` nesting** — deeper than two, refactor (extract a function, early returns, dict dispatch, or polymorphism). pylint's `max-nested-blocks` (currently `8`) is a loose backstop, not the target. *(Rule 1: no `goto`, `setjmp`/`longjmp`, recursion.)*

2. **Every loop has an explicit upper bound.** `for x in items:` is fine when `items` is bounded at the source (`limit=N` in the Mongo query, slice the list). `while` loops need a counter (`for _ in range(MAX_RETRIES):`) or an externally-bounded queue. **No bare `while True:`** without an explicit max-iterations counter and a `break`. Copy existing bounded loops (workflow agent-budget loop, `compounding_engine.compound_temporal()`'s 14-day window, AI rate-limiter retries). *(Rule 2: fixed upper bound on every loop.)*

3. **Bound the resources a single call holds.** Python's heap is dynamic, but avoid unpredictable runtime growth: cap batch sizes in long-running workers (collectors use `BATCH_SIZE=200`, materializers stream), never grow an in-memory dict/list for a job's lifetime (flush to Mongo periodically), set Mongo cursor `batch_size`/`limit`, and cap API payloads (pagination, not unbounded `find()`). *(Rule 3: no dynamic allocation after init.)*

4. **Functions ≤20 lines of logic.** Past 20 lines (counting logic lines — not blanks, comments, or the signature), _Extract Method_ into named helpers. This is the *Long Method* smell (see `/code-smells`); pylint's `max-statements` (currently `220`) is only a far-looser backstop, not the target — 20 is the target. Split worker/router/collector entry-points before adding new logic, never after. *(Rule 4: function fits on one sheet of paper.)*

5. **Validate inputs at entry; check return values at every call site.** Two checks per non-trivial function is the floor — a precondition (param valid?) and a postcondition/invariant (return shape matches what callers expect?). At API boundaries Pydantic does this; inside workers write guard clauses by hand (`if doc is None: raise ValueError(...)`). **Don't rely on `assert`** — Python strips it under `python -O`; use `raise` for contract enforcement and reserve `assert` for tests. *(Rule 5: ≥2 assertions per function.)*

6. **Declare data at the narrowest scope.** No module-level mutable state for per-request data. Per-tenant/per-request context lives in function locals, parameters, or `ContextVar` — never on a worker instance attribute (`self._current_client_id = ...` leaks across calls). Module-level globals are for constants and singletons only (loggers, the Mongo client). *(Rule 6: smallest scope.)*

7. **Check every return value; validate every parameter.** `find_one()` returns `None` on miss — handle it before dereferencing. `count_documents()` returns `0` — handle the empty case (no divide-by-zero). An `await` returning `T | None` needs a `None`-branch. **Don't silently swallow exceptions** — `except Exception: pass` is banned outside a narrow, commented, justified cleanup path. *(Rule 7: check returns + validate params.)*

8. **No code-as-string.** No `exec`, `eval`, `getattr(self, user_string)`, or f-string-then-`compile`. SQL goes through SQLAlchemy parameterized binds, not f-strings; Mongo pipelines are data structures, not assembled-from-input strings; AI tool calls use **structured Pydantic schemas** for input and output, never free-text code we then execute. *(Rule 8: limit the preprocessor.)*

9. **At most one level of attribute/key dereference per expression.** `obj.a` is fine; `obj.a.b.c.d` is the *Message Chain* smell. For a value three layers deep, ask the holder via a method (Law of Demeter). Same for dicts — replace `payload["data"]["nested"]["thing"]` with a Pydantic model or an extraction helper. *(Rule 9: ≤1 level of pointer dereferencing.)*

10. **Zero static-analysis warnings — pylint must rate `10.00/10`.** Run the gate via the **`/lint-test` skill** (`ruff format --check` + `ruff check` + `flake8` clean, `pylint` `10.00/10`). Anything below `10.00/10` is a finding to fix, not a score to negotiate down. CI gates `--fail-under=10.0` both post-merge ([python-test.yml](.github/workflows/python-test.yml), pylint over `lambdas/ tests/` — tests included) and pre-merge ([python-test-incremental.yml](.github/workflows/python-test-incremental.yml), changed files). **Never add `# noqa`, `# pylint: disable`, or any suppression — fix the underlying issue** (remove unused imports, extract to helpers, rename to convention, fix import grouping). Type-annotate every new function. *(Rule 10: pedantic warnings, zero output.)*

## Compliance posture

- **Project standards, not aspirations.** New code follows all 10 rules; PR review calls out violations like failing tests.
- **Existing violations are tech debt.** Don't bundle the cleanup into an unrelated PR (see `/bug-fix`); flag it — file a ticket, leave a `# TODO(nasa-power-of-10):` comment referencing it, and clean it up next time you touch that area (boy-scout rule).
- **A genuine deviation must be documented at the source.** A comment on the function/block explaining why the rule was relaxed and the alternative safeguard. "Recursion here because the tree is bounded at depth N by the indication scope; assertion at entry enforces the bound." is valid. "Felt cleaner." is not.
- **Hardest-enforced first** (tied to CI gates): Rule 10 (pylint `10.0`), Rule 4 (function length — `max-statements` + Long Method), Rule 7 (param validation — Pydantic at boundaries). Rules 1, 2, 5, 6, 9 rely on PR review — be vigilant.

The cost of a slow signal feed is annoyance. The cost of a *wrong* signal feed is a client misreading the competitive landscape and writing strategy on top of it. Code accordingly.

## Related skills

- `/lint-test` — Rule 10's gate (ruff + flake8 + pylint 10.00/10).
- `/code-no-keys` — the credential-placement specialist; Rule 8 (no code-as-string) and the Security non-negotiable overlap it. Externalize every key/token/password to env or a Secrets Manager ARN.
- `/code-smells` — Rule 4 (Long Method) and Rule 9 (Message Chains) overlap the smell catalog.
- `/bug-fix`, `/atomic-commit` — the flow these rules are checked within.
