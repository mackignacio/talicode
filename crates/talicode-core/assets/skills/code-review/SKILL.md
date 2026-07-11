---
name: code-review
description: run every default lens over the change and render one verdict
runs:
  - code-think-twice
  - code-kiss
  - code-yagni
  - code-dry
  - code-early-return
  - code-clear-exit
  - code-no-nested-loop
  - code-bounded-loops
  - code-bounded-recursion
  - code-deterministic-concurrency
  - code-composition
  - code-solid
  - code-smells
  - code-nitpick
  - code-rules
  - code-magic-strings
  - code-magic-numbers
  - code-no-keys
  - code-no-credentials
  - code-traceability
---
Run every listed lens over the change, aggregate all findings, and render a
single verdict: APPROVED only when every lens is clean; otherwise NOT APPROVED
with the consolidated findings. The opt-in code-aviation strict profile is not
part of the default run — enable it explicitly for safety-critical code.
