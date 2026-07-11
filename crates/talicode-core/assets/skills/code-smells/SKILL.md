---
name: code-smells
description: >-
  Scan the working-tree diff for code smells and fix the introduced ones before committing. Run
  it before every atomic commit (the /atomic-commit skill calls it as a precondition) and
  whenever your diff adds or changes code. Use when the user says "check for code smells", "clean
  this up", "refactor", or "review smells". Catalogs the smell families (bloaters, OO abusers,
  change preventers, dispensables, couplers) with repo-specific examples, and maps each smell to
  its canonical refactoring. Never introduce a smell deliberately; fix the ones your diff adds,
  leave a note for pre-existing ones.
---

# Avoid code smells

Reference: <https://refactoring.guru/refactoring/smells>. Before adding or changing code — and **before every atomic commit** — scan the diff for these patterns. Smells are signals: never introduce one deliberately, fix any your diff adds, and fix pre-existing ones opportunistically only when they're small, obviously-correct cleanups in the area you're already touching.

## How to run it (pre-commit)

1. `git diff HEAD` (and `git diff --staged`) — look at exactly what you changed.
2. For each hunk, check it against the catalog below. Flag every smell **your change introduces**.
3. Fix the introduced smells with the mapped refactoring (keep the diff scoped — apply the smallest refactoring that resolves the smell).
4. For a smell that was **already there** (not introduced by your diff): if it's a tiny, obviously-correct fix in the same area, apply it (boy-scout rule); otherwise leave a note / file a ticket rather than expanding the PR — silent refactors mixed into unrelated work make review harder.
5. Re-run the `/lint-test` gate after any fix, then proceed to `/atomic-commit`.

> This skill does not block on pre-existing debt — it blocks on **newly-introduced** smells. A change isn't ready to commit if it adds one.

## Catalog — smells to avoid

### Bloaters — code grown out of proportion
- **Long Method** — a function so long the reader can't hold it in their head; past ~50 lines of real logic deserves a second look.
- **Large Class** — a worker / router / collector doing everything (CRUD + AI + caching + serialization). Tenant-aware logic mixed with business logic is a frequent offender.
- **Primitive Obsession** — raw dicts / strings where a typed object would carry intent (untyped `scope: dict` instead of a Pydantic model).
- **Long Parameter List** — five+ positional args, especially when several travel together (`client_id, indication_id, signal_id, source_id, entity_id, …`).
- **Data Clumps** — the same field group recurring across signatures (`entity_id + source_id + indication_id` everywhere).

### Object-Orientation Abusers — broken or partial OOP
- **Alternative Classes with Different Interfaces** — two collectors/workers doing the same job with inconsistent names (`run()` vs `execute()` vs `collect_all()`).
- **Refused Bequest** — a subclass overriding most of its parent with `pass` / `raise NotImplementedError`.
- **Switch Statements** — long `if domain == "trials": … elif domain == "regulatory": …` chains on a type tag (a `SIGNAL_RULES[domain]` dict-dispatch is fine; a 14-branch if/elif is not).
- **Temporary Field** — an attribute only meaningful during certain calls (`self._pending_overlay` set by one method, read by another, else None).

### Change Preventers — small changes that ripple
- **Divergent Change** — one module edited for unrelated reasons every release (a `utils.py` owning caching, parsing, AND date math).
- **Parallel Inheritance Hierarchies** — adding a `FooCollector` forces a `FooSignalRule`, `FooTest`, `FooSchema` in lockstep.
- **Shotgun Surgery** — one feature change touching 15 files; the abstraction boundary is in the wrong place.

### Dispensables — pointless code
- **Comments explaining what** — a comment restating the next line; rename the symbol instead. Comments explain *why*, not *what*.
- **Duplicate Code** — copy-pasted blocks across collectors / workers / API handlers.
- **Data Class** — fields + getters, no behavior, when business logic that belongs on the type lives elsewhere.
- **Dead Code** — unused imports, branches, commented-out blocks, `_legacy_*` helpers nobody calls.
- **Lazy Class** — a class so thin it does less than its wrapper justifies (`class FooManager: def get(): return foo_dict[k]`).
- **Speculative Generality** — abstract bases, hooks, "extension points" for hypothetical future needs. Don't design for users that don't exist yet.

### Couplers — too much knowledge of other modules
- **Feature Envy** — a method calling more methods on another object than on `self`; it belongs there.
- **Inappropriate Intimacy** — two classes poking each other's `_private` attributes / sharing internal state.
- **Message Chains** — `client.scope().overlay().get("own_drugs")[0]` — every link is a coupling.
- **Middle Man** — a class whose every method is a one-line delegation; remove the layer.
- **Incomplete Library Class** — fighting a third-party lib for one missing method; wrap it via a helper, don't fork it.

## Fix map — smell → canonical refactoring

Apply the smallest refactoring that resolves the smell; keep diffs scoped.

**Composing methods:** Extract Method (Long Method, Duplicate Code, Comments-explaining-what) · Inline Method (Middle Man, some Speculative Generality) · Extract Variable (Comments-explaining-what) · Replace Temp with Query.

**Moving features between objects:** Move Method / Move Field (Feature Envy, Shotgun Surgery, Parallel Inheritance Hierarchies) · Extract Class (Large Class, Data Clumps, Temporary Field, Divergent Change) · Inline Class (Lazy Class, Speculative Generality) · Hide Delegate (Message Chains) · Remove Middle Man.

**Organizing data:** Replace Data Value with Object — e.g. `indication_id: str` → typed `IndicationId` (Primitive Obsession) · Introduce Parameter Object (Long Parameter List, Data Clumps) · Encapsulate Field (Data Class, Inappropriate Intimacy).

**Simplifying conditionals:** Replace Conditional with Polymorphism — a type-tag switch → per-subtype methods or a `dict[str, Callable]` dispatch (Switch Statements) · Decompose Conditional · Replace Nested Conditional with Guard Clauses (early-return at the top, flat happy path).

**Generalization:** Pull Up / Push Down Method or Field · Extract Superclass / Extract Interface (Alternative Classes with Different Interfaces) · Replace Inheritance with Delegation (Refused Bequest) · Collapse Hierarchy (Speculative Generality hierarchies).

**Dead code:** Remove unused imports, branches, files, env vars, finished-migration feature-flag scaffolding. Don't leave `_legacy` / `# TODO: remove after Q3` breadcrumbs unless you actually need one — if you do, file a ticket and reference it.

## Relationship to other skills

- `/atomic-commit` runs this skill as a pre-commit precondition — no commit lands with a newly-introduced smell.
- After fixing a smell, run `/lint-test` (ruff + flake8 + pylint 10.00/10) before committing.
- This skill is about design smells, not lint findings — the two gates are complementary.
