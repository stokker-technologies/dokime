/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

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
    let cli = dokime_cli::Cli::parse();
    let terminal = dokime_terminal::Terminal::new(cli.color);
    // `serve` owns loading its own environment (an explicit `--env-file` or
    // the walk-up `.env`); a blind load here ran before `--env-file` had a
    // chance to apply and, because dotenv never overrides an already-set
    // variable, could make the explicit file moot for variables the
    // walk-up happened to set first — a silent, hard-to-diagnose source of
    // a wrong secret value.
    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(dokime_cli::Error::from)
        .and_then(|runtime| runtime.block_on(cli.run()));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Captured rather than printed: the cause chain and the backtrace
            // are what a host failure has always had and never shown, and an
            // operator reading one top-level line has to guess the rest.
            terminal.failure(dokime::plugin::Diagnostic::capture(&error));
            error.exit_code()
        }
    }
}

/* === Tests === */
