# Roadmap — Multi-Agent Orchestration

> Deferred design, part of the TaliCode MVP (see [../plans/MVP.md](../plans/MVP.md)). TaliCode is
> positioned as a **multi-agent orchestrator**, and the [agent loop](./ROADMAP-AGENT-LOOP.md) exposes
> three verbs for it — `agent_spawn`, `agent_work_claims`, `agent_report`. This document defines how
> the loop **spawns, coordinates, and aggregates multiple agents** without them colliding.

## Why multi-agent

One agent, one context window, one thing at a time is the wrong shape for a repo-scale gatekeeper:

- **Specialization** — reviewing, fixing, test-writing, and planning want different prompts, tool
  subsets, and models. Splitting them into roles beats one prompt trying to do everything.
- **Parallel coverage** — auditing 40 changed files, or running five lenses, is embarrassingly
  parallel; fanning out cuts wall-clock time.
- **Scale beyond one context** — a large migration or audit doesn't fit one context window. Workers
  each hold a slice; only their *reports* return to the orchestrator, keeping its context bounded.

## The roster

Agents are **roles** — a system prompt + an allowed tool subset + a model (via the per-component model
routing in [ROADMAP-VSCODE](./ROADMAP-VSCODE.md)), declared under `agents:` in `config.tali` (which
already carries `agents.auditor`). Planned roles:

- **Auditor** — detect findings (today's agent).
- **Surgeon** — propose heals ([ROADMAP-HEAL](./ROADMAP-HEAL.md)).
- **Test-gen** — author missing tests ([ROADMAP-TEST](./ROADMAP-TEST.md)).
- **Planner** — decompose work into a plan + phases (`plan_create` / `plan_phase_create`).
- **Integrator** — drive git/PR/Jira connectors to land the result.

The loop's **Decide** phase is the **orchestrator**: it picks which roles run and how work is split.

## Lifecycle: decompose → claim → work → report → aggregate

```
ORCHESTRATOR (the loop's Decide/Act phase)
   │  1. decompose the pass into a work-list (files / findings / plan phases)
   │  2. agent_spawn  — start N workers (role + task + context slice + sub-budget)
   ▼
WORKER × N   (each in its own context window, and its own git worktree if it writes)
   │  3. agent_work_claims — atomically claim unclaimed work items (leases)
   │  4. do the work with the loop's tools (audit / heal / test / …)
   │  5. agent_report — return a structured result (findings, diff, status)
   ▼
ORCHESTRATOR
      6. aggregate reports → the loop's Gate verdict; merge approved worktrees
         back via branch_*/atomic_commit; record episodes in memory
```

### `agent_spawn`
Start a worker for a scoped task: `{ role, task, context (a slice of the parent's — the relevant
files/findings), tools (a subset), model, budget }`. The worker runs with its **own** context window;
the parent does **not** inherit the worker's transcript — only its `agent_report` bubbles up. This is
what keeps the orchestrator's working memory bounded no matter how many workers run.

### `agent_work_claims`
The **coordination primitive** that stops two agents doing the same thing — or worse, editing the same
file. Work items (a file, a finding, a plan phase, a ticket) live in a **claim ledger**
(`.talicode/claims`, or in-memory for a single run). `agent_work_claims` **atomically leases** a batch
of *unclaimed* items to the caller (lease = item id + owner + TTL). Claimed items are invisible to
other agents; a lease is released on `agent_report` or when its **TTL expires** (so a dead worker's
work is reclaimable). This partitions the work-list conflict-free without a central lock step.

### `agent_report`
A worker returns a **structured** result — findings, a proposed diff, test outcomes, status
(done/failed/blocked), and released claims. The orchestrator merges reports into one verdict; a
`failed`/`blocked` report releases the claim for retry or reassignment.

## Isolation & merge-back

Workers that only *read* (most Auditors) need no isolation and run fully parallel. Workers that
**write** (Surgeon, Test-gen) each operate in an **isolated git worktree** so concurrent edits can't
collide; the orchestrator merges the **approved** results back on the branch via `branch_*` +
`atomic_commit` — one commit per logical unit, still through the diff + approve gate. Unchanged
worktrees are discarded.

## Budget, ordering, and safety

- **Budget** — the working-memory soft/hard budget ([ROADMAP-MEMORY](./ROADMAP-MEMORY.md)) is shared;
  the orchestrator allocates a **sub-budget per worker** and, because only reports return, its own
  context stays small. Concurrency is capped.
- **Ordering** — independent items fan out freely; dependent ones (extract-before-consumer, "phase 2
  needs phase 1") are sequenced by the **plan** (`plan_phase_create` phases carry order). The
  orchestrator respects the dependency graph.
- **Safety (Zero-Trust)** — spawned agents **inherit** the parent's approval policy; a worker cannot
  escalate its own side-effecting permissions. A **spawn-depth cap** and a **total-agent cap** are
  runaway backstops (workers cannot recursively spawn without limit). Every mutation still routes
  through the diff + approve gate.

## Where it lives

The orchestrator + role definitions extend `talicode-agent`; the claim ledger and sub-budget
accounting extend `talicode-memory` (working memory); the drivers are the same as the loop's
(`talicode-cli`, the VS Code extension, and [TaliAgenticServer](./ROADMAP-TALIAGENTICSERVER.md), which
is where large autonomous fan-outs really live). It depends on the agentic tool-loop upgrade in
[ROADMAP-HEAL](./ROADMAP-HEAL.md).

## Out of scope / open questions

- **Claim granularity** — file vs. finding vs. symbol vs. phase; too coarse serializes, too fine
  thrashes.
- **Cross-worker conflicts** — two approved diffs that touch the same lines; merge policy and
  re-review of the merged result.
- **Report schema** — a single normalized report shape across roles, and how partial/streaming
  reports aggregate.
- **Scheduling** — priority, fairness, and cost caps when the work-list exceeds the concurrency cap.
- **Determinism** — reproducibility when workers run concurrently against shared external state
  (git, Jira).
