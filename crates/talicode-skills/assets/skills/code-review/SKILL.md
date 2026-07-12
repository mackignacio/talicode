---
name: code-review
description: >-
  Review code across every judgment lens this harness cares about, in one pass, by running
  the dedicated lenses in order: code-think-twice (understand before writing), code-kiss
  (simplicity / readability), code-yagni (minimal-change / no over-engineering), code-dry
  (duplication / single source of truth), code-early-return (guard clauses / flatten nesting),
  code-clear-exit (clear, non-branching exits), code-no-nested-loop (de-nest O(n²) / arrow-shaped
  loops), code-bounded-loops (statically bounded iteration), code-bounded-recursion (recursion
  terminates on a finite base case), code-deterministic-concurrency (same output regardless of
  scheduling), code-composition (composition over inheritance), code-solid (SOLID OO design),
  code-smells (design smells), code-nitpick (senior-engineer review beyond syntax), code-rules
  (NASA Power of 10 safety), code-magic-strings (no hardcoded meaningful literals), code-magic-numbers
  (no unexplained numeric literals), code-no-keys (no hardcoded secrets), code-no-credentials
  (no embedded logins / plaintext passwords), and code-traceability (best-effort requirement/test
  traceability). Every lens ALWAYS runs — none is skipped and none short-circuits the pass — so the
  review gathers the complete set of findings in one go, then renders ONE verdict: APPROVED only when
  every lens is clean; otherwise NOT APPROVED with a consolidated list of every lapse grouped by lens.
  The opt-in code-aviation strict profile is NOT part of the default run — enable it deliberately for
  safety-critical code. Use when writing or reviewing code — a diff, a PR, or your own change before a
  commit. This skill is a thin orchestrator; each lens's rules live in its own skill.
runs:
  - code-think-twice
  - code-kiss
  - code-yagni
  - code-dry
  - code-early-return
  - code-clear-exit
  - code-no-nested-loop
  - code-bounded-loops
  - code-bounded-recursion
  - code-deterministic-concurrency
  - code-composition
  - code-solid
  - code-smells
  - code-nitpick
  - code-rules
  - code-magic-strings
  - code-magic-numbers
  - code-no-keys
  - code-no-credentials
  - code-traceability
---

# Code review

A single review pass that runs every judgment lens in this harness over the change. This skill
**does not redefine** any rules — it sequences the twenty default lenses and aggregates their
findings into one verdict. Open each lens for its actual checks and catalog.

Run it when writing or reviewing a change — a diff, a PR, or your own work before committing.
**Look only at what the change touches:** flag issues the change *introduces*; for pre-existing
issues, note them rather than expanding the diff (boy-scout rule only for small, obviously-correct
cleanups).

## The pass — every lens runs, in order

1. **code-think-twice** — is the requirement and the surrounding system understood before writing?
2. **code-kiss** — is this the simplest form that works and reads clearly?
3. **code-yagni** — does every piece of this need to exist, or is it speculative?
4. **code-dry** — is there a single source of truth, or duplicated logic/literals?
5. **code-early-return** — are guard clauses used to flatten nesting?
6. **code-clear-exit** — is every exit clear and non-branching (no returns buried in nested branches)?
7. **code-no-nested-loop** — any O(n²) loop-in-loop / arrow-shaped code that should de-nest?
8. **code-bounded-loops** — does every loop have a statically verifiable upper bound?
9. **code-bounded-recursion** — does every recursion have a reachable, finite base case?
10. **code-deterministic-concurrency** — same output regardless of scheduling; no sleep-as-sync, no data races?
11. **code-composition** — composition preferred over inheritance where it fits?
12. **code-solid** — do the SOLID principles hold?
13. **code-smells** — any design smells introduced?
14. **code-nitpick** — the senior-engineer pass: misleading names, unhandled edges, stale comments.
15. **code-rules** — the NASA Power-of-10 safety rules (language-agnostic).
16. **code-magic-strings** — any meaningful string literal that should be a named constant/enum?
17. **code-magic-numbers** — any unexplained numeric literal that should be named?
18. **code-no-keys** — any hardcoded secret (key, token, connection string, private key)?
19. **code-no-credentials** — any embedded login or plaintext password?
20. **code-traceability** — public surface with no covering test or discernible purpose?

## The verdict

Gather **all** findings across every lens, de-duplicate overlaps by file + line, then render one
verdict:

- **APPROVED** — every lens is clean.
- **NOT APPROVED** — a consolidated list of every finding, grouped by lens, each anchored to
  `file:line`. Prefer silence over speculation: report only concrete, line-anchored violations —
  false positives kill trust in the gate.

The opt-in **code-aviation** DO-178C strict profile is not in this default run; a repo enables it
explicitly for safety-critical code, on top of these lenses.
