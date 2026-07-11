<p align="center">
  <img src="assets/tali-logo.png" alt="TaliCode" width="200">
</p>

<h1 align="center">TaliCode</h1>

<p align="center"><strong>The AI Slop Gatekeeper — your CTO running in the background.</strong></p>

---

TaliCode is the definitive **AI Slop Gatekeeper**, Zero-Trust execution harness, and multi-agent
orchestrator. As engineering teams adopt local AI agents (Claude Code, Cursor) to write code at
unprecedented speeds, the risk of "AI Slop" — hallucinated dependencies, lazy typing, bloated
boilerplate — entering the repo has skyrocketed. TaliCode operates silently across the local
machine, CLI, and CI/CD pipeline to ensure every line of AI-generated code meets strict, CTO-level
architectural standards **before it is permanently committed**.

This repository is the **detect-only MVP**: a compiled Rust CLI (`tali`) that audits staged code
with Claude and returns line-accurate findings. Healing, the pre-commit hook, the VS Code
extension, and the hosted server products are designed in the [roadmap docs](docs/roadmaps/) and
built later.

## Why "TaliCode"?

The name draws from a Tagalog metaphor that captures both the mechanics and the philosophy of the
platform (the capital **C** splits *Tali + Code*):

- **Tali / Talian** ("to tie / harness") — the **mechanical layer**: the execution harness that
  ties OpenAI, Gemini, and Claude into one pipeline.
- **Talikod** ("going back / behind") — the **architectural layer**: the ultimate backend, a
  secure safety net operating behind the scenes.
- **Talikod** ("to turn your back on") — the **philosophical layer**: a deliberate stance against
  AI slop, turning your back on unverified code, structural bloat, and hallucinated
  vulnerabilities.

## Quick Start

Install TaliCode globally via npm — the compiled `tali` binary ships inside the package (the
esbuild / Rolldown model), so there's nothing to build:

```bash
npm install -g @talicode/core
```

Scaffold a config into your repo:

```bash
tali init
```

This writes a `config.tali` (a custom `.tali` extension — YAML under the hood, like `.tf` for
Terraform), creates a `skills/` folder for your own review lenses, and git-ignores the local
`.talicode/` usage ledger. The starter config:

```yaml
version: "1.0"
name: "TaliCode Local Sweep"

agents:
  auditor:
    provider: "anthropic"
    model: "claude-sonnet-5"
    effort: "medium"
    role: >
      Identify AI slop: hallucinated or unverified imports, dead boilerplate,
      and obvious type/security issues. Report only concrete, line-anchored
      violations; prefer silence over speculation.

execution_flow:
  - step: "slop_sweep"
    agent: "auditor"
    target: "./src/**/*.rs"

# Skills the sweep runs. Empty selects the bundled `code-review` orchestrator
# (all default lenses). List specific skills to narrow the sweep.
skills:
  - code-review
```

Set your key and sweep your staged files:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
tali sweep --staged
```

TaliCode flags AI slop and architectural violations at the exact line, prints a token-spend footer
(`tokens: in … / out … · est. $…`) plus today's running total, and **exits non-zero** when the gate
trips — so the same exit code drives the future pre-commit hook.

## Commands

| Command | What it does |
| --- | --- |
| `tali init` | Scaffold `config.tali` + `skills/` (refuses to overwrite an existing config). |
| `tali sweep [--staged] [--skill <name>] [--json]` | Detect slop/violations in staged or target files. A bare sweep runs the `code-review` orchestrator (all default lenses). |
| `tali heal` | Runs a sweep, then points at the healing roadmap (healing not yet enabled). |
| `tali watch [--json]` | Monitor the current folder/repo and re-sweep on save (debounced). |
| `tali skills [--all]` | List your repo's authored skills; `--all` includes the bundled `code-*` defaults. |
| `tali usage [--json]` | Show token spend: today's total + recent daily history. |

## How it works

TaliCode plays the role a Claude Code session would: its **skill host** loads a `SKILL.md`
(guidance) + `rules.yaml` (concrete rules) for each selected lens and feeds them to the **Auditor**
over a provider seam (Anthropic's Messages API, using structured outputs to force the findings
schema — reliable detection instead of free-text parsing). Findings are `{ file, line, severity,
rule, message }`, aggregated and de-duplicated by file + line.

The hard, valuable part is **trustworthy detection with low false positives**, so the MVP invests
there and keeps a clean provider seam (OpenAI/Gemini can plug in later). The **Surgeon** agent that
*fixes* findings is designed in [ROADMAP-HEAL](docs/roadmaps/ROADMAP-HEAL.md) — TaliCode never
silently overwrites a commit.

### Default skills

The bundled harness is 21 language-agnostic `code-*` judgment lenses plus the `code-review`
orchestrator. `code-review` runs every default lens (simplicity, DRY, guard clauses, bounded loops
& recursion, deterministic concurrency, SOLID, smells, no hardcoded secrets/credentials, magic
strings/numbers, traceability, and more) and renders one verdict. The strict DO-178C
`code-aviation` profile is opt-in. Drop a folder in `skills/` to add or override a lens — no
recompile; `tali skills` shows it immediately.

## Roadmap

Deferred scope is documented, not hidden:

- [ROADMAP-HEAL](docs/roadmaps/ROADMAP-HEAL.md) — the Surgeon agent: Heal Preview (diff + approve)
  and opt-in Auto-Heal.
- [ROADMAP-DEPLOYMENT](docs/roadmaps/ROADMAP-DEPLOYMENT.md) — the pre-commit git hook and the VS
  Code extension (both MIT dev tools).
- [ROADMAP-TALICLOUD](docs/roadmaps/ROADMAP-TALICLOUD.md) — **TaliCloud**, the managed cloud
  platform (proprietary, commercial).
- [ROADMAP-TALIAGENTICSERVER](docs/roadmaps/ROADMAP-TALIAGENTICSERVER.md) — **TaliAgenticServer**,
  the always-on webhook / agentic daemon (proprietary, commercial).

## Contributing

TaliCode's core is a **Rust cargo workspace** (`talicode-core` + `talicode-cli`, emitting the `tali`
binary), packaged for **npm distribution** via a thin `@talicode/core` wrapper and per-platform
packages. The gate for any change is `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`,
and coverage (`cargo llvm-cov`); every module ships with tests. See [docs/plans/MVP.md](docs/plans/MVP.md)
and the phase docs for the build's structure, and [docs/TRACEABILITY.md](docs/TRACEABILITY.md) for
the module → issue → phase map.

## License

TaliCode's developer tools — the Core engine, the `tali` CLI, and (from the roadmap) the VS Code
extension — are **MIT-licensed** and fully open-source. Run it locally, bring your own API keys,
and secure your code. See [LICENSE](LICENSE).

The hosted/commercial offerings — **TaliCloud** and **TaliAgenticServer** — are proprietary and
roadmapped separately; their commercial-license terms live in their respective roadmap docs, not in
this MIT repo.
