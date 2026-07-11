# TaliCode MVP — CLI Core (Detect-Only)

## Context

**Positioning (canonical):** TaliCode is the definitive **AI Slop Gatekeeper**,
Zero-Trust execution harness, and multi-agent orchestrator. As engineering teams adopt
local AI agents (Claude Code, Cursor) to write code at unprecedented speeds, the risk of
"AI Slop" — hallucinated dependencies, lazy typing, bloated boilerplate — entering the
repo has skyrocketed. TaliCode operates silently across the local machine, CLI, and CI/CD
pipeline to ensure every line of AI-generated code meets strict, CTO-level architectural
standards before it is permanently committed. (The README intro is updated to this exact
wording; this plan builds the local/CLI slice of it — CI/CD lands in the roadmap.)

That full vision spans ~4 products. This plan builds the **one piece that proves the core
value**: a CLI that detects "AI slop" and architectural violations in staged code using
Claude, with line-accurate findings.

**Repo:** the MVP lives at `https://github.com/mackignacio/talicode`. The local working
copy should track that remote as `origin` (fresh clone, or `git init` + `git remote add
origin`); work lands on feature branches and PRs against it — never committing straight to
the default branch.

Decisions confirmed with the user:
- **Scope:** Detect only for now. Healing (preview + auto-write) is designed in a
  separate roadmap doc, not built.
- **Provider:** Anthropic / Claude first, behind a provider interface so
  OpenAI/Gemini can plug in later.
- **Depth:** CLI core only. Pre-commit hook and VS Code extension are captured in
  a roadmap doc, not built. The `tali heal` command *surface* is scaffolded
  now (runs detection + points to the healing roadmap) so the CLI shape is stable.

The guiding principle (from the review of the README): the hard, valuable part is
**trustworthy detection with low false positives**, not orchestration plumbing. So
the MVP invests in structured, line-anchored findings and a clean provider seam.

## Deliverables

1. A working Cargo workspace that compiles to a single `tali` binary (`talicode-cli`
   + `talicode-core` crates), plus the npm distribution layer (wrapper + platform
   packages) so `npm install -g @talicode/core` yields the native binary, exposed as the
   short `tali` command.
2. Commands: `tali init`, `tali sweep`, `tali heal` (heal = detect +
   "healing not yet enabled" notice), `tali skills`, `tali usage`, and
   `tali watch` (monitor the current folder/repo, sweep on change).
3. Claude-backed Auditor that returns structured findings (rule, severity, line, message).
3b. **Token-spend reporting**: a per-execution footer (in/out/cached tokens + estimated
   cost) and a daily rolling total, backed by a local `.talicode/usage.jsonl` ledger and
   surfaced via `tali usage`.
