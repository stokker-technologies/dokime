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

// Standard Library Imports

// Third Party Imports

// Local Imports

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

impl Command {
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::List(_) => "List Suites",
            Self::Show(_) => "Read Suite",
        }
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    match command {
        Command::List(args) => list::run(args, endpoint, settings).await,
        Command::Show(args) => show::run(args, endpoint, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
