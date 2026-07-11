---
name: code-bounded-loops
description: loops must have a fixed, verifiable upper bound
---
Every iterative process must have a statically verifiable, fixed upper bound before execution. This prevents infinite loops, limits runaway memory, and makes execution time modelable. Loops over a known-length collection or a constant bound pass. Flag while(true) / condition-only loops with no provable termination bound.
