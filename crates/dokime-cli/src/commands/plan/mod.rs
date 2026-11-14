/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plan Commands

/* === Modules === */

mod materialize;
mod preview;
mod register;

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
    Materialize(materialize::Args),
    Preview(preview::Args),
    Register(register::Args),
}

/* === Traits === */

/* === Implementations === */

impl Command {
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Materialize(_) => "Materialize Cases",
            Self::Preview(_) => "Preview Selection",
            Self::Register(_) => "Register Selection",
        }
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    match command {
        Command::Preview(args) => preview::run(args, endpoint, settings).await,
        Command::Register(args) => register::run(args, endpoint, settings).await,
        Command::Materialize(args) => materialize::run(args, endpoint, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
