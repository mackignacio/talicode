# Phase 6 — npm Distribution

> Part of the [TaliCode MVP](./MVP.md). Every story below becomes a GitHub issue; every commit references its issue.

## Goal

Package the compiled `tali` binary for `npm install -g @talicode/core` (the esbuild/Rolldown model): platform packages + a thin wrapper with a launcher. Full multi-platform publish is out of scope; the MVP verifies the layout + launcher on one platform.

## User Stories

### US-6.1 — Wrapper package + launcher (`@talicode/core`)
**As** a developer, **I want** `npm install -g @talicode/core` to give me the `tali` command, **so that** install matches the README Quick Start.
- **Files:** `npm/core/package.json`, `npm/core/bin/tali.js`.
- **Acceptance criteria:**
  - [ ] `package.json` name `@talicode/core`, `"bin": { "tali": "./bin/tali.js" }`, `optionalDependencies` on all platform packages, `license: "MIT"`.
  - [ ] `bin/tali.js` resolves the installed platform package and `execFileSync`/`spawn`s the native `tali`, forwarding args + exit code.
- **Tests:** launcher unit test (resolution logic) where feasible; covered by US-6.2 smoke test.

### US-6.2 — Platform packages + smoke test
**As** a maintainer, **I want** per-platform packages and a smoke test, **so that** the launcher path is proven before real publish.
- **Files:** `npm/platform/talicode-<os>-<arch>/package.json` (templates), a pack/smoke script (e.g. `scripts/npm-smoke.sh`).
- **Acceptance criteria:**
  - [ ] Each platform package `package.json` gated by `os` + `cpu` fields; ships one `tali` binary.
  - [ ] Script stages the current-platform binary, `npm pack`s wrapper + platform pkg, installs the tarballs into a scratch dir, and confirms `npx tali --version` execs the native binary and forwards the exit code.
- **Tests:** the smoke script (single platform).
