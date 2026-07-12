---
name: code-composition
description: >-
  Apply COMPOSITION OVER INHERITANCE — the "favor has-a over is-a" design discipline — BEFORE
  introducing a class hierarchy and as a review lens over every diff. MANDATORY for all coding
  work that adds or changes a function, class, or type: build behavior by assembling small
  single-responsibility components and injecting them, rather than extending a base class to reuse its
  code. For a plain function this means assembling behavior from small helpers and **injected
  collaborators/callbacks** (delegation), rather than hard-wiring a concrete dependency or branching a
  type-dispatch forest inside one function. Reach for composition/delegation when you would otherwise
  subclass for code reuse, mix in behavior, grow a deep/branching hierarchy, or bury type-switching in a
  function. Inheritance stays only for a genuine, stable is-a with a shared contract (an ABC/interface,
  Pydantic BaseModel, the BaseCollector). Use whenever you add/modify a function, class, or type, or the
  operator says "composition over inheritance", "don't subclass this", "use a mixin?", "this hierarchy
  is getting deep", or "has-a vs is-a".
---

# Composition over inheritance — assemble behavior, don't extend it

Sources: [dev.to — Composition over inheritance](https://dev.to/lovestaco/composition-over-inheritance-a-flexible-design-principle-4ehh) · [Wikipedia](https://en.wikipedia.org/wiki/Composition_over_inheritance) · [python-patterns.guide (GoF)](https://python-patterns.guide/gang-of-four/composition-over-inheritance/) · [GeeksforGeeks (Java)](https://www.geeksforgeeks.org/java/favoring-composition-over-inheritance-in-java-with-examples/).

> **Favor object composition over class inheritance** (Gang of Four). Build an object out of smaller components it *has* and delegates to, instead of extending a base class to inherit what it *is*. Composition asks *"what does this object contain and do?"*; inheritance asks *"what is this object?"* — and most of the time you want the former. Inheritance couples a subclass to its base's internals forever; composition lets you swap a collaborator without touching anything else.

This skill is **mandatory** whenever a change **adds or modifies a function, class, type, or hierarchy**. Run the **design check before introducing inheritance** and the **review pass before committing**. It is a judgment lens like `/code-yagni`, `/code-kiss`, `/code-dry`, and `/code-smells`; it sits on top of the `/lint-test` mechanical floor and is run by `/code-review`. The has-a-over-is-a mindset extends to a plain function: prefer **delegating to small helpers and taking collaborators/callbacks as arguments** over hard-wiring a concrete dependency or embedding a type-dispatch forest in one function (this is the function-level twin of DIP/OCP in `/code-solid`). (For a change with no function/class/type at all — e.g. a pure data/markdown edit — it's a no-op; note that and move on.)

## Where it sits among the disciplines

- **`/code-yagni`** — *whether* the code exists. **`/code-kiss`** — *how simply* it's written. **`/code-dry`** — *where the knowledge lives*. **`/code-composition`** — *how types relate*: prefer has-a wiring over is-a hierarchies. It overlaps `/code-smells` (*Refused Bequest*, *Parallel Inheritance Hierarchies*) and `/clean-code` (SOLID — LSP/DIP); this is the before-you-subclass mindset, those are the catalog + principles.

## The design check — BEFORE you reach for a base class

When you're about to write `class Foo(Bar):` for **code reuse**, stop and ask:

1. **Is this a true is-a, or just code I want to reuse?** If `Foo` isn't genuinely a kind of `Bar` (Liskov-substitutable everywhere `Bar` is used), don't subclass — give `Foo` a `Bar` (or the helper it needs) as a field and call it.
2. **Am I subclassing to share behavior across an axis?** Behavior that varies independently (output target × format × filter; visibility × movement × collision) → make each axis a small component and **inject** it, don't encode it in the class tree.
3. **Would combining traits force a class-per-combination?** That's the **subclass / combinatorial explosion** (2 axes × 2 = 4 classes; add a third → 12). Compose the traits instead — one object holding the components it needs.
4. **Am I reaching for multiple inheritance or a mixin?** Prefer injecting a collaborator. Mixins and multiple inheritance carry the same liabilities (untested combinations, attribute-name collisions, non-obvious MRO/base-ordering) — readability gain, correctness risk.
5. **Will the subclass depend on the base's internals?** That's the **fragile base class** trap — a base change silently breaks subclasses. A composed collaborator only exposes its public surface.

Default: **has-a + delegation/injection**. Use inheritance only when rung 1 is a clear, stable yes.

## Anti-patterns this replaces (and how)

- **Deep/branching hierarchy for reuse** → extract each behavior into a component; the domain object holds and delegates to them (method forwarding).
- **The `if`-forest god class** (one class with feature flags branching everywhere) → a small strategy/handler component per feature, injected; features stay localized and deletable.
- **Mixin / multiple inheritance for behavior** → inject the behavior as a collaborator object (dependency injection).
- **`type()` / dynamic class generation** → plain objects wired at runtime; never build classes on the fly (breaks editor nav, type-checkers, debugging).
- **Adapter / Bridge / Decorator** (GoF) are the composition shapes to reach for: wrap a foreign interface to fit (adapter), split what-callers-see from how-it-works (bridge), or stack same-interface components (decorator) — instead of a subclass per case.

## When inheritance IS the right call — keep it

Composition is the default, not a ban. Inheritance is correct for:

- **A genuine, stable is-a with Liskov substitutability** — the subclass is usable everywhere the base is.
- **A shared contract via an ABC / interface** — an abstract base defining the method surface, with little/no implementation to inherit (this is interface inheritance, the safe kind).
- **A single parent that already provides most of the functionality** and won't churn.
- **Repo bases that are designed for it:** `BaseCollector` (every collector *is a* collector with the `run()/collect()/normalize()` contract — extend it, don't re-wire it), Pydantic `BaseModel` schemas in `core/schema`, FastAPI/framework base classes. Don't "composition-ize" these — fighting a framework's intended inheritance is its own anti-pattern.

> Trade-off to accept consciously: composition adds **forwarding boilerplate** (delegating methods) and **more small classes** to navigate. That cost is usually worth the flexibility — but for a stable one-parent is-a, a subclass is simpler (a `/code-kiss` consideration). Don't convert healthy inheritance just to chase the principle.

## Review pass — run this over the diff before committing

`git diff HEAD` (and `--staged`); for each new/changed class ask whether inheritance earned its place, and for each new/changed function whether it delegates to injected helpers rather than hard-wiring concretions or burying a type-switch. Tags:

- **is-a?** — inheritance used for code reuse where it isn't a true is-a → convert to a held field + delegation.
- **explosion** — a class-per-combination (or heading there) → compose the independent behaviors instead.
- **fragile** — a subclass overriding/relying on base internals → inject a collaborator with a stable public surface.
- **mixin** — multiple inheritance / a behavior mixin → inject the behavior as a component.
- **forest** — feature-flag `if`-forest in a god class, or a type-dispatch `if`/`elif` chain inside a function → split into injected strategy components / a dispatch table of callables.
- **keep** — note inheritance you deliberately left (true stable is-a, ABC contract, `BaseCollector`/Pydantic base) so a reviewer doesn't "composition-ize" it.

**Finding format:** `L<line>: <tag> <description>. <composition fix>.` — multi-file: `<file>:L<line>: …`. Conclude with **`Composition sound. Ship.`** if there's no inheritance/delegation misuse (including diffs with no function/class/type to judge).

Scope: this pass judges *type relationships only* — not necessity (`/code-yagni`), simplicity (`/code-kiss`), duplication (`/code-dry`), correctness (`/bug-fix`), safety (`/code-rules`), or the broader smell catalog (`/code-smells`).

## Intensity

- **lite** — build it as written; note the composition alternative in one line, don't force the refactor.
- **full (default)** — prefer has-a wiring; reach for inheritance only on a clear stable is-a / ABC contract; flag reuse-by-subclassing.
- **ultra** — composition hard-liner; no implementation inheritance for reuse survives — but still keep genuine is-a / ABC contracts and framework bases (`BaseCollector`, Pydantic).

Default to **full**.

## Relationship to other skills

- Invoked by `/coding` as a **mandatory** step for any function/class/type-touching change, and run by `/code-review` as a judgment lens alongside `/code-yagni`, `/code-kiss`, `/code-dry`, `/code-smells`, `/code-rules`, and `/clean-code`.
- **Runs after `/code-dry`, before `/code-smells`**: once you've single-sourced the knowledge, decide how the types that hold it relate — composition vs inheritance — before the broader design smell pass.
- Overlaps `/code-smells` (*Refused Bequest*, *Parallel Inheritance Hierarchies*, *Inline Class*) and `/clean-code` SOLID (LSP, DIP, ISP) — use all three; this is the has-a-over-is-a mindset, they are the catalog and the principles.
- After any composition refactor, re-run the `/lint-test` gate, then commit via `/atomic-commit`.
