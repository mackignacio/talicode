---
name: code-solid
description: >-
  Apply the SOLID principles — the five object-oriented design guidelines (SRP, OCP, LSP, ISP,
  DIP) — BEFORE shaping a function, class, or module and as a review lens over every diff. MANDATORY
  for all coding work that adds or changes a function, class, or module: one reason to change per unit
  (SRP), open for extension but closed for modification (OCP), subclasses substitutable for their base
  (LSP), small focused interfaces/signatures (ISP), depend on abstractions not concretions (DIP). These
  apply to a plain function too — a function that does too many things (SRP), takes a grab-bag parameter
  set (ISP), or hard-wires a concrete dependency instead of taking it as an argument (DIP). Balance with
  KISS/YAGNI — don't prematurely add interfaces/abstractions/parameters for a single implementation. Use
  whenever you add/modify a function, class, or module, or the operator says "SOLID", "single
  responsibility", "this does too much", "open/closed", "Liskov", "fat interface", or "depend on an
  abstraction".
---

# SOLID — five OO design principles

Sources: the operator's SOLID brief (Uncle Bob) · [Wikipedia — SOLID](https://en.wikipedia.org/wiki/SOLID) · [GeeksforGeeks](https://www.geeksforgeeks.org/system-design/solid-principle-in-programming-understand-with-real-life-examples/) · [DigitalOcean](https://www.digitalocean.com/community/conceptual-articles/s-o-l-i-d-the-first-five-principles-of-object-oriented-design) · [freeCodeCamp](https://www.freecodecamp.org/news/what-are-the-solid-principles-in-csharp/) · [Baeldung](https://www.baeldung.com/solid-principles).

> Five OO design guidelines from Robert C. Martin (the acronym coined by Michael Feathers). Applied well, changes stay **isolated and predictable** — you spend less time breaking existing features as the system scales, and avoid spaghetti code, tight coupling, and hard-to-find bugs.

This skill is **mandatory** whenever a change **adds or modifies a function, class, or module / interface**. Run the **SOLID check before shaping the unit** and the **review pass before committing**. It is a judgment lens like `/code-yagni`, `/code-kiss`, `/code-early-return`, `/code-no-nested-loop`, `/code-dry`, `/code-composition`, and `/code-smells`; it sits on top of the `/lint-test` mechanical floor and is run by `/code-review`. The five principles are OO-rooted but most extend to a plain function: **SRP** (one job per function), **ISP** (a lean, focused signature — not a grab-bag of params/flags a caller half-ignores), and **DIP** (take collaborators as arguments rather than hard-wiring a concrete DB/HTTP/LLM handle inside). OCP/LSP are mainly type-level and are usually a no-op for a standalone function. (For a change with no function/class/module at all — e.g. a pure data/markdown edit — the whole lens is a no-op; note that and move on.)

## Order — runs after KISS, early-return, and no-nested-loop

**`/code-solid` runs after `/code-yagni`, `/code-kiss`, `/code-early-return`, and `/code-no-nested-loop`** (before DRY/composition): first prove the code should exist (YAGNI), write it simply (KISS), and flatten its control flow (early-return) and iteration (no-nested-loop), *then* apply SOLID to shape the design principles of the types that survive — and *then* DRY (single-source) and composition (has-a vs is-a). Don't SOLID-ify code that shouldn't exist or hasn't been simplified. SOLID and composition reinforce each other: composition + dependency injection is the primary way you achieve DIP and OCP.

## The five principles — check each when you add/change a function, class, or module

### S — Single Responsibility Principle
**One reason to change per class** — one job. *Signal:* a class/worker/collector doing CRUD **and** AI enrichment **and** caching **and** serialization; "and" in its description; tenant-routing tangled with business logic. *Fix:* split the responsibilities into separate cohesive units (Extract Class). Overlaps `/code-smells` *Large Class*, *Divergent Change*.

### O — Open/Closed Principle
**Open for extension, closed for modification** — add behavior without editing existing code. *Signal:* a growing `if domain == "trials": … elif domain == "regulatory": …` chain you must edit for every new case. *Fix:* dispatch/registry instead — e.g. `SIGNAL_RULES[domain]` dict-dispatch, or the self-registering `BaseCollector` + `collector.yaml` pattern (add a collector by adding a module, not by editing a dispatcher). Overlaps `/code-smells` *Switch Statements*.

### L — Liskov Substitution Principle
**Subclasses must be usable everywhere the base is**, without breaking behavior. *Signal:* a subclass that overrides a base method to `raise NotImplementedError`, narrows a return type, or violates the base's contract (a `BaseCollector` subclass whose `run()`/`collect()`/`normalize()` breaks the promised shape). *Fix:* don't subclass if it isn't a true is-a (see `/code-composition`); honor the base contract or compose instead. Overlaps `/code-smells` *Refused Bequest*.