4. **22 bundled default skills** (see "Default skills" section): the `code-*` coding
   harness — 21 judgment lenses + the `code-review` orchestrator — that TaliCode loads
   and runs as the host (TaliCode plays Claude's role over the local `SKILL.md` folders).
5. Four roadmap docs (see "Roadmap docs" section): healing, local dev-tool deployment
   (pre-commit hook + VS Code extension), **TaliCloud**, and **TaliAgenticServer**.
6. **MIT licensing** (MVP is fully open-source): a root MIT `LICENSE`, crate + npm
   `license` fields set to `MIT`, SPDX headers on source files, and a README License
   section trimmed to the MIT statement + a pointer that the commercial server products
   are roadmapped separately (their commercial-license text lives in the roadmap docs).

## Architecture

**Compiled Rust core, distributed via npm** — the **Rolldown / esbuild / swc model**:
the logic is a Cargo workspace producing a single static `tali` binary per platform;
the binary is shipped inside per-platform npm packages with a thin wrapper package so
`npm install -g @talicode/core` still works. The README's npm Quick Start therefore stays
valid (see "README updates" below for the one part that changes).

```
talicode/
  Cargo.toml                   # workspace manifest
  rust-toolchain.toml          # pin the toolchain
  .gitignore
  LICENSE                      # MIT — the open-core developer tools (core, CLI, VS Code ext)
  assets/
    tali-logo.png              # project logo (shown in the README header)
  crates/
    talicode-cli/              # binary crate → emits the `tali` executable ([[bin]] name = "tali")
      Cargo.toml
      src/
        main.rs                # clap entry; dispatch to commands
        commands/
          mod.rs
          init.rs              # write starter config.tali + scaffold skills/
          sweep.rs             # detect over staged/target files via the skill host
          heal.rs              # run sweep, print healing-roadmap notice
          skills.rs            # list USER-DEFINED (repo) skills; --all adds bundled
          usage.rs             # show token spend: today's total + recent daily history
          watch.rs             # monitor the current folder/repo; sweep on change; stream findings
    talicode-core/             # library crate (all logic; unit-testable)
      Cargo.toml
      src/
        lib.rs
        config.rs              # serde structs for config.tali + validation
        git.rs                 # staged files via `git` (std::process::Command)
        provider/
          mod.rs               # `Provider` trait (async) + registry keyed by `provider`
          anthropic.rs         # reqwest call to the Messages API; tool-use schema
        auditor.rs             # build prompt + tool schema → Vec<Finding>
        host/                  # TaliCode's skill runtime — "TaliCode acts as Claude"
          mod.rs
          skill.rs             # Skill struct (serde); parse SKILL.md frontmatter + rules.yaml
          discover.rs          # find skill folders (embedded defaults + repo-root skills/)
          invoke.rs            # expand orchestrator, compose lenses, aggregate verdict
        report.rs              # Finding + Severity; terminal render + exit-code decision
        usage.rs               # Usage struct; per-execution summary + daily ledger
      tests/                   # integration tests (fake Provider; no live LLM)
      assets/skills/           # BUNDLED defaults, EMBEDDED into the binary at compile
                               # time (rust-embed). Each = SKILL.md + rules.yaml
        code-think-twice/      #   think before writing
        code-kiss/             #   simplicity
        code-yagni/            #   no speculative code
        code-dry/              #   single source of truth
        code-early-return/     #   guard clauses
        code-clear-exit/       #   clear, non-branching exits (composes w/ early-return) (NEW; default)
        code-no-nested-loop/   #   no O(n^2) loop-in-loop
        code-bounded-loops/    #   loops have a fixed, verifiable upper bound (NEW; default)
        code-bounded-recursion/#   recursion must have a finite terminating base case (NEW; default)
        code-deterministic-concurrency/ # same output regardless of scheduling; no sleep/setTimeout for sync (NEW; default)
        code-composition/      #   composition over inheritance
        code-solid/            #   SOLID
        code-smells/           #   design smells
        code-nitpick/          #   senior-engineer review
        code-rules/            #   NASA Power-of-10 (language-agnostic)
        code-magic-strings/    #   no hardcoded meaningful string literals
        code-magic-numbers/    #   no unexplained numeric literals
        code-no-keys/          #   no hardcoded secrets
        code-no-credentials/   #   no embedded logins / plaintext passwords
        code-traceability/     #   best-effort requirement/test traceability (NEW; default)
        code-aviation/         #   DO-178C safety-critical profile (strict; opt-in)
        code-review/           #   ORCHESTRATOR: runs all lenses, one verdict
  npm/                         # npm DISTRIBUTION layer (esbuild/Rolldown-style)
    core/                      #   published as @talicode/core: `npm i -g @talicode/core`
      package.json             #     "bin": { "tali": "./bin/tali.js" }; optionalDependencies on platform pkgs
      bin/tali.js              #     launcher: resolve platform pkg, exec the native `tali` binary
    platform/                  #   one package per target triple, each ships the prebuilt `tali` binary
      talicode-darwin-arm64/   #     package.json + tali (Mach-O)
      talicode-darwin-x64/
      talicode-linux-x64/
      talicode-linux-arm64/
      talicode-win32-x64/      #     tali.exe
  docs/                        # ALL repo markdown lives here, grouped by category folder
    plans/
      MVP.md                   #   this plan (the detect-only MVP)
    roadmaps/
      ROADMAP-HEAL.md          #   Heal Preview + Auto-Heal design (deferred)
      ROADMAP-DEPLOYMENT.md    #   local dev-tool integrations: pre-commit hook + VS Code extension
      ROADMAP-TALICLOUD.md     #   TaliCloud — managed cloud platform (commercial; separate repo)
      ROADMAP-TALIAGENTICSERVER.md # TaliAgenticServer — webhook/agentic daemon (commercial; separate repo)
  skills/                      # repo-level TaliCode skills (user-authored; discovered
                               # + merged over embedded defaults, repo wins on name)
```

**Docs layout:** every repo markdown doc lives under `docs/` in a category subfolder —
`docs/plans/` for plans (this MVP → `docs/plans/MVP.md`) and `docs/roadmaps/` for
roadmaps — so plans and roadmaps are grouped, not scattered. The one exception is the
top-level `README.md`, which stays at the repo root because npm and GitHub require it
there for package/landing rendering.

Each skill folder = `SKILL.md` (frontmatter `name` + `description`, then the audit
guidance) + `rules.yaml` (`{ id, message, severity }` rule list). The **bundled
defaults are embedded into the binary** (via `rust-embed`) so a single compiled
`tali` is self-contained; a repo's on-disk `skills/<name>/` is read at runtime and
overrides an embedded default of the same name — so users add/override skills without
recompiling.

**Crates (dependencies):** `clap` (derive) for the CLI; `serde` + `serde_yaml` +
`serde_json` for config, `rules.yaml`, and the Messages-API wire types; `reqwest`
(rustls, `json`) + `tokio` for the async Anthropic call; `rust-embed` for the bundled
skills; `globset` for `target` globs; `notify` for the `watch` filesystem watcher;
`anyhow` + `thiserror` for errors. `git` is
invoked via `std::process::Command` (no C dep, keeps the build static). Keep the tree
small.

**Testing policy (hard requirement): every module ships with tests.** Each `.rs`
module carries a colocated `#[cfg(test)] mod tests`, with cross-module flows in
`crates/*/tests/`. The `Provider` **trait** makes the LLM trivially fakeable — unit
tests use an in-process fake impl (no network, no live LLM); the one live integration
test is `#[ignore]`d unless `ANTHROPIC_API_KEY` is set. A coverage gate
(`cargo llvm-cov`, lines + functions) is wired into the test task. No module lands
without its test.

### Licensing (`LICENSE` + README) — MIT (MVP is fully open-source)
This repo is **MIT**, full stop. Everything the MVP builds — the Core Engine, CLI, and
(from the roadmap) the VS Code extension (`@talicode/core`, `@talicode/cli`,
`@talicode/vscode`) — is 100% free and open-source. Root `LICENSE` holds the MIT text.
- **Metadata:** each crate's `Cargo.toml` `license = "MIT"`; each npm `package.json`
  `"license": "MIT"`; per-file `// SPDX-License-Identifier: MIT` headers.
- **No commercial-license content in the MVP.** The proprietary/commercial framing for the
  server products (TaliCloud, TaliAgenticServer) is **not** in this repo's README or
  license — it lives in their roadmap docs (see "Roadmap docs"). The README's License &
  Enterprise section is trimmed to the MIT statement plus a one-line pointer that the
  hosted/commercial offerings are roadmapped separately.

**README updates (part of the build):** the npm **Quick Start
(`npm install -g @talicode/core`) stays valid** — that's exactly what the npm
distribution layer delivers. The Quick Start's **example commands become the short `tali`
form** (`tali init`, `tali sweep --staged`), the generated config file is **`config.tali`**
(custom `.tali` extension, YAML content — `tali init` writes it), and the package's
`package.json` maps `"bin": { "tali": "./bin/tali.js" }` so the daily command is `tali`
while the package stays `@talicode/core`. Two other parts change: the **Node "Programmatic
Usage"** section
(there is no in-process JS API — the core is a Rust binary; drop it for the MVP, or
reframe as "spawn the `tali` CLI from Node via `child_process`"), and the
**contributing note** ("npm workspaces for the Core CLI…") which becomes "Rust cargo
workspace for the core + npm packaging for distribution." Also refresh the **intro** to
the canonical AI-Slop-Gatekeeper positioning (see Context) and the **"Why TaliCode?"**
section to the refined Tagalog etymology: **Tali / Talian** ("to tie / harness" — the
mechanical layer: the execution harness that ties OpenAI/Gemini/Claude into one pipeline),
**Talikod** ("going back / behind" — the architectural layer: the ultimate backend, a
secure safety net operating behind the scenes), and **Talikod** ("to turn your back on" —
the philosophical layer: a deliberate stance against AI slop, turning your back on
unverified code, structural bloat, and hallucinated vulnerabilities).

**Logo:** the logo already exists at the repo root as `tali-logo.png` (139 KB). Phase 0
moves it into `assets/tali-logo.png` (keeping it tracked), and the README rewrite (phase 7)
adds it to the header (centered image at the top, above the tagline).

**Brand casing (everywhere):** write the product name as **TaliCode** (capital C) in the
README, logo, and all documentation — the capital C splits *Tali + Code* so readers don't
misread it as "Tail." Technical identifiers stay **lowercase** by convention: the
`tali` command/binary, `config.tali`, the `.talicode/` dir, the `@talicode/*` npm
packages, and the `talicode-cli` / `talicode-core` crates.

## Key components

### Config schema (`config.rs`)
**Config file: `config.tali`** — a custom extension (like `.tf` for Terraform) that
signals "this is a TaliCode harness," **written in standard YAML under the hood**. The
core resolves `config.tali` in the repo root and parses it with `serde_yaml` (the
extension is cosmetic; the bytes are YAML). `tali init` generates it. Mirror the README's
`config.tali` with serde `Deserialize` structs:
`version`, `name`, `agents` (map of `{ provider, model, role, effort? }` — `effort`
defaults to `"medium"` for the auditor via `#[serde(default)]`), and `execution_flow`
(steps with `agent`, `target` glob, optional `anti_slop.mode`). Plus a top-level
`skills:` list selecting which rule packs the Auditor loads. Unknown-but-valid fields
are preserved (`#[serde(flatten)]` extras / non-`deny_unknown_fields`), not rejected;
a `validate()` pass checks required references. For the MVP only the `auditor` agent
and the `slop_sweep` step are exercised.

### Git staged reader (`git.rs`)
- `git diff --cached --name-only --diff-filter=ACM` (via `std::process::Command`) for `--staged`.
- Without `--staged`, expand the step's `target` glob against the working tree (`globset`).
- Read file contents; skip binary/oversized files (cap, e.g. 200 KB) and log what was
  skipped — no silent truncation.

### Provider seam (`provider/`)
A `Provider` **trait** (async) with one method that takes messages + a JSON-schema tool
definition and returns validated structured output **plus the response's `Usage`**
(input/output/cache tokens — see "Token-spend reporting"). The `anthropic.rs` impl calls the
**Messages API over `reqwest`** (there is no official Anthropic Rust SDK; raw HTTPS is
the supported path for languages without one) and uses Claude **tool-use to force the
findings schema** (this is what makes detection reliable vs free-text parsing). API key
from `ANTHROPIC_API_KEY`; fail with a clear message if missing. The trait is the test
seam — a fake impl backs every unit test. Default Auditor model:
**`claude-sonnet-5` at `output_config.effort: "medium"`** — Sonnet 5's semantic
judgment serves the low-false-positive goal, and medium effort balances that judgment
against the cost/latency of a per-commit sweep (raise to `high`/`xhigh` for a deeper
audit, drop to `low` for speed). Both model and effort are set in `config.tali` and
passed through the provider seam — do NOT hardcode the README's dated `claude-3-5-*`
ids in code. (`effort` is a `claude-sonnet-5` capability via `output_config`.)

**How "TaliCode invokes skills as Claude" is implemented:** the host loads a local
`SKILL.md` + `rules.yaml` and injects them into a normal `@anthropic-ai/sdk` Messages
call (skill guidance in the system prompt, rules composed into the tool schema).
This is the roll-your-own approach — it does NOT use Anthropic's hosted Agent-Skills
container feature (that runs skills in an Anthropic sandbox and targets doc-gen, not
local audit packs), and does NOT need the Claude Agent SDK. If a future phase makes
TaliCode *edit* files agentically (the healing roadmap), `@anthropic-ai/claude-agent-sdk`
becomes the natural upgrade; the MVP does not need it.

### Auditor agent (`auditor.rs`)
Input: file path, file content (line-numbered), and the agent's `role`/rules from
config. Output: `Vec<Finding>` where `Finding { file, line, severity, rule, message }`.
Prompt instructs Claude to report only concrete, line-anchored violations (hallucinated/
unverified imports, dead boilerplate, obvious type/security issues) and to prefer
silence over speculation — false positives kill adoption.

### Reporting (`report.rs`)
Group findings by file, print `file:line severity rule — message` (clickable).
Process exit code non-zero when any finding at/above a threshold exists (so the future
pre-commit hook can gate on the same exit code). `--json` flag for machine output.

### Token-spend reporting (`usage.rs`)
The Messages API returns a `usage` block on every response
(`input_tokens`, `output_tokens`, `cache_read_input_tokens`,
`cache_creation_input_tokens`). The `Provider` trait surfaces this alongside the
findings, so the auditor/host thread a `Usage` up from each Auditor call.
- **Per-execution:** a sweep sums usage across all its Auditor calls and prints a footer
  — `tokens: in <n> / out <n> (cached <n>) · est. $<x>`. Cost is an **estimate** from a
  small per-model price table (Sonnet 5 rates), clearly labeled and overridable in
  `config.tali`; tokens are the source of truth, cost is convenience.
- **Daily:** each run appends `{ date, model, input, output, cache_read, cache_creation,
  command }` to a local ledger at `.talicode/usage.jsonl` (repo-local; `init` adds
  `.talicode/` to `.gitignore`). The footer also prints **today's cumulative total**
  (local-date bucketed). `tali usage` rolls the ledger up by day.
- Failures to write the ledger are non-fatal (warn, continue) — usage accounting must
  never block a sweep.

### Watch mode (`watch.rs`)
Uses a filesystem watcher (`notify` crate) on the working directory plus `git` for
staged-change detection, with a debounce window so a burst of saves triggers one sweep.
Each sweep reuses the exact `sweep` path (host → auditor → findings), prints via the
same reporter, and records usage to the ledger — same as a one-shot. Scoped to the
current folder/repo only; no cross-window coordination (that's the extension's job, in
the roadmap).

### npm distribution (`npm/`)
Follows the esbuild/Rolldown pattern:
- **Platform packages** (`talicode-<os>-<arch>`) each contain one prebuilt `tali` binary
  and a `package.json` gated by `os` + `cpu` fields so npm installs only the matching one.
- **Wrapper package** (`@talicode/core`, what users install) exposes the **`tali` command**
  via `"bin": { "tali": "./bin/tali.js" }` and lists all platform packages under
  `optionalDependencies`; its `bin/tali.js` launcher resolves the installed platform
  package and `execFileSync`/`spawn`s the native `tali` binary, forwarding args + exit
  code. **Brand name stays TaliCode / `@talicode/core`; the daily command is `tali`**
  (short, like `git`/`npm`; `tali sweep` literally reads "harness sweep").
- **Release**: a CI target matrix runs `cargo build --release` per triple, drops each
  binary into its platform package, and publishes the platform packages + wrapper
  together (versions locked in lockstep). Not built in the MVP verification, but the
  package layout and launcher are (so a local `npm pack` + install smoke-tests the path).

### Commands
- `init`: refuse to overwrite an existing `config.tali`; otherwise write the
  README's example config verbatim (keeps README truthful).
- `sweep [--staged] [--skill <name>] [--json]`: resolves the selected skills via the
  host, invokes them over the target files, renders findings.
- `heal`: runs `sweep`, then prints that healing is not yet enabled and points to
  `docs/roadmaps/ROADMAP-HEAL.md`. Command exists so the CLI surface is stable.
- `skills`: lists **user-defined (repo `skills/`) skills only** — name + description.
  Bundled `code-*` defaults are intentionally hidden (they're the built-in harness,
  not the user's authored skills). `--all` includes bundled, labeled by source.
- `usage`: prints token spend — **today's running total** and the last N days from the
  ledger (`--json` for machine output). Every `sweep`/`heal` also prints a one-line
  per-execution footer and updates the ledger, so `usage` is the aggregate view.
- `watch [--json]`: **monitor mode for the current folder/repo** (the directory you run
  it in — i.e. the current VS Code window's workspace). Watches file saves and staged
  changes, runs a sweep on change, prints findings (human by default, NDJSON with
  `--json`). Long-running until `Ctrl-C`; debounced so rapid saves don't fan out into
  concurrent sweeps. Standalone in the MVP — you run it in the integrated terminal; the
  future VS Code extension (roadmap) drives this same mode for the squiggle experience.

## Default skills (TaliCode is the skill host)

There is **one** skill format, and **TaliCode itself invokes skills — acting as
Claude**. A skill is a folder with a `SKILL.md` (frontmatter `name` + `description`,
plus audit guidance the Auditor reads) and a `rules.yaml` (the concrete rule list).
TaliCode's **host runtime** (`host/` in `talicode-core`) discovers these folders,
parses them, and runs a selected skill by feeding its guidance + rules to the Auditor
over the provider — TaliCode plays the role a Claude Code session would.

- `host/skill.rs` — serde-`Deserialize` `Skill` model; parses `SKILL.md` frontmatter
  (`serde_yaml`) + loads `rules.yaml`. A skill is either a **lens** (has `rules.yaml`)
  or an **orchestrator** (its `SKILL.md` declares a `runs:` list of other skill names
  and carries no rules of its own).
- `host/discover.rs` — resolves skills from two roots: the **embedded defaults**
  (compiled in via `rust-embed`) and the **repo-root `skills/`** (read from disk);
  repo skills override embedded ones with the same `name`.
- `host/invoke.rs` — given selected skill names + target files, expands any
  orchestrator skills to their `runs:` lenses, composes the resolved lenses'
  guidance/rules into the Auditor's prompt + tool schema, and returns `Vec<Finding>`
  tagged with the originating skill/rule id (plus, for an orchestrator run, one
  aggregated verdict).

The bundled defaults are the **`code-*` coding harness** — 21 judgment lenses plus
the `code-review` orchestrator (22 skill folders, each a `SKILL.md` + `rules.yaml`):
- **code-think-twice** — understand the requirement/system before writing.
- **code-kiss** — simplicity / readability.
- **code-yagni** — no speculative or over-engineered code.
- **code-dry** — single source of truth, no duplication.
- **code-early-return** — guard clauses, flatten nesting.
- **code-clear-exit** *(new, default)* — **"clear single exit" that composes with
  `code-early-return`, not contradicts it.** Multiple `return` statements are allowed as
  long as every exit is clear and non-branching: guard-clause early returns at the top of
  a function (invalid state → return immediately) are encouraged, exactly as
  `code-early-return` wants. What it forbids is returns **buried inside deep or nested
  branches** (mid-loop, inside the third arm of a nested `if`) that make control-flow /
  path analysis hard. The two lenses reinforce each other — guard clauses satisfy both.
- **code-no-nested-loop** — no O(n²) loop-in-loop / arrow-shaped code.
- **code-bounded-loops** *(new, default)* — every iterative process has a **statically
  verifiable, fixed upper bound** before execution. This prevents infinite loops, limits
  runaway memory, and makes execution time modelable. Loops over a known-length
  collection or a constant bound pass; a `while (true)` / condition-only loop with no
  provable termination bound flags.
- **code-bounded-recursion** *(new, default)* — recursion is **allowed**, but must have a
  clear, finite terminating base case that provably stops it (like a `for` loop with a
  definite length) — no unbounded self-calls that can stack-overflow. Flags recursion
  with no reachable base case or no argument progressing toward it.
- **code-deterministic-concurrency** *(new, default)* — concurrent code must produce the
  **same output for the same input regardless of thread/task scheduling**: no data races,
  no deadlocks, no time-dependent bugs. Using `sleep`/`setTimeout`/`delay` **as a
  synchronization mechanism** is forbidden (it introduces race conditions); real
  synchronization primitives are required instead.
- **code-composition** — favor composition over inheritance.
- **code-solid** — SOLID OO design principles.
- **code-smells** — introduced design smells.
- **code-nitpick** — senior-engineer review beyond syntax.
- **code-rules** — NASA Power-of-10 safety rules (language-agnostic port).
- **code-magic-strings** — no hardcoded meaningful string literals.
- **code-magic-numbers** *(new)* — no unexplained numeric literals: a raw number that
  carries meaning (e.g. `86400`, `0.05`, a threshold) should be a named constant —
  obscured intent, scattered updates, typo risk, and no single source of truth are the
  costs. Its `rules.yaml` encodes the **"not magic" exceptions** so it stays low-noise:
  identity/degenerate values (`0`, `1`, `-1`), math that defines its own context
  (`n % 2`, `/ 2`), and standard loop scaffolding (`for i = 0; i < arr.length`) are not
  flagged. Complements `code-magic-strings` (string siblings) and overlaps `code-dry`
  (repeated literal = duplication) — findings are de-duplicated by file+line at the
  aggregation step.
- **code-no-keys** — no hardcoded secrets/keys/tokens.
- **code-no-credentials** — no embedded logins / plaintext passwords.
- **code-traceability** *(new, default)* — best-effort requirement/test traceability,
  inspired by DO-178C's "every line ↔ a requirement ↔ a test" but honest about scope.
  Full traceability is a **process standard** that needs requirement/test linkage beyond
  a per-file static sweep, so this lens flags what it *can* see: public surface with no
  covering test, exported/undocumented APIs, and code with no discernible reason to
  exist (ties into `code-smells`/dead-code). It is explicitly **best-effort** and its
  `SKILL.md` states it does **not** assert DO-178C certification — no overclaiming.
- **code-aviation** *(new, strict profile)* — the DO-178C safety-critical standard for
  airborne software: absolute predictability over developer convenience — eliminate
  undefined behavior, guarantee deterministic execution, keep software auditable.
  It is the **strict superset**: for the concepts that also exist as default lenses it
  applies the tighter DO-178C form — **no recursion at all** (stricter than
  `code-bounded-recursion`), **compile-time-constant loop bounds** (stricter than
  `code-bounded-loops`), and **locked priority-based scheduling** (stricter than
  `code-deterministic-concurrency`) — plus the rules unique to it: **no dynamic
  allocation** after init (no `malloc`/`free`/`new`), **strong typing** (no implicit
  conversions/truncation), and **no dead/unreachable code**. Its `SKILL.md` states the
  DO-178C philosophy (traceability, determinism, auditability) so the Auditor flags in
  that spirit, not just literal patterns. Overlap with the default lenses is
  de-duplicated by file+line at aggregation.

  `code-aviation` is an **opt-in strict profile** (its rules are too strict for general
  code), so it is **not** in the default `code-review` `runs:` list; a repo enables it
  deliberately for safety-critical code. The **clear-single-exit** and **traceability**
  dimensions are now their own default lenses (`code-clear-exit`, `code-traceability`),
  extracted from this profile — `code-aviation` relies on them (in strict form) rather
  than restating them.

**Language-agnostic by design:** each `rules.yaml` keeps its judgment lens without
tying to one language, with an optional `language:` hint per pack for
language-specific rules (e.g. `code-rules` can carry a language-specific safety set).

**`code-review` is a bundled orchestrator skill (`skills/code-review/`).** Unlike the
lenses, its `rules.yaml` carries no detection rules of its own; instead its `SKILL.md`
declares a `runs:` list naming the lenses to execute — **all bundled lenses except the
opt-in `code-aviation` strict profile**, so `code-clear-exit`, `code-bounded-loops`,
`code-bounded-recursion`, `code-deterministic-concurrency`, and `code-traceability` run by
default. The host expands it — running each named
lens, aggregating findings, and rendering one verdict. `tali sweep` with no
`--skill` invokes `code-review` by default, so a bare sweep runs the default harness.
This keeps the orchestrator visible and editable as a skill folder (add/remove a lens by
editing its `runs:` list) rather than hiding it in engine code. The host recognizes this
"orchestrator" skill shape (a skill that lists other skills) generically, so users can
author their own orchestrators too.

`config.tali` selects skills via a `skills:` list (default: the bundled `code-*`
harness via `code-review`); `sweep --skill <name>` overrides per-run. `tali init`
writes the `skills:` block and can scaffold a starter `skills/` folder in the repo.
`tali skills` lists only the user's own repo skills (bundled defaults hidden;
`--all` reveals them) — so authoring a new skill is just dropping a folder in
`skills/` and seeing it appear, no recompile.

## Roadmap docs (written, not built)

- **`docs/roadmaps/ROADMAP-HEAL.md`** — the Surgeon agent; two modes: **Heal Preview**
  (propose fixes, render a unified diff, require explicit approval before writing)
  and **Auto-Heal** (write in place). Covers the Auditor→Surgeon handoff format,
  the mandatory diff-preview UX, `--heal` wiring, retries/`max_retries`, and the
  trust/safety rationale for never silently overwriting a commit.
- **`docs/roadmaps/ROADMAP-DEPLOYMENT.md`** — the **pre-commit git hook** (wraps
  `tali sweep --staged`, gates on exit code) and the **VS Code extension**. The
  extension is a thin TypeScript client over the CLI's `tali watch` mode (which the
  MVP ships): it **auto-starts on window open** (`activationEvents: ["onStartupFinished"]`,
  no command to run) and, because each VS Code window is its own extension host, opening
  N windows monitors N folders/repos independently. On activate it resolves the `tali`
  binary (bundled npm platform package, then `PATH`), starts `tali watch` for the
  workspace folder, and maps findings to VS Code diagnostics (squiggles); Quick-Fix /
  "Heal" ties into the healing roadmap. It also **registers the `.tali` config extension
  as YAML** via `contributes.languages` (`{ id: "yaml", extensions: [".tali"], aliases:
  ["TaliCode Configuration"] }`) so `config.tali` gets full YAML syntax highlighting /
  indentation instead of plain-text, plus a `contributes.icons` file icon for `.tali`.
  The **only user requirement is installing the extension**; missing binary / API key
  surfaces a dismissible notice, never a hard fail. These are local, MIT dev-tool
  integrations — the two commercial server components get their own roadmap files below.
- **`docs/roadmaps/ROADMAP-TALICLOUD.md`** — **TaliCloud**, the managed cloud platform.
  Managed LLM routing, centralized API-key management, high-concurrency rate limits, and
  SOC2-ready audit logging — the hosted "background CTO across the org" from the README.
  **Proprietary commercial software under a Commercial License** (its commercial-license
  text lives here, not in the MVP). Separate proprietary repo — this doc sketches its
  scope and how the MIT CLI/engine would talk to it; no code in this repo.
- **`docs/roadmaps/ROADMAP-TALIAGENTICSERVER.md`** — **TaliAgenticServer**, the always-on
  webhook / agentic daemon (Pillar 3). GitHub PR Gatekeeper (intercept PRs, run the audit
  loop, block/auto-push fixes), Jira scaffolder (provision compliant feature branches from
  tickets), CI/CD self-healing, and team-wide policy enforcement. **This is where the
  commercial-license text lives** (moved out of the MVP): the centralized EC2 Webhook
  Server providing the PR Gatekeeper, Jira scaffolding, team-wide policy enforcement, and
  SOC2-ready audit logging is **proprietary commercial software** under a Commercial
  License. Separate proprietary repo — this doc sketches the webhook surface and its reuse
  of the engine; no code in this repo.

## Verification

1. `cargo build --release` — compiles clean; produces a single `tali` binary.
   `cargo clippy -- -D warnings` and `cargo fmt --check` are green.
1b. **npm path:** stage the current-platform binary into its `npm/platform/*` package,
   `npm pack` the wrapper + platform package, install the tarballs into a scratch dir,
   and confirm `npx tali --version` execs the native binary and forwards the exit
   code. (Full multi-platform publish is out of scope for the MVP; this smoke-tests the
   launcher/resolution path on one platform.)
2. `tali init` in a scratch dir → generates `config.tali`; re-running refuses to
   overwrite.
3. In a scratch git repo, stage a file with obvious slop (e.g. an import of a
   nonexistent package + an unused boilerplate type), set `ANTHROPIC_API_KEY`, run
   `tali sweep --staged` → flags it at the right line and exits non-zero. Stage a
   clean file → no findings, exit 0. Each sweep prints a token footer (in/out/cached +
   est. cost) and today's running total; `.talicode/usage.jsonl` gains a row. Run
   `tali usage` → shows today's total and daily history.
3c. `tali watch` in a scratch repo → stays running; editing + saving a file triggers
   a single (debounced) sweep whose findings print; `Ctrl-C` exits cleanly.
4. `tali heal` → runs detection, prints the roadmap notice.
   Also: a bare `tali sweep --staged` invokes the `code-review` orchestrator,
   which runs every bundled lens and renders one aggregated verdict; `tali sweep
   --staged --skill code-no-keys` runs only that lens. `tali skills` on a fresh
   repo prints nothing (no user skills yet); `tali skills --all` shows the 22
   bundled defaults; dropping a `skills/my-rule/` folder then makes `my-rule` appear
   in plain `tali skills`. A guard-clause early return passes both `code-clear-exit`
   and `code-early-return`; a return buried in nested branches flags `code-clear-exit`.
   Enabling `code-aviation` in `config.tali` additionally runs the strict DO-178C
   guidelines on top.
5. `cargo test` + `cargo llvm-cov` — green with the coverage gate passing. **Every
   module has tests**, `Provider` faked via the trait:
   - `config.rs` — valid config deserializes; each invalid shape rejected by `validate()`;
     finds/parses `config.tali`; clear error when missing.
   - `git.rs` — staged-file listing + binary/oversize skip.
   - `provider/anthropic.rs` — maps tool-use response → structured output; missing-API-key
     error path (HTTP faked with `wiremock`/`mockito`).
   - `provider/mod.rs` — registry resolves `provider` name, errors on unknown.
   - `auditor.rs` — builds prompt/tool schema, returns `Vec<Finding>` from a fake provider,
     empty-findings path.
   - `host/skill.rs` — parses SKILL.md frontmatter + rules.yaml, rejects malformed.
   - `host/discover.rs` — finds embedded + repo skills; repo overrides embedded on name;
     errors on unknown selected skill.
   - `host/invoke.rs` — composes selected skills into the Auditor call (fake provider),
     tags findings with skill/rule id; expands the `code-review` orchestrator's `runs:`
     list to real bundled lenses; `code-clear-exit` and `code-early-return` compose (a
     guard-clause early return satisfies both — no exclusion; a return buried in nested
     branches flags `code-clear-exit`).
   - each bundled `code-*` default is validated by a data test that loads it through
     `host/skill.rs` (all 22 covered).
   - `report.rs` — rendering + exit-code threshold logic.
   - `usage.rs` — parses `Usage` from a provider response; sums per-execution; appends
     to and rolls up the ledger by local date; cost estimate from the price table;
     ledger-write failure is non-fatal.
   - `watch.rs` — debounce collapses a burst of change events into one sweep; a change
     triggers exactly one sweep (fake watcher + fake provider); scopes to CWD/repo.
   - `commands/*.rs` — `init` writes config / refuses overwrite + adds `.talicode/` to
     `.gitignore`; `sweep`/`heal` wiring against faked git + provider (`heal` prints the
     roadmap notice); `skills` lists only repo skills by default, `--all` includes bundled
     with source labels; `usage` renders today + daily history from a seeded ledger;
     `watch` wiring against a fake watcher (change → one debounced sweep).
   - `main.rs` — clap dispatch.
   One live integration test in `crates/talicode-core/tests/`, `#[ignore]`d unless
   `ANTHROPIC_API_KEY` is set.

## Out of scope (this pass)

Healing implementation, pre-commit hook, VS Code extension, TaliCloud, TaliAgenticServer, Jira/CI
integrations, multi-provider (OpenAI/Gemini) wiring, the full multi-platform CI build
matrix and npm publish (only the package layout + single-platform launcher smoke-test
are in the MVP).

## Execution methodology (docs → issues → traceable implementation)

The build itself follows `code-traceability`: **every line of code traces to a GitHub
issue, which traces to a phase doc, which traces to this MVP plan.**

**Prerequisites (first, one-time):**
- Wire the local working copy to `origin` (`https://github.com/mackignacio/talicode`) —
  clone fresh or `git init` + `git remote add origin`; reconcile the existing `README.md`.
- Confirm `gh` CLI is authenticated (`gh auth status`) and can create issues on the repo.

**Step 1 — Write the plan to the repo.** Save this whole document to
`docs/plans/MVP.md` (the source of truth).

**Step 2 — Phase docs.** Split the build into phases, each written to its own file under
`docs/plans/`:
- `phase-0-scaffold.md` — repo wiring, Cargo workspace, `.gitignore`, `rust-toolchain`,
  MIT `LICENSE`, docs skeleton, CI (`fmt`/`clippy`/`test`) skeleton.
- `phase-1-cli-config.md` — `talicode-cli` + clap + `tali` bin dispatch; `config.rs`
  (`config.tali` serde/validate); `tali init`.
- `phase-2-provider-auditor.md` — `Provider` trait + fake; `anthropic.rs` (Messages API,
  forced tool-use schema, `Usage`); `auditor.rs`.
- `phase-3-skill-host.md` — `host/skill.rs`, `discover.rs` (rust-embed), `invoke.rs`
  (orchestrator expansion); authoring the 22 bundled `code-*` skills.
- `phase-4-sweep-report.md` — `git.rs`, `report.rs`, `sweep.rs`, `heal.rs`.
- `phase-5-usage-watch.md` — `usage.rs` + `tali usage`; `watch.rs` + `tali watch`;
  `tali skills`.
- `phase-6-npm-dist.md` — `@talicode/core` wrapper (`bin: tali`, `bin/tali.js` launcher),
  platform packages, single-platform smoke test.
- `phase-7-docs-license.md` — roadmap docs (HEAL, DEPLOYMENT, TALICLOUD, TALIAGENTICSERVER),
  README rewrite (positioning / Why TaliCode / Quick Start with `tali` + `config.tali` /
  MIT), and `docs/TRACEABILITY.md`.

**Step 3 — Atomic user stories.** Each phase doc lists its work as **atomic user stories**
(one testable unit each — e.g. "US-1.2: `config.rs` parses & validates `config.tali`",
"US-3.n: author the `code-<name>` skill"). Atomic = independently implementable, testable,
and revertible; one story ≈ one PR.

**Step 4 — GitHub issues.** Create one GitHub issue per user story via `gh issue create`,
labeled by phase, body linking back to its phase doc + the MVP plan. Do this one by one so
each story has a stable issue number.

**Step 5 — Implement, issue by issue.** For each issue in order: branch
(`feat/<issue#>-slug`), implement with its module tests, run the gate
(`cargo fmt`/`clippy`/`test`/`llvm-cov`), commit referencing the issue (`… (#<issue>)`),
open a PR that `Closes #<issue>`. **Traceability is enforced**: every source file carries
an SPDX header and an issue reference (module-level `//! Implements #<issue>`), and
`docs/TRACEABILITY.md` maps module → issue → phase → MVP. The bundled `code-traceability`
lens then has real linkage to check against.

> Note: this is a large, multi-turn effort (dozens of atomic issues). After approval I'll
> run the prerequisites + Steps 1–4 first (all the planning artifacts and issues), then
> implement Step 5 issue-by-issue.
