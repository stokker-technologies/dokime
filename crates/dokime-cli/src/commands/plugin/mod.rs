/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod install;

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
    Install(install::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::Install(args) => install::run(args).await,
    }
}

/* === Private Functions === */

/* === Tests === */
