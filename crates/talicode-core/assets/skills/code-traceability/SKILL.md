---
name: code-traceability
description: >-
  Best-effort requirement/test traceability lens, inspired by DO-178C's ideal
  that every line of code should map to a stated purpose and to a test that
  covers it. HONEST about scope: this is a per-file static sweep, NOT a
  process audit and NOT a compliance certification. Flags only what the code
  itself reveals: exported/public surface with no covering test, undocumented
  public APIs with no stated purpose, and code with no discernible reason to
  exist (dead code, unreachable branches, unused exports). Prefer silence over
  speculation. Ties into code-smells for dead-code signals. Use when reviewing
  traceability, untested public surface, undocumented APIs, dead code,
  unreachable branches, or unused exports.
---

# Code Traceability

The ideal, borrowed from safety-critical practice (DO-178C): every line of code
exists for a reason, and that reason is verified. Concretely, each unit of code
should trace UP to a requirement or documented purpose, and DOWN to a test that
exercises it. Code that traces to nothing is either a gap in verification or a
gap in intent — both are risk.

This lens cannot see requirements databases, test-management systems, or
coverage reports. It sees the diff. So it makes the honest, narrow claim it can
support from the code alone: does this public surface state why it exists, and
is there a test that touches it?

## The check

1. Identify the newly added or changed **public/exported surface** — exported
   functions, public methods, classes, HTTP/RPC endpoints, CLI commands, and
   public constants that carry behavior.
2. For each, look for a **covering test** in the diff or the reachable test
   files: a test that names it, imports it, or exercises the endpoint/command.
3. For each, look for a **stated purpose** — a doc comment, docstring, or
   contract that says what it does and why a caller would reach for it.
4. Separately, scan for code with **no discernible reason to exist**:
   unreachable branches, unused exports, functions no caller invokes, dead
   parameters.
5. Report only what the visible code establishes. If test coverage plausibly
   lives outside the review window, say so rather than asserting a gap.

## Hard rules

- Report public/exported surface with neither a covering test nor a documented
  purpose under the rule id `untested-public-surface`, severity **info**:
  "Exported/public API with no covering test or documented purpose."
- Never claim or imply DO-178C compliance, certification, or full coverage.
  This lens does not certify anything; it surfaces visible gaps only.
- Private/internal helpers exercised only through a tested public path are
  traced — do not demand a direct test for them.
- Prefer silence over speculation: if you cannot see whether a test exists, do
  not invent a finding.

## What to flag

- A new exported function/class/endpoint with no test touching it AND no
  docstring or contract stating its purpose.
- An exported/public API whose purpose is undocumented, when its behavior is
  non-obvious from the name and signature alone.
- Code with no reason to exist: unreachable branches, unused exports, dead
  parameters, functions no reachable caller invokes (cross-reference
  code-smells).

## What NOT to flag

- Internal/private code covered transitively by a tested public path.
- Generated code, migrations, or trivial getters/setters where a test adds no
  verification value.
- Public surface whose test clearly lives outside the changed files, when you
  can see or reasonably infer that linkage.
- Purpose that is self-evident from a well-named, well-typed signature.

## Scope & honesty

Be explicit in every finding: this is a best-effort, per-file static
traceability check, not a process audit. Full requirement-to-code-to-test
traceability is a standards-level activity that needs linkage this lens cannot
observe from a diff. Treat findings as prompts to add a test or a one-line
purpose, not as evidence of non-compliance. When the code does not give you
enough to be sure, say nothing.
