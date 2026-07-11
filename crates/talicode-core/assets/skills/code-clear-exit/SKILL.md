---
name: code-clear-exit
description: clear, non-branching exits (composes with early-return)
---
Every function exit must be clear and non-branching. Multiple return statements are fine when each exit is obvious: guard-clause early returns at the top are encouraged, exactly as code-early-return wants. Flag returns buried inside deep or nested branches (mid-loop, inside the third arm of a nested if) that make control-flow and path analysis hard.
