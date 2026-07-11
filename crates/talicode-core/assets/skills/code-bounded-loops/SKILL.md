---
name: code-bounded-loops
description: >-
  Enforce that every iterative process has a statically verifiable, fixed
  upper bound before it runs. Loops over a known-length collection, or with a
  constant or parameter-derived maximum, pass; a while(true), condition-only,
  or open-ended loop whose termination cannot be reasoned about statically is
  flagged as a potential infinite loop or runaway. Lenient general-code
  profile, not the strict aviation one: a clearly-terminating condition loop
  is fine — the target is loops whose bound cannot be modeled. Guards against
  hangs, unbounded memory growth, and unmodelable worst-case time. Use when a
  diff adds or changes iteration and the reviewer weighs "bounded loop",
  "infinite loop", "loop bound", "while true", "unbounded iteration", or
  "runaway loop".
---

# Bounded Loops

Every loop should have a ceiling you can name before the first iteration. If a
reader — or the compiler — can point to the value that caps the iteration count
(a collection's length, a constant, a validated argument), the loop is bounded
and its worst case is modelable. If the only thing that stops the loop is a
runtime condition that might never flip, the loop is a latent hang and an
unbounded resource sink. This lens keeps iteration reasoned-about, not
hoped-about. It is the lenient general-code profile: a clearly-terminating
condition loop is acceptable; the target is loops whose termination cannot be
argued statically.

## The check

1. Identify each loop in the change and ask the single deciding question: what
   is the maximum number of iterations, and where does that maximum come from?
2. If the bound is the length of a known finite collection (iterating an array,
   list, map, set, range, or fixed sequence), it PASSES — that is the common,
   healthy case and needs no ceremony.
3. If the bound is a constant, or a value derived from a parameter or config
   that is validated finite before the loop begins, it PASSES.
4. If termination depends only on a condition (`while (cond)`, `for (;;)` with a
   `break`, recursion used as a loop), decide whether that condition provably
   moves toward false on a bounded counter or a strictly shrinking finite
   quantity. A clearly-terminating condition loop PASSES.
5. If you cannot name any static ceiling — `while (true)`, poll-until-ready with
   no attempt cap, drain-a-queue that another thread refills, retry with no max
   retries — FLAG it.

## Hard rules

- A loop whose iteration count has no statically verifiable upper bound is a
  finding: id `unbounded-loop`, severity `warning` — "A loop with no statically
  verifiable upper bound (potential infinite loop)."
- Iterating a finite in-memory collection is bounded by construction; never flag
  it, and do not demand a redundant counter on top of it.
- `while (true)` / `for (;;)` / an infinite `loop` must have a visible, bounded
  exit — a counter cap, a timeout, or a strictly shrinking finite quantity — to
  escape the flag. A bare `break` is not enough on its own.
- Retry, poll, reconnect, and backoff loops need an explicit maximum attempts or
  a deadline. "Eventually it will succeed" is not a bound.
- A `break` inside the body only bounds the loop if the condition that triggers
  it is itself driven toward truth by bounded progress; an incidental break
  behind an unrelated branch does not bound anything.
- Judge worst case against adversarial input: if a caller or peer can feed state
  that spins the loop forever, it is unbounded regardless of the happy path.

## What to flag / What NOT to flag

FLAG:

- `while (true)` with no counter, timeout, or provably-reached break.
- Polling `while (!ready())` with no attempt cap or elapsed-time cutoff.
- Retry / backoff loops with no maximum retry count.
- Loops whose termination hinges on external state a caller or peer controls,
  with nothing capping how long you wait.
- Consuming a queue or buffer a concurrent producer can keep refilling, with no
  drain limit.

Do NOT flag:

- `for item in collection` or index loops over a known-length structure.
- Loops bounded by a constant or a validated-finite parameter.
- Condition loops with an incrementing counter compared against a fixed maximum.
- Condition loops over a strictly decreasing finite quantity (a value halved
  each pass, a remaining-count that only shrinks).
- Intended long-lived service or event loops whose per-iteration work is bounded
  and whose input cannot be forced to loop forever by an adversary.

When unsure, resolve it with one demand: name the ceiling. If you can point to
it in the code, keep the loop. If you cannot, flag it.
