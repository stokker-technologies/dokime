/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog Commands

/* === Modules === */

mod list;
mod show;

/* === Re-Exports === */

/* === Imports === */

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    List(list::Args),
    Show(show::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    match command {
        Command::List(args) => list::run(args, endpoint, settings).await,
        Command::Show(args) => show::run(args, endpoint, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
