# Roadmap — TaliCode Test (universal test orchestration)

> Deferred design, part of the TaliCode MVP (see [../plans/MVP.md](../plans/MVP.md)). A new module —
> **designed here, not yet built.** TaliCode Test extends the gatekeeper beyond static / AI-slop
> detection to **running (and later generating) the right test suite for whatever the project is
> built with** — Python to TypeScript to Terraform — and folding the result into the same gate.

## The idea

AI agents write code fast; "did it pass the tests?" is the other half of the trust the gatekeeper
should own. Every area — frontend, backend, mobile, infrastructure — has its own test frameworks and
languages. **TaliCode Test detects the stack and drives its native test suite**, normalizes the
result into TaliCode's findings/report model, and gates the commit on it — one command, one verdict,
across the whole polyglot repo.

## Delivered via the `tali` CLI

TaliCode Test is **not a separate binary or product** — it ships inside the existing `tali` CLI as a
first-class subcommand, exactly like `sweep`, `watch`, and `map`:

```
tali test [--changed | --all] [--adapter <name>] [--list] [--json]
```

- `--changed` (default) runs only the adapters relevant to the changed files; `--all` runs every
  detected suite.
- `--adapter <name>` restricts to one adapter; `--list` shows the detected/available adapters.
- `--json` emits the normalized findings; the exit code follows the same gate contract as `tali
  sweep`, so `tali test` slots straight into the pre-commit hook and CI.

Implementation-wise it lives in the same Rust workspace as the rest of the CLI (a `talicode-test`
crate alongside `talicode-core`/`agent`/`skills`/`memory`, wired into `talicode-cli`).

## Principles

- **Delegate, never reinvent.** TaliCode Test does not implement a test framework. It detects and
  runs the project's *existing* runner (pytest, vitest, `go test`, `terraform validate`, XCTest, …)
  and parses its output. The frameworks stay the source of truth for what "passing" means.
- **One verdict, one gate.** Test outcomes normalize into the same `{ file, line, severity, rule,
  message }` report and **exit-code contract** as `tali sweep`, so `tali test` gates a commit exactly
  the way a sweep does — and the pre-commit hook can run both.
- **Extensible via adapters, not core changes.** Just as skills are pluggable review lenses,
  **test adapters** are pluggable per-stack runners. Adding a language or framework means adding an
  adapter, bundled or repo-authored — no recompile of the core.

## Test adapters — the extensibility model

An **adapter** declares three things:

1. **Detection** — the file markers that identify the stack (`pyproject.toml`, `package.json` test
   script, `go.mod`, `Cargo.toml`, `*.tf`, `build.gradle`, `pubspec.yaml`, …).
2. **Run** — the command(s) to execute the suite (with sensible defaults, overridable in
   `config.tali`).
3. **Parse** — a normalizer that turns the framework's output (JUnit XML, TAP, JSON reporters, exit
   codes) into TaliCode findings, mapping each failing test back to a `file:line` where possible.

Bundled adapter targets (illustrative — the point is breadth):

- **Backend / general languages** — Python (pytest, unittest), Node/TS (`node --test`, Jest, Vitest),
  Go (`go test`), Rust (`cargo test` / nextest), Java·Kotlin (JUnit via Maven/Gradle), Ruby
  (RSpec/minitest), PHP (PHPUnit/Pest), .NET (`dotnet test`), Elixir (ExUnit).
- **Frontend** — unit (Jest/Vitest), component (Testing Library), end-to-end (Playwright/Cypress),
  type-check as a test (`tsc --noEmit`).
- **Mobile** — iOS (XCTest / `swift test`), Android (JUnit/Espresso via Gradle), Flutter
  (`flutter test`), React Native (Jest + Detox).
- **Infrastructure as Code** — Terraform (`validate`/`plan`, tflint, Terratest, checkov), Kubernetes
  (kubeconform, conftest/OPA), Docker (hadolint), shell (bats).

### Reference adapter — Python (strict quality gate)

The first concrete adapter, and the template for the rest. "Test" here means the **full quality
gate**, not just unit tests — the adapter runs each step in order and passes only when **every one is
green**:

1. `ruff format --check` — formatting is clean (no diff).
2. `ruff check` — lints clean.
3. `flake8` — clean.
4. `pylint --fail-under=10.0` — must score a perfect **10.00/10** (zero warnings).
5. `pytest` — the test suite passes.

Any non-zero exit — or a pylint score below `10.00` — fails the gate, and each failure normalizes
into a `file:line` finding like any other adapter's output. "Green" therefore means **formatted +
lint-clean + a perfect pylint score + passing tests** — the same bar TaliCode holds its own code to.
The step commands and thresholds (e.g. the pylint floor) are overridable in the `test:` block of
`config.tali`, so a project can relax or tighten them.

