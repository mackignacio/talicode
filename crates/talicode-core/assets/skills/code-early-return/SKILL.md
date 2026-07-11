---
name: code-early-return
description: use guard clauses to flatten nesting
---
Exit a function the moment an invalid state, missing precondition, or trivial case is known. Flag deeply nested conditionals where a guard clause / early return would flatten the happy path.
