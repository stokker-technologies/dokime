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
mod error;
mod output;
mod settings;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use clap::Parser;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Result<T> = std::result::Result<T, error::Error>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = cli::Cli::parse();
    let mut settings = match settings::Settings::load(cli.config.as_deref()) {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("Read Configuration: {error}");
            return std::process::ExitCode::from(2);
        }
    };
    settings.remote = cli.remote.or_else(|| {
        if cli.endpoint.is_none() { std::env::var_os("DOKIME_REMOTE").filter(|value| !value.is_empty()).map(Into::into) } else { None }
    });
    let action = cli.command.action();
    match commands::run(cli.command, cli.endpoint.or(settings.endpoint.clone()), settings).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{action}: {error}");
            let code = error.exit_code();
            std::process::ExitCode::from(code)
        }
    }
}

/* === Tests === */
