/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Prompt Commands

/* === Modules === */

mod answer;
mod next;
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
    Next(next::Args),
    Answer(answer::Args),
    Show(show::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    match command {
        Command::Next(args) => next::run(args, endpoint, settings).await,
        Command::Answer(args) => answer::run(args, endpoint, settings).await,
        Command::Show(args) => show::run(args, endpoint, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