### I — Interface Segregation Principle
**No client should depend on methods it doesn't use** — prefer small, focused interfaces over one fat general one. *Signal:* a base/protocol with many methods where most implementers `pass`/stub half of them. *Fix:* split into narrow role-interfaces (in Python, small `Protocol`s / ABCs) so each caller depends only on what it needs.

### D — Dependency Inversion Principle
**Depend on abstractions, not concretions** — high-level modules and low-level modules both depend on an abstraction. *Signal:* a worker constructing a concrete Mongo handle / HTTP client / LLM provider inline, hard-wired to one implementation. *Fix:* depend on the abstraction and **inject** the collaborator — read collections via the `get_client_collection` abstraction (not a raw DB handle), translate via the `translate_text()` helper (DeepL-or-LLM) not a concrete provider, pass clients in rather than newing them up. DIP is achieved with composition + dependency injection (see `/code-composition`).

## Balance — don't over-apply (KISS/YAGNI guardrail)

SOLID is a means to maintainability, not a checklist to maximize. **Over-engineering and premature abstraction make code harder to read** — the bigger risk in a young codebase than under-abstraction.

- **No interface/ABC for a single implementation** — that's a `/code-yagni` violation. Introduce the abstraction when there's a *second* real implementor or a proven seam, not speculatively (AHA — Avoid Hasty Abstractions, see `/code-dry`).
- **Don't split a class into five one-method classes** chasing SRP — a cohesive class doing one job is fine even with several methods (`/code-kiss`).
- **Don't invert a dependency that will only ever have one concrete form** — direct use is simpler and clearer.
- Respect the repo's intentional structure: `BaseCollector`, Pydantic `BaseModel`, and framework bases are legitimate contracts (LSP/OCP done right) — don't dismantle them in the name of SOLID.

## Review pass — run this over the diff before committing

`git diff HEAD` (and `--staged`); for each new/changed function, class, or module, check the five (for a plain function, focus on **srp**/**isp**/**dip** — OCP/LSP are usually type-level no-ops). Tags:

- **srp** — a class with more than one reason to change → split responsibilities.
- **ocp** — an if/elif type-switch you must edit per new case → dict-dispatch / registry / polymorphism.
- **lsp** — a subclass that breaks its base's contract (NotImplementedError, narrowed return) → honor the contract or compose.
- **isp** — a fat interface forcing clients to depend on unused methods → split into focused role-interfaces.
- **dip** — a high-level unit hard-wired to a concrete collaborator → depend on an abstraction and inject it.
- **keep** — note an abstraction you deliberately did NOT add (single implementor, no seam yet) so a reviewer doesn't push a premature one.

**Finding format:** `L<line>: <tag> <description>. <SOLID fix>.` — multi-file: `<file>:L<line>: …`. Conclude with **`SOLID. Ship.`** if there's no violation (including diffs with no function/class/module to judge).

Scope: this pass judges *OO design principles only* — not necessity (`/code-yagni`), simplicity (`/code-kiss`), duplication (`/code-dry`), has-a-vs-is-a specifically (`/code-composition`), correctness (`/bug-fix`), safety (`/code-rules`), or the broader smell catalog (`/code-smells`).

## Intensity

- **lite** — build it as written; note the SOLID improvement in one line, don't force the refactor.
- **full (default)** — apply the five where they earn their place; introduce abstractions only at a real seam; flag clear violations (god class, type-switch, broken substitution, fat interface, hard-wired dependency).
- **ultra** — SOLID hard-liner; one-reason-to-change classes, dispatch over conditionals, injected abstractions throughout — but still bound by KISS/YAGNI (no abstraction for a single implementor).

Default to **full**.

## Relationship to other skills

- Invoked by `/coding` as a **mandatory** step for any function/class/module-touching change, and run by `/code-review` as a judgment lens alongside `/code-yagni`, `/code-kiss`, `/code-early-return`, `/code-no-nested-loop`, `/code-dry`, `/code-composition`, `/code-smells`, `/code-rules`, and `/clean-code`.
- **Runs after `/code-yagni`, `/code-kiss`, `/code-early-return`, and `/code-no-nested-loop`** (before DRY/composition): shape the OO design principles of the types that survive the necessity, simplicity, and flatten passes; DRY and composition follow.
- **Pairs with `/code-composition`**: composition + dependency injection is the main vehicle for DIP and OCP; LSP is the test for whether inheritance (vs composition) is legitimate.
- Overlaps `/code-smells` (*Large Class*/*Divergent Change* = SRP, *Switch Statements* = OCP, *Refused Bequest* = LSP) and `/clean-code` (which applies SOLID at the style level) — `/code-solid` is the dedicated principle-level lens; those are the catalog and the style floor.
- After any SOLID refactor, re-run the `/lint-test` gate, then commit via `/atomic-commit`.
