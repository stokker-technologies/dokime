/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod append;
mod create;
mod seal;

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
    #[command(about = "Create An Empty Active Cycle.")]
    Create(create::Args),

    #[command(about = "Register Explicit Executions In An Existing Cycle.")]
    Append(append::Args),

    #[command(about = "Close Admission, Stop This Cycle's Work And Wait For Durable Sealing.")]
    Seal(seal::Args),

    #[command(about = "Download A Self-Contained Evidence ZIP For A Sealed Cycle.")]
    Export(super::evidence::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::Create(args) => create::run(args).await,
        Command::Append(args) => append::run(args).await,
        Command::Seal(args) => seal::run(args).await,
        Command::Export(args) => super::evidence::run(args, dokime::domain::model::exports::EvidenceSubject::Cycle, None).await,
    }
}

/* === Private Functions === */

/* === Tests === */
