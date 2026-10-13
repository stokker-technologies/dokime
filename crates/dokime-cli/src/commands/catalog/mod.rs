/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Browse Registered Suite Catalogs

/* === Modules === */
mod list;
mod show;

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
    #[command(about = "List Registered Suites In Catalog Order.")]
    List(list::Args),
    #[command(about = "Show A Registered Suite And Its Template Contracts.")]
    Show(show::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::List(args) => list::run(args).await,
        Command::Show(args) => show::run(args).await,
    }
}

/* === Private Functions === */

/* === Tests === */
