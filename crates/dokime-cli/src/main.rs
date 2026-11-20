/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Command-Line Startup

/* === Modules === */

mod cli;
mod commands;
mod environment;
mod error;
mod output;
mod settings;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{env, process::ExitCode, result};

// Third Party Imports
use clap::{CommandFactory, Parser, error::ErrorKind};
use tokio::runtime::Builder;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Result<T> = result::Result<T, error::Error>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> ExitCode {
    let mut cli = cli::Cli::parse();
    if cli.version {
        return match cli::version(cli.json) {
            Ok(version) => {
                println!("{version}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("Read Build Version: {error}");
                ExitCode::from(error.exit_code())
            }
        };
    }
    let command = cli
        .command
        .take()
        .unwrap_or_else(|| cli::Cli::command().error(ErrorKind::MissingSubcommand, "Select a Command or Use --version").exit());
    let runtime = match Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("Start Command Runtime: {error}");
            return ExitCode::FAILURE;
        }
    };
    let result = runtime.block_on(run(cli, command));
    // A Cancelled Console Read Must Not Keep the Process Alive After Owned Cleanup.
    runtime.shutdown_background();
    result
}

async fn run(cli: cli::Cli, command: commands::Command) -> ExitCode {
    let mut settings = match settings::Settings::load(cli.config.as_deref(), cli.connect_timeout) {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("Read Configuration: {error}");
            return ExitCode::from(2);
        }
    };
    if let Some(namespace) = cli.credential_namespace {
        settings.credential_namespace = namespace;
        if let Err(error) = settings.credentials() {
            eprintln!("Read Configuration: {error}");
            return ExitCode::from(2);
        }
    }
    settings.remote = cli.remote.or_else(|| {
        if cli.endpoint.is_none() { env::var_os("DOKIME_REMOTE").filter(|value| !value.is_empty()).map(Into::into) } else { None }
    });
    let action = command.action();
    match Box::pin(commands::run(command, cli.endpoint.or(settings.endpoint.clone()), settings)).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{action}: {error}");
            let code = error.exit_code();
            ExitCode::from(code)
        }
    }
}

/* === Tests === */
