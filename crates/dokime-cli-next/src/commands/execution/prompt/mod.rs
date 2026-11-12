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
    Next(next::Args),
    Answer(answer::Args),
    Show(show::Args),
}

/* === Traits === */

/* === Implementations === */

impl Command {
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Next(_) => "Await Prompt",
            Self::Show(_) => "Read Prompt",
            Self::Answer(_) => "Answer Prompt",
        }
    }
}

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
