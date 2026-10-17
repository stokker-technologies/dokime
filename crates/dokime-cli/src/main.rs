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
const STACK_BYTES: usize = 32 * 1024 * 1024;

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
    let terminal = dokime_cli::terminal::Terminal::new(cli.color);
    // The client stays off the Dokime runtime crate, so it sizes its own
    // stacks the way `dokime::process` does: Windows grants the main thread
    // one megabyte, which dev-profile futures overflow.
    let driver = std::thread::Builder::new()
        .name("dokime-main".to_owned())
        .stack_size(STACK_BYTES)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_stack_size(STACK_BYTES)
                .build()
                .map_err(dokime_cli::Error::from)?;
            runtime.block_on(cli.run())
        })
        .map_err(dokime_cli::Error::from);
    let result = driver.and_then(|driver| match driver.join() {
        Ok(outcome) => outcome,
        Err(panic) => std::panic::resume_unwind(panic),
    });
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
