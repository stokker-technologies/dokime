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
    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(dokime_cli::Error::from)
        .and_then(|runtime| runtime.block_on(cli.run()));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // Preserve The Cause Chain In The Single Terminal Diagnostic.
            terminal.failure(dokime_contract::diagnostics::Diagnostic::capture(&error));
            error.exit_code()
        }
    }
}

/* === Tests === */
