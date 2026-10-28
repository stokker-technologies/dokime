/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod append;
mod create;
mod seal;

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

pub(crate) async fn run(command: Command, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    match command {
        Command::Create(args) => create::run(args, settings).await,
        Command::Append(args) => append::run(args, settings).await,
        Command::Seal(args) => seal::run(args, settings).await,
        Command::Export(args) => super::evidence::run(args, dokime::domain::model::exports::EvidenceSubject::Cycle, None, settings).await,
    }
}
