---
name: code-aviation
description: >-
  OPT-IN, STRICT, safety-critical review profile modeled on the DO-178C
  standard for airborne software. NOT part of the default review run — its
  rules are deliberately tighter than general code should bear, and it is
  enabled only for code that must never fail. Enforces absolute predictability
  over developer convenience: no dynamic allocation after initialization, no
  recursion of any kind, compile-time-constant loop bounds, locked
  priority-based scheduling, strong typing with no implicit conversions, no
  dead or unreachable code, and a single exit per function. A STRICT SUPERSET
  of code-bounded-recursion, code-bounded-loops, and
  code-deterministic-concurrency. Use when reviewing safety-critical,
  DO-178C, aviation, avionics, flight-control, or life-critical software, or
  when the operator explicitly opts a change into the aviation profile.
---

# code-aviation — the DO-178C safety-critical lens

This lens reviews code the way certification authorities review airborne
software under DO-178C. The governing philosophy is **absolute
predictability over developer convenience**. On the ground a crash is an
inconvenience; in the air it is a fatality. So every construct that trades a
sliver of runtime uncertainty for a measure of programmer ease is rejected
outright.

Three properties are non-negotiable, and every rule below serves at least one
of them:

- **Determinism** — given the same inputs and the same state, the software
  must always execute the same path in the same bounded time. No allocation
  that may or may not succeed, no scheduling that may or may not preempt, no
  recursion whose depth depends on data.
- **Auditability** — a certifier must be able to read the code and reason
  about its worst-case behavior by inspection, without running it. Anything
  that hides control flow or resource use from a reader is a defect.
- **Traceability** — every line must map to a requirement and be exercisable
  by a test. Dead code, unreachable branches, and hidden exits break that
  chain and are treated as errors, not cleanups.

This SKILL.md conveys the DO-178C **spirit**, not merely a pattern list. Flag
findings that violate the intent — a construct that quietly defeats
determinism, auditability, or traceability — even when it does not match one
of the literal shapes named below.

## The rules

- **No dynamic memory allocation after initialization.** All memory is claimed
  during a bounded startup phase and never released or re-claimed at runtime.
  `malloc`, `free`, `new`, `delete`, and any equivalent runtime heap operation
  after init is an error (`dynamic-allocation`, severity error): heap
  exhaustion and fragmentation are non-deterministic failure modes.
- **No recursion, direct or indirect.** This is *stricter* than merely
  requiring a base case — DO-178C bans recursion entirely, because call-stack
  depth must be statically provable. Any function that calls itself, or
  participates in a cycle of calls, is an error (`recursion-banned`, severity
  error). Rewrite as explicit bounded iteration.
- **Compile-time-constant loop bounds.** Every loop must terminate at a bound
  fixed at compile time. This is *stricter* than a merely statically
  verifiable bound — a runtime-computed limit, even a provable one, is not
  enough. The upper bound must be a constant a certifier can read off the
  source.
- **Strong typing with no implicit conversions.** No implicit widening,
  narrowing, sign change, or truncation. Every conversion is explicit and
  bounds-checked. An implicit type conversion that can truncate or corrupt
  data is an error (`implicit-conversion`, severity error): silent data
  corruption is a classic latent avionics fault.
- **Locked, priority-based scheduling.** Concurrency uses a fixed, statically
  declared, priority-based schedule. This is *stricter* than general
  deterministic concurrency — dynamic task creation, unbounded queues, and
  best-effort ordering are rejected. Timing must be analyzable up front.
- **No dead or unreachable code.** Every statement must be reachable and
  traceable to a requirement. Unreachable branches, unused functions, and code
  disabled by always-false conditions break traceability and are defects.
- **A single, clear exit per function.** One return path per function.
  Scattered early exits fracture the control-flow graph a certifier must audit
  and complicate worst-case timing analysis.

## Relationship to the default lenses

code-aviation is a **strict superset** of several default lenses. Where a
concept overlaps, this lens applies the tighter DO-178C form:

- It subsumes **code-bounded-recursion** — that lens merely requires a base
  case; this one bans recursion outright.
- It subsumes **code-bounded-loops** — that lens accepts any statically
  verifiable bound; this one demands a compile-time constant.
- It subsumes **code-deterministic-concurrency** — that lens wants
  determinism; this one requires a locked, priority-based schedule with no
  dynamic tasking.

On top of those it adds rules with no default counterpart: no runtime heap
allocation, strong typing with no implicit conversions, no dead code, and a
single exit per function.

Because it overlaps the defaults, this lens is **not** part of the default
code-review run — its rules are too strict for general code, where they would
produce noise rather than signal. It is **opt-in**: deliberately enabled for
safety-critical work, never applied on its own. When it does run alongside a
default lens, any overlapping finding is **de-duplicated by file and line at
aggregation**, so the same location is reported once, in its strictest form.
