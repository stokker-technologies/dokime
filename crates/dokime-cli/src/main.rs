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
use dokime::domain::model::diagnostics::Diagnostic;

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
    let terminal = dokime::terminal::Terminal::new(cli.color);
    let result = execute(cli);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => report_error(&terminal, error),
    }
}

fn execute(cli: dokime_cli::Cli) -> dokime_cli::Result<()> {
    let configuration = cli.configuration()?;
    let stack_bytes = 8 * 1024 * 1024;
    // The CLI sizes its own stacks the way `dokime::process` does:
    // Windows grants the main thread
    // one megabyte, which dev-profile futures overflow.
    let driver = std::thread::Builder::new()
        .name("dokime-main".to_owned())
        .stack_size(stack_bytes)
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_stack_size(stack_bytes)
                .build()
                .map_err(dokime_cli::Error::from)?;
            runtime.block_on(cli.run(configuration))
        })
        .map_err(dokime_cli::Error::from);
    driver.and_then(|driver| match driver.join() {
        Ok(outcome) => outcome,
        Err(panic) => std::panic::resume_unwind(panic),
    })
}

fn report_error(terminal: &dokime::terminal::Terminal, error: dokime_cli::Error) -> ExitCode {
    // Present Distinct Causes Without Capturing Debug Traces.
    let mut diagnostic = Diagnostic { message: error.to_string(), ..Default::default() };
    let mut source = std::error::Error::source(&error);
    while let Some(cause) = source {
        let message = cause.to_string();
        if message != diagnostic.message && !diagnostic.chain.contains(&message) {
            diagnostic.chain.push(message);
        }
        source = cause.source();
    }
    terminal.failure(diagnostic);
    error.exit_code()
}

/* === Tests === */
