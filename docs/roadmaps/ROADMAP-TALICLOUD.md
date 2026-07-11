# Roadmap — TaliCloud (managed cloud platform)

> **Note:** TaliCloud is a proprietary, commercial product and is roadmapped separately from the open-source TaliCode developer tools. It is referenced from — but not part of — the [TaliCode MVP](../plans/MVP.md). No TaliCloud code lives in the TaliCode MVP repository.

---

## Overview

TaliCode ships as an **open-core** project: the Core engine and the `tali` CLI are MIT-licensed and fully open-source, giving every developer a local "AI Slop Gatekeeper" on their own machine.

**TaliCloud** is the commercial, hosted layer on top of that open core — a managed cloud platform that acts as a **"background CTO across the org"**. Instead of each developer running the gatekeeper in isolation, teams connect their TaliCode CLI/engine to TaliCloud and get a shared, always-on, organization-wide policy and review surface.

The local MIT tools remain fully functional on their own. TaliCloud is strictly additive: it exists to solve the problems that only appear at team and org scale.

## Capabilities

TaliCloud focuses on org-scale concerns that are intentionally out of scope for the local MIT CLI:

- **Managed LLM routing** — the platform brokers all model calls, so individual developers never need to hold or configure their own Anthropic (or other provider) API keys. Routing, fallback, and model selection are handled centrally.
- **Centralized API-key management** — provider credentials are provisioned, rotated, and revoked in one place by admins, never distributed to laptops. Keys stay server-side.
- **High-concurrency rate limits** — pooled, org-level throughput and quota management so large teams and CI fleets can run the gatekeeper concurrently without tripping per-developer provider limits.
- **SOC2-ready audit logging** — a centralized, tamper-evident record of gatekeeper decisions, model usage, and policy actions across the org, designed to support SOC2 and similar compliance programs.

These are deliberately **beyond** what the local, single-developer MIT CLI does. The open-source tools stay lean and self-contained; the org-scale governance, pooling, and compliance features live only in TaliCloud.

## How the open-source CLI talks to it

TaliCode is built around a **provider seam** in the engine — the abstraction the Core engine uses to reach an LLM provider. In the open-source MVP, that seam typically points at a direct Anthropic API key configured by the developer.

The same seam is the single integration point for TaliCloud:

- The MIT `tali` CLI / Core engine can be pointed at a **TaliCloud endpoint** configured as a *managed provider*, instead of a direct Anthropic key.
- When configured this way, the engine sends its requests to TaliCloud, which performs managed routing, key management, rate limiting, and audit logging on the org's behalf, then returns results through the same provider interface.
- To the open-source engine, TaliCloud simply looks like another provider behind the existing seam — no special-casing required.

Crucially, **no proprietary code enters the MIT repository**. The open-source side only needs its existing, generic provider abstraction (an endpoint URL plus credentials/config). All TaliCloud-specific logic lives on the server and in the separate proprietary client, never in the MIT codebase.

## Separate repository

TaliCloud lives in a **separate, proprietary repository**. The TaliCode MVP repository contains **no TaliCloud code** — not the platform, not a proprietary client, not server logic. The only coupling is the stable, open provider seam described above, which is a generic extension point rather than a TaliCloud-specific interface.

This separation keeps the open-source developer tools cleanly MIT-licensed and independently useful, while the commercial platform evolves on its own cadence under its own license.

## License

**Commercial License — All Rights Reserved.**

TaliCloud is proprietary commercial software. It is **not** covered by the MIT license that governs the open-source TaliCode developer tools (the Core engine and the `tali` CLI). Nothing about the MIT licensing of those tools grants any rights to TaliCloud.

All rights to TaliCloud are reserved by its owner. Use, access, hosting, or deployment of TaliCloud requires a valid **commercial license or active subscription**. There is no open-source grant, no redistribution right, and no implied license of any kind for the TaliCloud platform or its client.

For licensing, pricing, or subscription inquiries, please contact the TaliCode team.

---

## Related roadmaps

- Sibling commercial product: [Roadmap — TaliAgenticServer](./ROADMAP-TALIAGENTICSERVER.md)
- Parent open-source plan: [TaliCode MVP](../plans/MVP.md)
