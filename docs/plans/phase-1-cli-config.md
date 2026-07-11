# Phase 1 — CLI & Config

> Part of the [TaliCode MVP](./MVP.md). Every story below becomes a GitHub issue; every commit references its issue.

## Goal

The `tali` command surface (clap) with dispatch, the `config.tali` schema (YAML via `serde_yaml`), and `tali init`.

## User Stories

### US-1.1 — CLI skeleton & command dispatch
**As** a developer, **I want** a `tali` CLI with subcommands, **so that** I have a stable command surface.
- **Files:** `crates/talicode-cli/src/main.rs`, `crates/talicode-cli/src/commands/{mod.rs,init.rs,sweep.rs,heal.rs,skills.rs,usage.rs,watch.rs}` (stubs).
- **Acceptance criteria:**
  - [ ] `clap` (derive) parses `init`, `sweep`, `heal`, `skills`, `usage`, `watch`.
  - [ ] `tali --version` / `tali --help` work; unknown command errors cleanly.
  - [ ] Each subcommand dispatches to its `commands/*.rs` handler (stubs OK).
- **Tests:** `main.rs` dispatch test (arg parse → correct command enum).

### US-1.2 — `config.tali` schema (serde) + validation
**As** a developer, **I want** `config.tali` parsed and validated, **so that** misconfig fails fast with a clear message.
- **Files:** `crates/talicode-core/src/config.rs`.
- **Acceptance criteria:**
  - [ ] Serde structs: `version`, `name`, `agents` (`{provider, model, role, effort?}`, `effort` defaults `"medium"`), `execution_flow` (`{agent, target, anti_slop?}`), top-level `skills:` list.
  - [ ] Content parsed as YAML from a `.tali` file (`serde_yaml`).
  - [ ] `validate()` checks required references (e.g. a step's `agent` exists).
  - [ ] Unknown-but-valid fields preserved, not rejected.
- **Tests:** valid config deserializes; each invalid shape rejected; missing-file error is clear.

### US-1.3 — `tali init`
**As** a developer, **I want** `tali init` to scaffold my repo, **so that** I get a working `config.tali` and skills dir.
- **Files:** `crates/talicode-cli/src/commands/init.rs`.
- **Acceptance criteria:**
  - [ ] Writes a starter `config.tali` (matches README example).
  - [ ] Refuses to overwrite an existing `config.tali`.
  - [ ] Scaffolds an empty repo `skills/` dir.
  - [ ] Adds `.talicode/` to the repo `.gitignore`.
- **Tests:** writes config; refuses overwrite; gitignore updated.
