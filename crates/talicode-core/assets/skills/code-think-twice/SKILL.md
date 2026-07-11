---
name: code-think-twice
description: >-
  Think Twice, Code Once — the think-before-you-write discipline that runs FIRST, ahead of
  every other coding skill. MANDATORY for all non-trivial coding work: before touching the
  editor, understand the requirement and the system, then WRITE DOWN your thinking. Owns the
  three-step process — gain context (ask "why?" five times), draw assumptions (at least two
  candidate paths, stack-ranked with pros/cons), then implement the chosen one. Catches the
  missed-reuse / wrong-abstraction / "computers do what you told, not what you meant" failures
  that come from jumping straight to code. Balances analysis against "done is better than
  perfect" — think more, write less, but don't gold-plate the thinking either. Use whenever you
  start a fix/feature/change, or the operator says "think first", "plan this", "think twice",
  "what's the approach", or you catch yourself about to code without a plan.
---

# Think Twice, Code Once

> **The most expensive code is the code you wrote before you understood the problem.** A few minutes of reflection saves hours of coding later. We jump to implementation without thinking about abstraction or reuse — without even nailing the requirement — and end up asking, in hindsight: *why did we miss the chance to reuse? why wasn't this thought through? why didn't we follow the architecture?* The computer did exactly what we told it — the gap was in our thinking, not its execution. Think twice, code once, and those questions never get asked.

Working inside a system shaped by good abstraction is *rewarding* — and well-designed abstraction saves massive work later. That payoff only exists if someone thought before they typed. This skill is that thinking step.

This skill is **mandatory** for every non-trivial coding task — bug fix, change, feature, user story — and it runs **first**, before `/code-yagni`, `/code-kiss`, and the rest of the pre-code disciplines. Those decide *whether*, *how*, and *where the knowledge lives*; this one ensures you understood *what* and *why* before any of that. It is a judgment lens like `/code-yagni` and `/code-kiss`; it sits on top of the `/lint-test` mechanical floor.

## To think, you have to write

Thinking that stays in your head is wishful thinking — it skips the hard parts. **Writing forces the gaps into the open.** Before implementation, produce a short written plan (a few lines in the response, a scratch note, or the PR/commit body draft). The act of writing it is the point; the artifact is a bonus.

The goal is **maintainable, scalable solutions that are also easy to understand and test** — and you can't aim for that target until you've named it in writing.

## The thinking process — three steps, BEFORE you implement

### 1. Gain context — understand *why*, then ask "why?" five times

Before any implementation, understand **why** the change exists — that defines its impact and tells you what "done" means. Retrieve the actual requirements: read the ticket, the surrounding code, the architecture (CLAUDE.md invariants), the callers and callees. For this repo that means: which layer (shared data lake vs client lens), which collection, which scope object, what the canonical schema expects, what's already built that you can reuse.

Then **ask "why?" five times** to get past the surface request to the real need:

> *Optimise this feature.* → **Why?** It's slow. → **Why does slow matter here?** It's on the signal-feed hot path. → **Why is it on the hot path?** Every client request re-reads the shared lake. → **Why re-read every time?** There's no materialized view. → **Why not materialize it?** Nobody decided the refresh cadence.

Five "why?"s turn "make it fast" into "add a materialized refresh" — a different, better solution than the one you'd have jumped to. Stop early only when the chain bottoms out in a real root cause.

### 2. Draw your assumptions — at least two paths, stack-ranked

Write down what needs to be done and how. **Have at least two candidate paths** — a single option is a decision you didn't actually make. For each, jot the **pros and cons**, then **stack-rank** them, compromising where needed (the repo invariants and the `/code-yagni` ladder are tie-breakers — reuse beats build, minimal beats clever, shared-vs-tenant boundary is non-negotiable).

**Limit the number of assumptions** — two or three real options, not ten — so you stay focused instead of distracted. **This step is iterative: do it at least twice.** The first pass surfaces the obvious options; the second pass is where the better one usually appears (the missed reuse, the existing helper, the abstraction that collapses two paths into one).

### 3. Implement — the easy part, now that the hard part is done

Picking a path is now near-effortless: you already know each option's trade-offs, so choosing is a lookup, not a leap. Implementation runs *faster* because *what* and *how* are settled — you're transcribing a decision, not discovering it mid-keystroke. Hand off to `/code-yagni` → `/code-kiss` → the design lenses to shape the code itself.

