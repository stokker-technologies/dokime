/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycle Commands

/* === Modules === */

mod append;
mod create;
mod rerun;
mod seal;

/* === Re-Exports === */

/* === Imports === */

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    Create(create::Args),
    Append(append::Args),
    Seal(seal::Args),
    Rerun(rerun::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    match command {
        Command::Create(args) => create::run(args, endpoint, settings).await,
        Command::Append(args) => append::run(args, endpoint, settings).await,
        Command::Seal(args) => seal::run(args, endpoint, settings).await,
        Command::Rerun(args) => rerun::run(args, endpoint, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
