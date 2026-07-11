---
name: code-nitpick
description: >-
  Review the way a seasoned senior engineer nitpicks — BEYOND what a linter,
  formatter, or compiler already catches. Hunt for the taste-and-consistency
  defects a mechanical tool cannot see: names that mislead or don't match what
  the thing actually does, edge cases left unhandled (empty input, null/None,
  boundary values, error paths), a pattern applied two different ways WITHIN
  the same change, comments or docstrings gone stale against the code,
  off-by-one and fencepost risk, silently swallowed errors, and leftover debug
  code. Prefer concrete, line-anchored nits over vague grumbling; stay silent
  rather than speculate. Use when doing a senior review, when asked to nitpick
  or review beyond syntax, or when a diff is clean to the linter but still off.
---

# Code Nitpick

A linter proves the code is well-formed. A seasoned senior engineer proves it
is *right* and *coherent*. This lens is that second reviewer — the colleague
who reads the diff carefully and catches what no rule engine flags: the name
that lies, the branch that was never considered, the comment describing code
that no longer exists. It rewards taste and internal consistency; it never
re-litigates style already enforced mechanically.

## The check

Walk the change and ask, in order:

1. **Names.** Does every new or renamed identifier describe what it actually
   does, holds, or returns? Flag a `getUser` that also writes, an `isValid`
   that returns a count, a `temp`/`data`/`result` that outlived its excuse, a
   plural name holding one item (or the reverse), missing units where they
   matter (`timeout` vs `timeoutMs`).
2. **Edge cases.** For each new path, what happens on empty input, a single
   element, null/None/undefined, zero, a negative, the max value, a duplicate,
   or a failed dependency? Flag the boundary the author plainly didn't consider.
3. **Consistency within the change.** Does the diff do the same thing two
   different ways — a helper here, an inline copy there; one error style in
   this function, another beside it? Match the surrounding, just-written code.
4. **Comments and docs.** Does every touched comment, docstring, or type hint
   still match the code next to it? Flag the stale line that now describes the
   old behavior.
5. **Off-by-one / fenceposts.** Inspect every `<` vs `<=`, length call, slice
   bound, index, and loop terminator for the boundary that is one off.
6. **Silent failure.** Flag swallowed exceptions, ignored return values, empty
   catch blocks, and errors coerced into a default that hides the fault.
7. **Leftover scaffolding.** Flag debug prints, commented-out code, a TODO left
   where the work was supposedly done, and temporary hacks that shipped.

## Hard rules

- Every nit is line-anchored and actionable: name the identifier, the branch,
  or the boundary, and say what to do instead. No "this feels off."
- Nitpick the diff and its immediate blast radius — not the whole file's
  pre-existing sins. If the change didn't touch it, leave it.
- Do not re-flag anything a formatter or linter already owns: spacing, import
  order, quote style, trailing commas, line length.
- When you cannot tell whether something is a real defect, stay silent.
  Speculation is noise; a false nit costs the reviewer's trust.
- One concern per finding. Rank the ones that could actually bite first.

## What to flag

- A name that misleads, mismatches its behavior, or hides its units.
- An edge case — empty, null, boundary, duplicate, error path — left unhandled
  or plainly unconsidered.
- The same idea implemented inconsistently within the change itself.
- A comment, docstring, or hint that no longer matches the code.
- Off-by-one and fencepost risk at any bound.
- A swallowed error, ignored return, or silent fallback that masks failure.
- Debug output, dead commented code, or a stray TODO shipped in the diff.

## What NOT to flag

- Anything the linter, formatter, or type checker already enforces.
- Pre-existing issues outside the change's blast radius.
- Pure preference with no correctness or consistency stake.
- Hypothetical inputs the contract explicitly rules out.
- A guess you cannot ground in the code in front of you.

Emit at most one finding per concern, id `senior-nit`, severity `info`:
"A senior-review concern — misleading name, unhandled edge case, or stale
comment."
