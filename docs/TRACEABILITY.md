# Traceability Matrix

> Part of the [TaliCode MVP](plans/MVP.md). Closes the loop the build follows: **every line of code
> traces to a GitHub issue → a phase doc → a section of the MVP plan.** This is the real linkage the
> bundled `code-traceability` lens checks against.

Every source file carries a `// SPDX-License-Identifier: MIT` header and a module-level
`//! Implements #<issue>` reference. This document maps each module (and each non-code artifact) to
its implementing issue, phase, and MVP section. Keep it current as issues land.

## Phases → issues

| Phase | Doc | Issues |
| --- | --- | --- |
| 0 — Scaffold | [phase-0-scaffold.md](plans/phase-0-scaffold.md) | #2 workspace · #3 hygiene (LICENSE / .gitignore / toolchain) · #4 CI |
| 1 — CLI & config | [phase-1-cli-config.md](plans/phase-1-cli-config.md) | #5 CLI dispatch · #6 config · #7 init |
| 2 — Provider & auditor | [phase-2-provider-auditor.md](plans/phase-2-provider-auditor.md) | #8 provider trait · #9 anthropic · #10 auditor |
| 3 — Skill host | [phase-3-skill-host.md](plans/phase-3-skill-host.md) | #11 skill · #12 discover · #13 invoke · #14–#19 the 22 bundled skills |
| 4 — Sweep & report | [phase-4-sweep-report.md](plans/phase-4-sweep-report.md) | #20 git · #21 report · #22 sweep · #23 heal |
| 5 — Usage & watch | [phase-5-usage-watch.md](plans/phase-5-usage-watch.md) | #24 usage ledger · #25 usage cmd · #26 watch · #27 skills cmd |
| 6 — npm distribution | [phase-6-npm-dist.md](plans/phase-6-npm-dist.md) | #28 launcher + postinstall downloader · #29 release workflow + smoke + scope stubs |
| 7 — Docs & license | [phase-7-docs-license.md](plans/phase-7-docs-license.md) | #30 roadmaps · #31 README · #32 this matrix |
| Post-3 — Enrichment | (global-skills single source) | #40 enrich the 22 bundled skills to rich guidance |
| 8 — Long-term memory | [phase-8-memory.md](plans/phase-8-memory.md) | #44 semantic · #45 episodic · #46 auto-promotion · #47 procedural search · #48 architecture · #49 working · #50 config · #51 wiring · #52 commands · #53 init · #54 docs |

## Modules → issue → phase → MVP section

| Source module | Issue | Phase | MVP section |
| --- | --- | --- | --- |
| `crates/talicode-core/src/lib.rs` | #2 | 0 | Architecture — workspace |
| `crates/talicode-cli/src/main.rs` | #5 | 1 | Commands — CLI entry/dispatch |
| `crates/talicode-cli/src/commands/mod.rs` | #5 | 1 | Commands |
| `crates/talicode-core/src/config.rs` | #6 | 1 | Config schema (`config.tali`) |
| `crates/talicode-cli/src/commands/init.rs` | #7 | 1 | Commands — `init` |
| `crates/talicode-core/src/provider/mod.rs` | #8 | 2 | Provider seam |
| `crates/talicode-core/src/provider/anthropic.rs` | #9 | 2 | Provider seam — anthropic |
| `crates/talicode-agent/src/auditor.rs` | #10 | 2 | Auditor agent |
| `crates/talicode-skills/src/host/mod.rs` | #11 | 3 | Skill host — module root |
| `crates/talicode-skills/src/host/skill.rs` | #11 | 3 | Skill host — skill model/parser |
| `crates/talicode-skills/src/host/discover.rs` | #12 | 3 | Skill host — discovery (rust-embed) |
| `crates/talicode-skills/src/host/invoke.rs` | #13 | 3 | Skill host — orchestrator expansion |
| `crates/talicode-skills/assets/skills/**` (22) | #14–#19, #40 | 3 | Default skills (authored + enriched) |
| `crates/talicode-core/src/git.rs` | #20 | 4 | Git staged reader |
| `crates/talicode-core/src/report.rs` | #21 (types from #10) | 4 | Reporting |
| `crates/talicode-cli/src/commands/sweep.rs` | #22 | 4 | Commands — `sweep` |
| `crates/talicode-cli/src/commands/heal.rs` | #23 | 4 | Commands — `heal` |
| `crates/talicode-core/src/usage.rs` | #24 | 5 | Token-spend reporting |
| `crates/talicode-cli/src/commands/usage.rs` | #25 | 5 | Commands — `usage` |
| `crates/talicode-core/src/watch.rs` | #26 | 5 | Watch mode |
| `crates/talicode-cli/src/commands/watch.rs` | #26 | 5 | Commands — `watch` |
| `crates/talicode-cli/src/commands/skills.rs` | #27 | 5 | Commands — `skills` |
| `crates/talicode-memory/src/memory.rs` | #44 | 8 | Semantic memory |
| `crates/talicode-memory/src/episode.rs` | #45, #46 | 8 | Episodic memory + auto-promotion |
| `crates/talicode-skills/src/host/retrieve.rs` | #47 | 8 | Procedural memory (skill search) |
| `crates/talicode-memory/src/architecture.rs` | #48 | 8 | Architectural memory |
| `crates/talicode-memory/src/context.rs` | #49 | 8 | Working memory (assemble/compress/chain) |
| `crates/talicode-core/src/config.rs` (`MemoryConfig`) | #50 | 8 | Config — `memory:` |
| `crates/talicode-agent/src/auditor.rs` (memory wiring) | #51 | 8 | Auditor memory injection |
| `crates/talicode-skills/src/host/invoke.rs` (`InvokeOptions`) | #51 | 8 | Memory + retrieval wiring |
| `crates/talicode-cli/src/commands/sweep.rs` (memory context) | #51 | 8 | Sweep memory assembly |
| `crates/talicode-cli/src/commands/memory.rs` | #52 | 8 | Commands — `memory` |
| `crates/talicode-cli/src/commands/map.rs` | #52 | 8 | Commands — `map` |

## Non-code artifacts → issue → phase

| Artifact | Issue | Phase | MVP section |
| --- | --- | --- | --- |
| `Cargo.toml` (workspace) | #2 | 0 | Architecture |
| `rust-toolchain.toml`, `.gitignore`, `LICENSE` | #3 | 0 | Licensing / scaffold |
| `.github/workflows/ci.yml` | #4 | 0 | Verification — the gate |
| `package.json`, `bin/tali.js`, `scripts/install.js` (launcher + postinstall downloader) | #28 | 6 | npm distribution |
| `.github/workflows/release.yml`, `scripts/npm-smoke.sh`, `npm/modules/**` (`@talicode.ai/*` stubs), `scripts/publish-modules.sh` | #29 | 6 | npm distribution |
| `docs/roadmaps/*.md` | #30 | 7 | Roadmap docs |
| `README.md` | #31 | 7 | README updates |
| `docs/TRACEABILITY.md` | #32 | 7 | Execution methodology |
| `docs/plans/phase-8-memory.md` | #44–#54 | 8 | Phase 8 plan |
| `docs/roadmaps/ROADMAP-MEMORY.md` | #54 | 8 | Memory roadmap |

## How to keep this current

When you add a module, give it an SPDX header and a `//! Implements #<issue>` line, then add a row
here. The invariant: `grep -rn '//! Implements' crates` should account for every `.rs` file, and
every issue number here should map back to a merged PR.
