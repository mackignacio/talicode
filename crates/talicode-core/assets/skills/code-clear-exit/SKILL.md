---
name: code-clear-exit
description: >-
  Enforces clear, non-branching function exits so any reader can enumerate
  every exit point at a glance. Multiple returns are fine — what matters is
  that each one is visible and shallow, not buried inside deep or nested
  control flow. Composes with code-early-return: guard-clause returns at the
  TOP of a function are encouraged, and a single guard clause satisfies both
  lenses at once. Flags returns hidden mid-loop or inside the third arm of a
  nested if, where they obscure control-flow and path analysis. Never treat
  many returns as the problem; the problem is a return you have to hunt for.
  Use when the operator says "clear exit", "single exit", "too many returns",
  "buried return", "hidden return", "exit points", or "hard to follow the
  control flow".
---

# code-clear-exit

A function should let any reader account for how it can finish without tracing
every branch. The number of terminating statements — `return`, `raise`/`throw`,
or any construct that exits the function — is not the concern. Their
**visibility** is. An exit that sits flush at the top, or at a shallow and
obvious level of the body, is trivial to reason about. An exit buried three
branches deep, or hidden partway through a loop, forces the reader to mentally
simulate the whole function just to know it is even there. This lens keeps
exits in plain sight.

This lens **composes with `code-early-return`; it never contradicts it.**
Guard clauses that reject invalid state at the top of a function and return
immediately are exactly what `code-early-return` asks for, and they are
exactly what this lens rewards: shallow, visible, non-branching. A single guard
clause satisfies both lenses. When the two lenses look at the same early
return, they agree — one flattens nesting, the other keeps the exit obvious,
and the guard clause serves both.

## The check

1. Enumerate the function's exit points. If you cannot list them without
   simulating the body, the exits are not clear enough.
2. For each exit, note its nesting depth. Top-level and shallow exits pass;
   exits reachable only through several stacked conditions or loop iterations
   are suspect.
3. Confirm guard clauses sit at the TOP and return immediately — reward them,
   do not flag them. They serve this lens and code-early-return equally.
4. Look for terminating statements embedded mid-loop or inside the innermost
   arm of a nested conditional; these are the exits a reader will miss.
5. Prefer restructuring so exits rise toward the surface: hoist a condition
   into a top guard, or extract a nested block into its own function whose own
   exits are then shallow again.
6. Judge composition, not headcount. Several shallow exits beat one exit that
   is only reachable through a maze.

## Hard rules

- Multiple exits are allowed. Never flag a function merely for having more than
  one return.
- Guard-clause early returns at the top of a function are ENCOURAGED, never
  flagged. They reinforce code-early-return.
- An exit buried inside a nested branch or partway through a loop, where it
  obscures control flow, is a `buried-return` (severity: warning).
- The reader must be able to enumerate all exit points at a glance; if they
  cannot, the structure fails this lens.
- Never force a single-exit rewrite that reintroduces the deep nesting
  code-early-return removed. Clarity of exits, not one lone return statement, is
  the aim.
- Language-agnostic: applies to any terminating construct in any language —
  return, raise/throw, an early break that leaves the function, or equivalent.

## What to flag / What NOT to flag

Flag a `buried-return` when a terminating statement is reachable only through
several stacked conditions, or from inside a loop body, so its presence and its
trigger condition are not obvious from a scan of the function. Flag exits that
force the reader to reconstruct the full control-flow graph to know how the
function can end. Flag a deeply nested return that could instead become a top
guard clause or be extracted into a smaller function with shallow exits.

Do NOT flag a function for having several returns when each is shallow and
plainly visible. Do NOT flag a guard clause at the top — that is encouraged and
satisfies code-early-return too. Do NOT demand a single exit point, and do NOT
push a rewrite that trades shallow, readable returns for one return wrapped in
deep nesting. Do NOT treat an early return or break that flattens nesting as a
violation; if it makes the exits clearer, it is precisely what this lens wants.
