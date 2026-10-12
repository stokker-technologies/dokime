/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod cycle;
mod operation;
pub mod serve;
pub mod version;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    #[command(
        subcommand,
        about = "Submit Cycles And Recover Durable Request Progress."
    )]
    Cycle(cycle::Command),
    #[command(subcommand, about = "Discover And Invoke Remote Operations.")]
    Operation(operation::Command),
    // Boxed: the serve arguments dwarf the other variants, and clippy's
    // large-enum-variant lint trips on the Windows layout.
    Serve(Box<serve::Args>),
    Version(version::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(
    command: Command,
    terminal: dokime_terminal::Terminal,
) -> crate::Result<()> {
    match command {
        Command::Cycle(command) => cycle::run(command).await,
        Command::Operation(command) => operation::run(command).await,
        Command::Serve(args) => serve::run(*args, terminal).await,
        Command::Version(args) => {
            version::run(args);
            Ok(())
        }
    }
}

/* === Private Functions === */

/* === Tests === */
