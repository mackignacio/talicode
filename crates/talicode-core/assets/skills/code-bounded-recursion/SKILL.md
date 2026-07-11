---
name: code-bounded-recursion
description: >-
  Recursion is ALLOWED, but every recursive function must have a clear, finite,
  reachable terminating base case that provably stops it — the recursive analogue
  of a for-loop with a definite length. The base case must EXIST and every recursive
  path must measurably PROGRESS toward it: some argument (size, depth, index, remaining
  work) must strictly shrink on each call so the recursion is guaranteed to bottom out.
  Flag recursion with no reachable base case, or where no argument moves toward the base
  case on each call — that is unbounded recursion and a stack-overflow waiting to happen.
  This is the lenient general-code profile (bounded recursion is fine); it is NOT the
  strict aviation profile that bans recursion outright. Use when reviewing recursion, a
  function that calls itself, mutual recursion, tree/graph traversal, divide-and-conquer,
  or when you see terms like "recursion", "base case", "recursive call", "stack overflow",
  "infinite recursion", "termination", or "recursion depth".
---

# Bounded recursion — recursion is fine, but it must provably terminate

Recursion is a legitimate tool, not a smell. A function may call itself as long as it behaves like a loop with a definite bound: it has a **base case** that stops the descent, and **every** recursive path drives an argument toward that base case. The mental model is a `for` loop of known length — you can point at the value that changes each step and name the point at which it stops. If you cannot, the recursion is unbounded and one deep input away from a stack overflow.

Two conditions must BOTH hold. A base case that exists but is never reached is worthless; progress toward a base case that doesn't exist is progress toward nothing. Termination requires the pair.

## The check

1. **Locate the base case.** Find the non-recursive return(s) — the branch(es) that stop the descent. If there is none, that is an `unbounded-recursion` error, full stop.
2. **Prove the base case is reachable.** Confirm the base condition can actually become true for real inputs, not just in theory. A base case guarded by a condition the recursion never satisfies is effectively absent.
3. **Identify the variant.** Name the quantity that measures distance to the base case — collection size, remaining depth, string length, a shrinking index, a numeric counter. There must be one concrete value you can point at.
4. **Prove strict progress.** On every recursive call, that variant must strictly move toward the base case (usually strictly decrease). If any recursive path leaves it unchanged or moves it the wrong way, termination is not guaranteed.
5. **Check every path, including mutual recursion.** All recursive branches must progress, not just the common one. For mutual recursion (a→b→a), trace the variant across the whole cycle, not one hop.
6. **Sanity-check the depth.** Even bounded recursion can overflow the stack if the bound scales with input size (deep recursion on a large linked structure). Note when depth is proportional to unbounded input and an iterative form or explicit stack would be safer.

## Hard rules

- Every recursive function MUST have at least one reachable, non-recursive terminating base case.
- Every recursive call MUST make measurable progress toward a base case — a strictly decreasing (or otherwise convergent) variant, argued explicitly.
- The base case MUST be checked before recursing on the shrinking value, so the smallest case returns instead of stepping past its terminator.
- Recursion driven by external/unbounded state (user input depth, arbitrary data nesting) MUST have a hard depth guard or be rewritten iteratively.
- A recursive path that can leave the variant unchanged (e.g. recursing on the same argument under some branch) is treated as unbounded until proven otherwise.

## What to flag

- A self-calling function with no base case, or whose only base case is unreachable for valid inputs (`unbounded-recursion`, severity error).
- A recursive call where no argument provably shrinks — the same list, the same index, the same node passed straight through.
- Off-by-one terminators that step past the base case (recursing on `n-1` while the base checks `n == 0` but `n` can start negative).
- Mutual recursion where the cycle as a whole makes no net progress toward any base case.
- Deep recursion whose depth grows with unbounded input, risking stack overflow even though it technically terminates — recommend an explicit-stack or iterative rewrite.

## What NOT to flag

- Recursion with a clear base case AND a strictly shrinking argument on every path — that is bounded recursion, and it passes. Do not push an iterative rewrite for its own sake.
- Structural recursion over a finite, statically-bounded structure (a fixed-arity tree, a known-depth AST) where depth cannot exceed a small constant.
- Tail-recursive or divide-and-conquer forms whose variant obviously halves or decrements each step.
- Iteration that merely looks recursive (a helper called in a bounded loop) — that is the bounded-loops lens's concern, not this one.
- Do NOT apply the strict no-recursion-at-all rule here; that belongs to the opt-in aviation profile. Under this lens, provably-terminating recursion is allowed.
