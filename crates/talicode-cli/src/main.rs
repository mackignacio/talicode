// SPDX-License-Identifier: MIT
//! TaliCode CLI (`tali`) — entry point and command dispatch.
//!
//! Implements #5. Parses the command surface with clap and dispatches to the
//! per-command handlers in [`commands`]. Handlers are stubs here; later phases
//! fill them in (config/init in #7, provider/auditor in phase 2, the sweep
//! path in phase 4, usage/watch/skills in phase 5).

mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "tali",
    version,
    about = "TaliCode — the AI Slop Gatekeeper",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold a `config.tali` + `skills/` in the current repo.
    Init,
    /// Detect AI slop / architectural violations in staged (or target) code.
    Sweep(commands::sweep::Args),
    /// Run a sweep, then point at the healing roadmap (not yet enabled).
    Heal,
    /// List user-authored repo skills (`--all` to include bundled).
    Skills(commands::skills::Args),
    /// Show token spend: today's total + recent daily history.
    Usage(commands::usage::Args),
    /// Monitor the current folder/repo and sweep on change.
    Watch(commands::watch::Args),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Init => commands::init::run(),
        Command::Sweep(args) => commands::sweep::run(args).await,
        Command::Heal => commands::heal::run().await,
        Command::Skills(args) => commands::skills::run(args),
        Command::Usage(args) => commands::usage::run(args),
        Command::Watch(args) => commands::watch::run(args).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_sweep_with_flags() {
        let cli = Cli::try_parse_from(["tali", "sweep", "--staged", "--json"]).unwrap();
        match cli.command {
            Command::Sweep(a) => {
                assert!(a.staged && a.json && a.skill.is_none());
            }
            _ => panic!("expected sweep"),
        }
    }

    #[test]
    fn parses_bare_subcommands() {
        assert!(matches!(
            Cli::try_parse_from(["tali", "init"]).unwrap().command,
            Command::Init
        ));
        assert!(matches!(
            Cli::try_parse_from(["tali", "heal"]).unwrap().command,
            Command::Heal
        ));
    }

    #[test]
    fn unknown_command_errors() {
        assert!(Cli::try_parse_from(["tali", "nope"]).is_err());
    }
}
