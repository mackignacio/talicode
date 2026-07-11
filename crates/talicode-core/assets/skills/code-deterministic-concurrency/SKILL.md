---
name: code-deterministic-concurrency
description: >-
  Enforce deterministic concurrency: concurrent code must produce the SAME
  output for the same input regardless of thread or task scheduling. No data
  races, no deadlocks, no time-dependent behavior. Forbid sleep/setTimeout/delay
  used AS SYNCHRONIZATION to "wait for" another task; demand real primitives —
  locks, channels, atomics, condition variables, joins, awaits. Flag shared
  mutable state touched concurrently without synchronization, ordering
  assumptions that lean on the scheduler, and lock acquisition orders that can
  deadlock. Use when reviewing threads, async/await, goroutines, coroutines,
  futures, workers, or shared state; on any hint of concurrency, race condition,
  deadlock, thread safety, or flaky timing-dependent tests.
---

# Deterministic Concurrency

Concurrent code is correct only when its result does not depend on the order in
which the scheduler happens to run things. The same input must yield the same
output across every interleaving — under load, on one core or many, with a
debugger attached or not. Code that "works on my machine" because tasks usually
finish in a convenient order is not working; it is holding a losing lottery
ticket that will be cashed in production. Determinism comes from explicit
synchronization, never from hope about timing.

## The check

1. Identify every piece of state shared across concurrent contexts (threads,
   tasks, coroutines, workers) and confirm each concurrent access is guarded by
   a real primitive — a lock, atomic, channel, or an ownership transfer that
   makes sharing impossible.
2. Find every `sleep`, `setTimeout`, `delay`, `wait(ms)`, or busy-poll and ask
   what it is waiting for. If the answer is "for another task to finish, to be
   ready, or to have written its result," it is a race dressed as a pause.
3. Trace ordering assumptions: does correctness rely on task A observably
   completing before task B, without a join, await, channel receive, or other
   happens-before edge forcing that order?
4. Map lock acquisition. If two paths can take locks L1 and L2 in opposite
   orders, a deadlock exists regardless of how rarely it triggers.
5. Check completion and shutdown: are spawned tasks joined or awaited, and is
   their state fully published before it is read?

## Hard rules

- Never use `sleep`/`setTimeout`/`delay` to coordinate correctness between
  concurrent tasks. Replace it with the primitive that actually signals
  readiness: join/await the task, receive on a channel, wait on a condition
  variable, or await a future. `sleep-as-sync` (severity: error) — sleep/
  setTimeout/delay used to coordinate concurrency is a race condition; use real
  synchronization.
- Never read or write shared mutable state from more than one context without
  synchronization. `data-race` (severity: error) — shared mutable state
  accessed concurrently without synchronization.
- Establish a single global lock ordering and acquire locks in that order
  everywhere; a cycle in acquisition order is a latent deadlock.
- Do not depend on scheduler behavior — task start order, thread priority, or
  time slice — for any observable outcome.
- Publish a task's results through the same primitive that signals its
  completion, so no reader can observe a half-written value.

## What to flag / What NOT to flag

Flag: a `sleep(100)` followed by reading a value another thread was supposed to
set; a shared counter, map, list, or flag mutated from multiple tasks with no
lock or atomic; a test that passes only because one task "usually" finishes
first; two functions grabbing the same two locks in opposite orders; detached
tasks whose output is read without a join or await; double-checked state without
proper memory ordering; "wait until it's probably done" polling loops with no
real readiness signal.

Do NOT flag: a `sleep`/backoff used for legitimate rate-limiting, retry backoff,
polling an external system on an interval, debouncing, or animation pacing —
timing that is the goal, not a stand-in for coordination. Do not flag immutable
shared data, state confined to a single task, message passing that transfers
ownership, or state already protected by a correctly scoped lock, atomic, or
channel. The target is timing used to fake synchronization, not timing used as
timing.
