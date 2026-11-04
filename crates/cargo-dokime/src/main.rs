/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod cli;
mod commands;
mod error;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process::ExitCode;

// Third Party Imports
use clap::Parser;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> ExitCode {
    let _trace = dokime::diagnostics::test_trace::process();
    let mut args = std::env::args_os().collect::<Vec<_>>();
    if args.get(1).is_some_and(|arg| arg == "dokime") {
        args.remove(1);
    }
    let cli = cli::Cli::parse_from(args);
    dokime::terminal::set_color_mode(cli.color);
    let outcome = execute(cli);
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error::Error::DevelopmentExit(code)) => std::process::exit(code),
        Err(error::Error::Cancelled) => ExitCode::from(130),
        Err(error) => {
            dokime::terminal::Terminal::default().error(error);
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: cli::Cli) -> Result<(), error::Error> {
    dokime::process::block_on(cli.command.run())?
}

/* === Tests === */
