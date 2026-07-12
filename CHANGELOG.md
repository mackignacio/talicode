<!-- SPDX-License-Identifier: MIT -->
# Changelog

All notable changes to TaliCode are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.1] — 2026-07-12

First published release — the detect-only MVP of the AI Slop Gatekeeper.

### Added
- **`tali` CLI** — a single self-contained Rust binary that hosts Claude with bundled skill
  lenses to catch AI-generated code problems before commit. Commands: `init`, `sweep`, `heal`,
  `watch`, `skills`, `usage`, `memory`, `map`.
- **22 bundled skill lenses** (`code-review`, `code-rules`, `code-smells`, `code-no-keys`,
  `code-no-credentials`, `code-magic-strings`/`-numbers`, `code-dry`, `code-solid`, `code-kiss`,
  `code-yagni`, `code-early-return`, `code-composition`, `code-no-nested-loop`,
  `code-bounded-loops`/`-recursion`, `code-clear-exit`, `code-deterministic-concurrency`,
  `code-aviation`, `code-think-twice`, `code-traceability`, `code-nitpick`).
- **Five-type long-term memory** — working (context assembly + budgeted compression), semantic
  (durable committed facts), procedural (native on-demand skill search, no resident index),
  episodic (learnings/mistakes/experiences/summaries with auto-promotion of recurring experiences
  into skills), and architectural (a codebase map). All native, zero new dependencies, fully
  defaulted so existing configs are unaffected.
- **Anthropic provider seam**, auditor agent, git staged reader, reporting, and a token-usage
  ledger with daily roll-up.
- **npm distribution** — installable as the unscoped `talicode` package; a thin `bin/tali.js`
  launcher whose postinstall step (`scripts/install.js`) downloads the matching native `tali` binary
  for the host (darwin arm64/x64, linux x64/arm64, win32 x64) from the versioned GitHub Release.
- **`@talicode.ai/{core,cli,agent,skill,memory}`** — reserved module packages under the org scope.
- **Release tooling** — `scripts/npm-smoke.sh` (offline host-only launcher check),
  `scripts/publish-modules.sh` (the reserved scope stubs), and a GitHub Actions release workflow that
  cross-compiles all five targets on a `v*` tag, attaches them to the GitHub Release, and publishes
  `talicode` to npm.

[0.0.1]: https://github.com/mackignacio/talicode/releases/tag/v0.0.1
