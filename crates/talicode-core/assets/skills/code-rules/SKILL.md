---
name: code-rules
description: NASA Power-of-10 safety-critical rules (language-agnostic)
---
Apply safety-critical coding rules (NASA Power of 10, language-agnostic): bound all loops, check every return value that signals success/failure, keep data at the smallest scope, avoid non-linear control flow (goto/longjmp), and assert invariants. Flag violations of these safety rules.