## Stack detection — automatic, from the code

**The user never picks a test type; TaliCode figures it out from the file itself.** Each changed file
is classified and routed to the matching adapter automatically, using layered signals (cheapest
first):

1. **Extension** — `.py` → Python, `.ts`/`.tsx` → TypeScript, `.go` → Go, `.rs` → Rust, `.tf` →
   Terraform, `.swift` → iOS, `.kt` → Android/Kotlin, `.dart` → Flutter, and so on.
2. **Content signatures** — shebangs, imports, and framework markers disambiguate when the extension
   isn't enough: `import pytest` vs. `unittest`, `from playwright` vs. a Vitest `describe`, a React
   component vs. a plain TS module, `provider "aws"` in HCL.
3. **Project manifests + architectural map** — `pyproject.toml`, `package.json` test scripts,
   `go.mod`, `Cargo.toml`, `build.gradle`, `pubspec.yaml`, resolved against the
   [architectural memory](./ROADMAP-MEMORY.md) map to know which module/suite a file belongs to.

Detection is the adapter's own `detect` step, so it stays extensible — a new adapter teaches TaliCode
a new signature set, no core change. In a monorepo this means a single `tali test` run
auto-dispatches only the relevant suites for what changed: the Python service's pytest, the web app's
Vitest, the `infra/` Terraform checks — each file to its right runner, with no manual selection. An
explicit `--adapter <name>` override is available for the rare case detection guesses wrong.

## Phased plan

### Phase T1 — Run & normalize
- `tali test [--changed | --all] [--json]` — detect the stack(s), run the matching adapter(s),
  normalize pass/fail (each failure → a `file:line` finding), and **exit non-zero** on failure. Full
  gate parity with `tali sweep`.

### Phase T2 — Coverage & gating
- Parse coverage from the runner (coverage.py, c8/istanbul, `go -cover`, cargo-llvm-cov, JaCoCo),
  enforce thresholds from `config.tali`, and surface **uncovered changed lines** as findings.

### Phase T3 — Test generation & healing (agentic)
- The agent **authors missing tests** for changed code, guided by per-language **test-writing skills**
  (framework idioms, naming, arrange/act/assert conventions). Generated tests are proposed through the
  **Heal** flow (diff + approve) — never written silently. Flaky tests and past failures are
  remembered in **episodic memory**. Depends on the agentic tool-loop upgrade in
  [ROADMAP-HEAL](./ROADMAP-HEAL.md).

### Phase T4 — Selection & speed
- **Test-impact analysis** via the architectural dependency graph to run only tests affected by the
  change; parallel execution and result caching for large suites.

## Config

A `test:` block in `config.tali`: which adapters are enabled, per-adapter command overrides, coverage
thresholds, the changed-vs-all default, and a per-adapter **model** (ties into the per-component model
routing in [ROADMAP-VSCODE](./ROADMAP-VSCODE.md) — a cheap model to triage failures, a stronger one to
generate tests).

## Relationship to the rest of TaliCode

- **The gate** — `tali test` feeds the same findings/exit-code contract as `tali sweep`; the
  pre-commit hook ([ROADMAP-DEPLOYMENT](./ROADMAP-DEPLOYMENT.md)) can gate on both.
- **Skills** (procedural memory) — test-writing conventions per language are skills, searchable and
  overridable like the `code-*` lenses.
- **Memory** — **architectural** memory drives stack detection; **episodic** memory remembers flaky
  tests and recurring failures.
- **Agent + Heal** — test generation rides the agentic loop and the Heal diff+approve flow.
- **Editor** — runs, results, and coverage surface through the [VS Code extension](./ROADMAP-VSCODE.md).

## Out of scope / open questions

**Out of scope**
- Being a test framework or a CI system — TaliCode Test orchestrates local runners; server-side/CI
  test gating lives with the commercial products
  ([TaliCloud](./ROADMAP-TALICLOUD.md), [TaliAgenticServer](./ROADMAP-TALIAGENTICSERVER.md)).

**Open questions**
- **Toolchain presence** — TaliCode Test assumes the project's runtimes/runners are installed (it
  drives them). Should it detect-and-guide when they're missing, or offer containerized toolchains
  later?
- **Sandboxing** — running a repo's test suite executes arbitrary project code; how to run it safely
  (and whether that's in scope vs. the user's responsibility).
- **Failure → line mapping** — how reliably each framework's output maps to line-accurate findings,
  and the fallback when it only yields a test name.
- **Monorepo resolution** — how adapters compose when multiple stacks share a tree and changes span
  several.
- **Flaky-test policy** — detection (retry N times?), and how a known-flaky test affects the gate.
