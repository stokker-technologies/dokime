/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod call;
mod list;

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
    #[command(about = "List Available Operations And Their Request Contracts.")]
    List(list::Args),
    #[command(about = "Validate Local JSON And Invoke An Operation Once.")]
    Call(call::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::List(args) => list::run(args).await,
        Command::Call(args) => call::run(args).await,
    }
}

/* === Private Functions === */

/* === Tests === */
