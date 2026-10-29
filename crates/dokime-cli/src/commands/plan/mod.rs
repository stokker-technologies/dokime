/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Preview And Register Canonical Selection Plans

pub(crate) mod input;
mod materialize;
mod preview;
mod register;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    #[command(about = "Preview Cases Without Registering Them.")]
    Preview(preview::Args),

    #[command(about = "Register Cases Using A Caller-Retained Idempotency Key.")]
    Register(register::Args),

    #[command(about = "Materialize Canonical Template Cases Without Creating Or Starting Executions.")]
    Materialize(materialize::Args),
}

pub(crate) async fn run(command: Command, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    match command {
        Command::Preview(args) => preview::run(args, settings).await,
        Command::Register(args) => register::run(args, settings).await,
        Command::Materialize(args) => materialize::run(args, settings).await,
    }
}
