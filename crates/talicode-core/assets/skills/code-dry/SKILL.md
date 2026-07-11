---
name: code-dry
description: >-
  Apply DRY (Do Not Repeat Yourself) — the single-source-of-truth discipline — BEFORE writing
  any code and as a review lens over every diff. MANDATORY for all coding work: invoke it for
  every fix, change, feature, or implementation so each piece of knowledge has ONE authoritative
  representation. Before duplicating logic, a constant, a query, a validation, or a config value,
  reuse or extract the existing source instead of copy-pasting. But DRY is about knowledge, not
  text: never merge code that merely looks alike but represents different decisions, and prefer a
  little duplication over the wrong abstraction (AHA — Avoid Hasty Abstractions). Use whenever you
  add/modify code or the operator says "don't repeat yourself", "dry this up", "deduplicate",
  "single source of truth", or "this is copy-pasted".
---

# DRY — one authoritative source per piece of knowledge

Sources: [GeeksforGeeks — DRY](https://www.geeksforgeeks.org/software-engineering/dont-repeat-yourselfdry-in-software-development/) · [Medium — DRY guide](https://medium.com/@nipun.rajput6586s/the-dry-principle-in-programming-a-comprehensive-guide-e0504a4b393a) · [Wikipedia — DRY](https://en.wikipedia.org/wiki/Don%27t_repeat_yourself) · [Codefinity — DRY](https://codefinity.com/blog/The-DRY-Principle).

> **Every piece of knowledge must have a single, unambiguous, authoritative representation within a system.** (Hunt & Thomas, *The Pragmatic Programmer*, 1999.) When a rule, value, or behavior lives in one place, you change it once and the whole system stays consistent. When it's copy-pasted, every copy is a future bug waiting for the day someone updates only some of them. The opposite of DRY is **WET** — "write everything twice."

This skill is **mandatory** for every coding task — bug fix, change, feature, user story. Run the **dedup check before writing** and the **review pass before committing**. It is a judgment lens like `/code-yagni`, `/code-kiss`, and `/code-smells`; it sits on top of the `/lint-test` mechanical floor and is run by `/code-review`.

## DRY vs YAGNI vs KISS — the split

All three are complementary and mandatory. **`/code-yagni`** decides *whether* code should exist (the smallest amount). **`/code-kiss`** decides *how* the code that must exist is written (the simplest shape). **`/code-dry`** decides *where the knowledge lives* (one authoritative place, not many copies). Run them in review order — YAGNI and KISS first, DRY after the lenses between them: cut what you don't need, write the rest simply, and make sure each fact has a single home. DRY and KISS can pull in opposite directions — a too-clever abstraction added "to be DRY" violates KISS; resolve it with AHA (below).

## The dedup check — BEFORE and WHILE writing

Before you copy-paste, hard-code, or re-implement, ask:

1. **Does an authoritative source already exist?** A helper, constant, schema, config, base class, or util that already owns this knowledge. In this repo that's often `core/utils/client_tenant.py` (`get_client_collection`), the canonical signal schema, `SIGNAL_RULES`, the `BaseCollector`, the indication/client scope objects — reuse them, don't re-derive.
2. **Am I about to write the same logic a second time?** Repeated validation, the same query shape, the same transform, the same error-handling block → extract a function/method and call it from both sites.
3. **Am I hard-coding a value that appears elsewhere?** A magic number, a collection name, a threshold, a default → name it once (a constant / config / env var) and reference it.
4. **Is this knowledge already expressed in another layer?** A business rule duplicated across an API handler, a worker, and a validator is the dangerous kind — they drift. Centralize it.
5. **Is this *real* duplication or just *coincidental*?** Two blocks that look identical today but encode **different decisions** that will evolve independently are NOT a DRY violation — see the AHA guard below. Don't merge them.

Goal: each rule/value/behavior has exactly one home; everything else references it.

## What counts as duplication (and where it bites here)

- **Logic** — the same algorithm/validation/transform pasted across collectors, workers, or API handlers.
- **Constants & config** — the same literal collection name, boost threshold, page size, TTL, or URL repeated instead of a shared constant.
- **Knowledge across layers** — a business rule (e.g. boost overlay math, suppression filtering, dedup key `(entity_id, source_id, indication_id)`) implemented separately in more than one place.
- **Queries / API calls** — the same Mongo filter or external request built by hand in several spots instead of one builder/helper.
- **Schema / shape** — re-declaring a payload structure instead of reusing the Pydantic model in `core/schema`.

## Techniques — give the knowledge one home

- **Extract a function / method** for repeated logic (the most common fix).
- **Name a constant / config** for a repeated literal; one source of truth for the value.
- **Reuse the existing helper / base class** (`get_client_collection`, `BaseCollector`, `translate_text`, the canonical schema) instead of re-implementing.
- **Centralize a cross-layer rule** into one module both callers import.
- **Use a Pydantic model** (`core/schema`) as the single definition of a data shape.

## Don't over-apply — AHA, and "duplication over the wrong abstraction"

DRY is about **knowledge**, not character-for-character text. Over-DRYing is its own failure mode (it collides with `/code-kiss` and `/code-yagni`):

- **AHA — Avoid Hasty Abstractions.** Don't abstract on the *first* repetition. Wait until the pattern genuinely emerges (a common rule of thumb: the third occurrence) and the shared knowledge is real and stable. An abstraction built too early calcifies around the wrong shape.
- **Prefer a little duplication over the wrong abstraction** (Sandi Metz). A wrong abstraction couples things that should evolve separately and is *harder* to unwind than the duplication was. If two sites will diverge, leave them duplicated.
- **Coincidental duplication is not duplication.** Same code, different reasons-to-change → keep them apart. Merging them creates a fake single-source that the next requirement will tear in two.
- **Never abstract across the layer boundary the repo forbids.** Don't "DRY" shared-lake code and client-lens code into one path — the shared-lake / client-lens separation is intentional; that's distinct knowledge, not duplication.
- **Don't let DRY add complexity nobody needs** — a one-caller helper extracted "for reuse" that never gets reused is a `/code-yagni` violation. Extract when there's a *second real caller*, not speculatively.

## Single source of truth — non-negotiables stay correct

DRY removes redundant copies; it never removes a *necessary* check. Don't collapse distinct validations/error paths into one just to dedupe — that's over-simplifying critical logic (a `/code-kiss` pitfall too). Centralizing a validation is good; deleting one of two genuinely-different validations because they "look similar" is a bug.

## Review pass — run this over the diff before committing

`git diff HEAD` (and `--staged`), then flag duplicated knowledge. Tags:

- **extract** — a repeated logic/validation/transform block → pull into a shared function/method and call it.
- **constant** — a repeated literal / magic value / collection name → name it once, reference it.
- **reuse** — re-implemented logic that an existing helper / base class / schema already owns → call the existing source.
- **consolidate** — the same knowledge in two+ places/layers that will drift → one authoritative home.
- **keep** — note *coincidental* look-alike code you deliberately did **not** merge (different reasons to change), so a later reviewer doesn't "fix" it.

**Finding format:** `L<line>: <tag> <description>. <single-source fix>.` — multi-file: `<file>:L<line>: …`. Conclude with **`Single-sourced. Ship.`** if there's no real duplication to remove.

Scope: this pass judges *duplication of knowledge only* — not necessity (`/code-yagni`), simplicity (`/code-kiss`), correctness (`/bug-fix`), safety (`/code-rules`), or the broader smell catalog (`/code-smells`). It overlaps `/code-smells`' *Duplicate Code* — DRY is the before-you-write discipline and the single-source lens; code-smells is the broader catalog.

## Intensity

- **lite** — build it as written; note the duplication + single-source fix in one line, don't force the extraction.
- **full (default)** — reuse existing sources; extract on the second real occurrence; name repeated literals; respect AHA.
- **ultra** — single-source hard-liner; no repeated literal or logic survives — but still bound by AHA (never abstract coincidental duplication or across the shared-lake / client-lens boundary).

Default to **full**.

## Relationship to other skills

- Invoked by `/coding` as a **mandatory** step of every coding task, and run by `/code-review` as a judgment lens alongside `/code-yagni` (necessity), `/code-kiss` (simplicity), `/code-smells` (design), `/code-rules` (safety), and `/clean-code` (style).
- **Runs after `/code-yagni` and `/code-kiss`**: don't dedupe code that shouldn't exist (YAGNI) or hasn't been simplified yet (KISS); single-source what remains.
- Overlaps `/code-smells`' *Duplicate Code* (and its *Inline Class* / *Speculative Generality* over-abstraction smells) — use both: DRY is the mindset, code-smells is the catalog + fix map.
- After any DRY extraction, re-run the `/lint-test` gate, then commit via `/atomic-commit`.
