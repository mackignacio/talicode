# Roadmap — TaliAgenticServer (webhook / agentic daemon)

> **Note:** TaliAgenticServer is a **proprietary, commercial product** roadmapped separately from the open-source TaliCode developer tools. It is referenced from the TaliCode MVP (see [../plans/MVP.md](../plans/MVP.md)) as **Pillar 3** of the vision, but **no TaliAgenticServer code lives in the MVP repo**. This document sketches its webhook surface and how it reuses the MIT-licensed core engine.

---

## 1. Overview

TaliAgenticServer is a centralized, always-on server — deployed, for example, on an EC2 instance — that runs the **TaliCode audit loop** as a team-wide service rather than a per-developer CLI invocation.

Where the open-source `tali` CLI is triggered manually by a single developer on their local machine, TaliAgenticServer is triggered by **webhooks** from the tools a team already uses (GitHub, Jira, CI/CD). It acts as an autonomous **agentic daemon**: it wakes on an inbound event, runs the same detection-and-remediation loop, and takes action on behalf of the whole team.

This is **Pillar 3** of the TaliCode vision — the "AI Slop Gatekeeper" operating continuously and centrally, instead of only at the individual developer's discretion.

---

## 2. Capabilities

### GitHub PR Gatekeeper
Intercepts pull requests via a GitHub webhook, runs the full audit loop against the diff, and either **blocks the PR** (failing the required status check) or **auto-pushes fixes** back to the branch within policy. AI slop never merges unreviewed.

### Jira scaffolder
Listens for Jira ticket events and **provisions compliant feature branches** directly from tickets — correct naming, baseline structure, and architectural scaffolding — so work starts from a known-good, standards-aligned state.

### CI/CD self-healing
Hooks into CI pipelines and **auto-remediates failures within policy** — applying bounded, pre-approved fixes for common failure classes and re-running the pipeline, escalating to a human only when the failure falls outside policy.

### Team-wide policy enforcement
**Centrally enforces architectural standards** across many repositories from a single source of truth. Policy is defined once and applied uniformly, rather than depending on each developer running the CLI locally.

### SOC2-ready audit logging
Emits structured, tamper-evident **audit logs** of every decision, block, fix, and escalation — designed to satisfy SOC2 evidence and review requirements out of the box.

---

## 3. How it reuses the engine

TaliAgenticServer does **not** re-implement detection. It **wraps the same TaliCode detection engine** — the MIT-licensed open-source core — behind a **webhook API**.

- The **engine** (the `@talicode/*` core, the audit loop, the detection rules) is **reused as-is** from the open-source project.
- The **server, orchestration, and integrations** (webhook ingestion, the GitHub/Jira/CI adapters, policy management, audit logging, deployment) are **proprietary** and layered on top.

The boundary is deliberate and one-directional: the proprietary server depends on the open-source engine, but **no proprietary code ever enters the MIT repo**. Improvements to detection accrue to the open-source core; the commercial value lives in the always-on, team-wide orchestration around it.

---

## 4. Separate repository

TaliAgenticServer lives in a **separate, proprietary repository**. The TaliCode MVP repository contains **no TaliAgenticServer code** — not the server, not the webhook handlers, not the integrations.

This separation was a **deliberate scoping decision**: TaliAgenticServer was intentionally moved **out of the MVP scope** so that the open-core developer tools (the Core engine and the `tali` CLI) remain cleanly MIT-licensed and fully open-source, while the commercial product evolves independently on its own roadmap.

See also the sibling commercial product roadmap: [./ROADMAP-TALICLOUD.md](./ROADMAP-TALICLOUD.md).

---

## License

**Commercial License — All Rights Reserved.**

The centralized EC2 Webhook Server described in this roadmap — providing the **GitHub PR Gatekeeper**, **Jira scaffolding**, **CI/CD self-healing**, **team-wide policy enforcement**, and **SOC2-ready audit logging** — is **proprietary commercial software**. All rights reserved.

This product is **NOT** covered by the MIT license that governs the open-source TaliCode developer tools (the Core engine and the `tali` CLI). The MIT license applies only to those open-source components; it does **not** extend to TaliAgenticServer, its source, its integrations, or its deployment.

**Use of TaliAgenticServer requires a commercial license.** No right to use, copy, modify, distribute, or deploy the server is granted by the open-source project's MIT license.

For commercial licensing, please contact the TaliCode team.
