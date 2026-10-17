/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Preview And Register Canonical Selection Plans

/* === Modules === */
pub(crate) mod input;
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
#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    #[command(about = "Preview Cases Without Registering Them.")]
    Preview(preview::Args),
    #[command(about = "Register Cases Using A Caller-Retained Idempotency Key.")]
    Register(register::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::Preview(args) => preview::run(args).await,
        Command::Register(args) => register::run(args).await,
    }
}

/* === Private Functions === */

/* === Tests === */
