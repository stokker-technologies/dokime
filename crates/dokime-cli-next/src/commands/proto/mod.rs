/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Proto Commands

/* === Modules === */

mod export;

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
    Export(export::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::Export(args) => export::run(args),
    }
}

/* === Private Functions === */

/* === Tests === */
