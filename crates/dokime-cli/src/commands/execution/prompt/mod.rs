/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Compose Execution Prompt Commands

mod answer;
mod next;
mod show;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    #[command(about = "Find A Pending Prompt For One Execution, Optionally Waiting.")]
    Next(next::Args),

    #[command(about = "Display A Prompt Bound To Its Execution And Prompt UUIDs.")]
    Show(show::Args),

    #[command(about = "Answer One Prompt Once; Never Retry Automatically After An Uncertain Reply.")]
    Answer(answer::Args),
}

pub(crate) async fn run(command: Command, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    match command {
        Command::Next(args) => next::run(args, settings).await,
        Command::Show(args) => show::run(args, settings).await,
        Command::Answer(args) => answer::run(args, settings).await,
    }
}
