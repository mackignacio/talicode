# Roadmap — The Agent Loop ("running in the background")

> Deferred design, part of the TaliCode MVP (see [../plans/MVP.md](../plans/MVP.md)). TaliCode's
> positioning is **"your CTO running in the background,"** but the MVP ships a one-shot CLI. This
> document defines the **agent loop** that makes "background" real — the continuous
> observe → orient → decide → act → learn cycle — and shows how today's `tali sweep` / `tali watch`
> are degenerate cases of it, and how Heal, Test, Chat, and the server products are *modes and phases
> of the same loop* rather than separate machines.

## Where we are today

- **`tali sweep`** is a **single iteration**: observe the staged files → orient (load memory + select
  skills) → act (one audit call) → report → exit.
- **`tali watch`** re-fires that iteration on save — a *reactive trigger* — but each pass is still
  one-shot: a single structured-output provider call, with no tool use and no learning fed back into
  memory.

What's missing is the loop itself: a **persistent driver**, an **agentic tool-use inner loop** (so
the agent can read, look up, test, and propose fixes within a pass), and the **learn** phase that
closes the cycle back into memory. This document defines all three.

## The loop

One iteration, six phases:

```
        ┌──────────── trigger ────────────┐
        ▼                                  │
   1. OBSERVE   what changed (git diff, changed files, or a prompt)
        │
   2. ORIENT    assemble working context — architectural-map lookup for the
        │       changed files + semantic facts + episodic "last time" +
        │       procedural skill search — within the working-memory budget
        │
   3. DECIDE    plan the pass: which skills apply, run tests?, heal?
        │
   4. ACT       the tool-use inner loop (audit / test / propose heal) — below
        │
   5. GATE      aggregate findings + test results → verdict; block (gate mode)
        │       or surface (background mode: diagnostics / notice / PR comment)
        │
   6. LEARN     record an episode, update memory; at the budget, run the
        │       compression cascade + skill synthesizer; then idle
        └───────────────────────────────► (idle until the next trigger)
```

## The inner loop (agentic tool use) — the core upgrade

Today the Auditor makes **one** structured-output call. The agent loop replaces that single call with
a **tool-enabled loop**, which is what actually turns a request/response into an *agent*:

- **Tools the agent can call:** `changed_files`, `read_file`, `lookup_architecture` (query the map
  instead of grepping the tree), `recall_memory`, `run_skill` / audit a file against a lens,
  `run_test` (a [TaliCode Test](./ROADMAP-TEST.md) adapter), and `propose_heal` (produce a diff).
- **Cycle:** model → `tool_use` request → host executes the tool → `tool_result` → repeat until the
  model emits its findings/verdict. Tools are deterministic where possible (git, the arch map, test
  runners) so the loop is reproducible.
- **Prerequisite:** the tool-use/agentic SDK upgrade — the same one the Surgeon needs; it is tracked
  in [ROADMAP-HEAL](./ROADMAP-HEAL.md). Nothing mutates silently: heals and generated tests go
  through the Heal diff + approve flow.

## Triggers & modes

Same loop *body*, different *drivers*:

| Mode | Trigger | Driver | Verdict behavior |
| --- | --- | --- | --- |
| **One-shot** (today) | `tali sweep` | `talicode-cli` | exit code |
| **Reactive** | file save | `tali watch` / the [VS Code extension](./ROADMAP-VSCODE.md) | editor diagnostics |
| **Gate** | `git commit` / CI | the pre-commit hook ([ROADMAP-DEPLOYMENT](./ROADMAP-DEPLOYMENT.md)) | block on non-zero |
| **Interactive** | a chat message | the [VS Code chat](./ROADMAP-VSCODE.md) | conversational, turn-by-turn |
| **Always-on** | webhook / schedule | [TaliAgenticServer](./ROADMAP-TALIAGENTICSERVER.md) (commercial) | autonomous; PR comments / reports |

## Budget, termination, and "never interrupt the coding LLM"

The loop is bounded by **working memory** (see [ROADMAP-MEMORY](./ROADMAP-MEMORY.md)): the conversation
budget counts `ALL INPUTS + OUTPUT − SEMANTIC tokens`, soft **250K** / hard **500K**.

- The loop keeps iterating while under budget. Crucially, it **never interrupts the coding LLM
  mid-task**: past the soft budget it waits for the coding agent to go idle, *then* runs the
  **compression cascade** — refresh the architectural map → compress the conversation into an episodic
  `summary` (semantic stripped) → reconcile the semantic delta chain → run the skill synthesizer. The
  hard budget forces the cascade immediately.
- In **background** modes the loop **surfaces, it doesn't block** (diagnostics, a dismissible notice,
  a PR comment); only **gate** mode blocks, and only heals/test-gen that the user approves ever touch
  the code.

## Where it lives (crates)

- **Context assembly + budget + compression** already live in `talicode-memory` (working memory) —
  that is the loop's short-term memory and its termination policy.
- The **inner tool loop** extends `talicode-agent`: the Auditor becomes a tool-using agent, and the
  Surgeon (heal) and test-generation are additional agents the loop can schedule.
- The **driver** is per-mode: `talicode-cli` (sweep/watch), the VS Code extension (reactive/chat), and
  TaliAgenticServer (always-on).

## Relationship to the other roadmaps

This loop is the center the others plug into: [ROADMAP-HEAL](./ROADMAP-HEAL.md) (the tool loop + the
heal action), [ROADMAP-TEST](./ROADMAP-TEST.md) (test actions in the Act phase),
[ROADMAP-VSCODE](./ROADMAP-VSCODE.md) (the reactive/interactive local drivers),
[ROADMAP-MEMORY](./ROADMAP-MEMORY.md) (the budget/compression/learn phase), and
[ROADMAP-TALIAGENTICSERVER](./ROADMAP-TALIAGENTICSERVER.md) (the always-on hosted driver).

## Out of scope / open questions

- **Autonomy bounds** — how much the loop does without asking. Default: surface-only; heals and
  generated tests always go through diff + approve. Auto-Heal is an explicit opt-in (ROADMAP-HEAL).
- **Concurrency & debounce** — overlapping triggers (a save mid-pass), and coalescing rapid events
  into one iteration.
- **Multi-agent scheduling** — today there is one Auditor; the loop is designed to schedule several
  agents (auditor, surgeon, test-gen). The orchestration policy (order, when each runs, budget split)
  is open.
- **Always-on economics** — scheduling/backoff and per-period cost caps for the autonomous server
  mode.