## ⚖️ Less code, more thought — and "good enough is good enough"

Two lessons that keep this discipline from backfiring:

1. **Think more, write less.** A productive session is not measured in lines written. A few minutes of reflection routinely saves hours of coding. The win is the line you didn't have to write because you thought first.
2. **Done is better than perfect.** Perfect code is nice; a working solution that moves the project forward is the job. Don't get stuck in an endless "just a little more improvement" loop — on the code *or* on the plan.

Strike the balance: **think, plan, write only what's necessary** — then ship. Over-thinking is its own failure mode (see below). Reduced stress, faster work, and better solutions come from the middle, not from either extreme.

## Hard rules

- **Understand before you type.** Never start implementation on a requirement you can't restate in one sentence, or in a system area you haven't read.
- **Write the plan down.** Thinking that isn't written is incomplete thinking. A few lines beats a perfect mental model that evaporates on the first edit.
- **Always have ≥2 paths before choosing.** One option is not a decision. Name the alternative and why you rejected it.
- **Hunt for reuse and abstraction in the planning step**, not the regret step. The missed-reuse question is cheap to answer *before* coding and expensive *after*.
- **Cap the analysis.** Two or three options, two iterations — then decide. Don't let "think twice" become "think forever."
- **Good enough ships.** Once the solution works and meets the requirement, stop polishing. Perfection past "correct, clear, tested" is gold-plating.

## Don't over-apply — when to skip the ceremony

`/code-think-twice` is for work with real design choices. It is **not** a tax on every keystroke:

- **Trivial / mechanical changes** — a typo, a one-line guard, a rename, a version bump, a lint fix — skip straight to implementation. Forcing a five-why on a one-liner is the analysis-paralysis the skill warns against.
- **The path is genuinely obvious and singular** — when there's truly one correct, minimal way (and `/code-yagni` confirms it), a one-line "approach: X, no real alternative" is the whole plan. Don't manufacture fake options to fill a template.
- **Time-boxed.** If the requirement is clear and the system is familiar, the three steps can take two minutes. Scale the thinking to the stakes: a hot-path refactor or a new abstraction earns the full process; a contained fix earns a sentence.

The test: *would jumping to code risk the wrong abstraction, a missed reuse, or a misunderstood requirement?* If yes, think twice. If no, code once.

## Output — the written plan

When the task warrants it, produce a short plan **before** the diff:

```
Why (root cause / real need): <the five-why endpoint, one line>
Options:
  A) <path> — pros: <…>  cons: <…>
  B) <path> — pros: <…>  cons: <…>
Chosen: <A|B> because <trade-off + reuse/minimality/boundary reason>
```

For a trivial change, collapse it to one line: **`Obvious single path: <X>. Proceeding.`** The format is a thinking aid, not a deliverable — keep it terse.

## Intensity

- **lite** — clear requirement, familiar area: a one-line approach note, then implement.
- **full (default)** — run the three steps; write the plan; ≥2 options stack-ranked; iterate the assumptions once.
- **ultra** — high-stakes / new-abstraction / cross-boundary work: five-why in full, three options, two iterations, explicit reuse and scalability analysis before a line is written.

Default to **full**; drop to **lite** for contained changes; go **ultra** when the operator says "think hard", "design this properly", or the change touches an architectural seam.

## Relationship to other skills

- Invoked by `/coding` as the **first** mandatory step of every non-trivial coding task — it runs *before* `/code-yagni` (necessity), `/code-kiss` (simplicity), `/code-early-return`, `/code-no-nested-loop`, `/code-solid`, `/code-dry`, and `/code-composition`. Those shape the code; this one ensures you're solving the right problem the right way before any of them apply.
- **Feeds `/code-yagni` and `/code-kiss`**: the "draw your assumptions" step *is* where the YAGNI ladder (reuse > build, stdlib > new code) and the KISS "obvious solution" question get applied to candidate paths — think-twice picks the path, those two shape it.
- **Pairs with the `Plan` agent / `EnterPlanMode`** for larger work: this skill is the lightweight, always-on version of that planning loop; escalate to a full plan when the change spans many files.
- Complements `/bug-fix`'s root-cause-first rule and `/multi-phase-fix`'s up-front decomposition — both are this discipline applied to a specific workflow.
- After the plan is set and the code is written, run `/code-review` (which runs the shaping lenses) → the `/lint-test` gate → commit via `/atomic-commit`.
