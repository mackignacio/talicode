---
name: code-no-nested-loop
description: avoid quadratic loop-in-loop
---
Avoid a loop inside a loop where it costs quadratic time or buries logic under deep indentation. Flag an O(n^2) inner scan that a set/map lookup would make O(n). Genuinely multi-dimensional data (grids, matrices) and strictly-bounded inputs are fine.
