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
mod output;
mod settings;

/* === Re-Exports === */

/* === Imports === */

use clap::Parser;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = cli::Cli::parse();
    let settings = match settings::Settings::load(cli.config.as_deref()) {
        Ok(settings) => settings,
        Err(error) => {
            eprintln!("Read Configuration: {error}");
            return std::process::ExitCode::from(2);
        }
    };
    let action = cli.command.action();
    match commands::run(cli.command, cli.endpoint.or(settings.endpoint.clone()), settings).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{action}: {error}");
            let code = match error.downcast_ref::<tonic::Status>().map(tonic::Status::code) {
                Some(tonic::Code::ResourceExhausted) => 4,
                Some(tonic::Code::InvalidArgument) => 2,
                _ => 3,
            };
            std::process::ExitCode::from(code)
        }
    }
}

/* === Tests === */
