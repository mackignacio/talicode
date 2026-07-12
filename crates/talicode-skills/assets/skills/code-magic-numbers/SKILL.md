---
name: code-magic-numbers
description: >-
  Judgment lens for unexplained numeric literals in a diff. Flags a raw number
  that carries domain meaning — a threshold, a factor, a duration, a retry limit,
  a buffer size, a rate, a port — sitting inline where it should be a NAMED
  CONSTANT. Enforces single-source-of-truth for meaningful numbers: obscured
  intent, scattered edits, typo risk, and drift all trace back to the same
  literal copied around. Complements code-magic-strings (the string sibling)
  and overlaps code-dry when the same literal repeats. Low-noise: identity and
  degenerate values, self-defining arithmetic, and loop scaffolding are NOT
  flags. Use when reviewing code with a magic number, a hardcoded number, an
  unnamed threshold, or a bare numeric literal that should be a named constant.
---

# code-magic-numbers — no unexplained numeric literals

A number in code should explain itself. When a literal carries meaning — `86400`
is a day in seconds, `0.05` is a five-percent rate, `3` is the max retry count,
`4096` is a buffer size — that meaning belongs in a name, declared once. The bare
literal hides intent from the next reader, forces every future change to hunt
down each occurrence, invites a mistyped digit that no reviewer will catch, and
leaves the codebase with no single source of truth for a value the domain
actually cares about. A repeated meaningful literal is also duplication (see
code-dry); the fix is the same — hoist it to one named constant.

This lens is about *meaning*, not about numbers. Most numbers in real code are
structural or degenerate and mean exactly themselves. Flag the number that
encodes a decision; leave the number that is just arithmetic.

## The check

1. Scan the diff for numeric literals — integers, floats, hex, durations, byte
   sizes — that appear inline in expressions, conditions, arguments, or config.
2. For each, ask: does this number encode a domain decision (a limit, rate,
   threshold, timeout, capacity, tuning knob, magic offset) that a maintainer
   might one day need to find, understand, or change?
3. If yes, and it is written as a bare literal rather than referenced through a
   named constant, flag it. The name is the documentation.
4. If the same meaningful literal appears more than once, flag it and note the
   duplication — it now has more than one place to drift out of sync.
5. Prefer flagging the literal at its point of use; the fix is a named constant
   (or a config value) declared once with a name that states the intent.

## Hard rules

- A literal that sets a policy — timeout, retry count, page size, rate limit,
  percentage, threshold, capacity, TTL, port — must be a named constant.
- A meaningful literal that repeats is always a finding, even if any single
  occurrence looks harmless. Duplication of a number is drift waiting to happen.
- The finding is `magic-number`, severity info: a meaningful numeric literal
  (threshold, factor, duration) that should be a named constant.
- Name the constant for what it *means*, not for its value. `MAX_RETRIES`, not
  `THREE`. If you cannot name the meaning, reconsider whether it is meaningful.
- Do not invent thresholds. Only flag numbers actually present in the diff; never
  speculate about values that "might" appear elsewhere.
- Findings de-duplicate by file and line with the other lenses at aggregation —
  report the literal once, at its line.

## Not magic — do NOT flag

Keeping this lens quiet matters as much as catching real cases. The following are
structural or self-explaining and must NOT be reported:

- Identity and degenerate values: `0`, `1`, `-1`. Empty checks, increments,
  decrements, "not found" sentinels, first/last index, sign flips. These mean
  themselves.
- Arithmetic that defines its own context: `n % 2` for even/odd, dividing by `2`
  to halve or find a midpoint, `index + 1` / `index - 1` for neighbors, `* 100`
  to render a ratio as a percentage. The surrounding expression is the
  explanation.
- Loop and iteration scaffolding: `for i = 0; i < length; i++`, starting at `0`,
  stepping by `1`, slicing `[1:]`. The `0` and `1` are boilerplate, not policy.
- Small structural counts inherent to the operation: taking `2` arguments,
  swapping a pair, a `3`-element coordinate — where the number is the shape of
  the data, not a tunable value.
- Values already named or read from configuration, constants, enums, or the
  environment. The intent is already documented; there is nothing to hoist.
- Trivial, universally understood units where a name adds no clarity — dividing
  milliseconds by `1000`, a base `10` radix, `2`-based bit math. Judgment
  applies: if a maintainer recognizes it instantly, leave it.

When unsure whether a number is policy or plumbing, ask whether someone would
ever need to *find and change* it deliberately. If yes, it wants a name. If it is
just how the arithmetic works, let it be.
