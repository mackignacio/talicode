---
name: code-deterministic-concurrency
description: same output regardless of scheduling
---
Concurrent code must produce the same output for the same input regardless of thread/task scheduling: no data races, no deadlocks, no time-dependent bugs. Using sleep/setTimeout/delay as a synchronization mechanism is forbidden — it introduces race conditions; real synchronization primitives are required.
