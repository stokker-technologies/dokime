/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod call;
mod list;
mod show;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    #[command(about = "List Available Operations And Their Request Contracts.")]
    List(list::Args),

    #[command(about = "Show Live Provider Contracts For An Operation.")]
    Show(show::Args),

    #[command(about = "Call A Discovered Provider And Read Its Items And Completion.")]
    Call(call::Args),
}

pub(crate) async fn run(command: Command, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    match command {
        Command::List(args) => list::run(args, settings).await,
        Command::Show(args) => show::run(args, settings).await,
        Command::Call(args) => call::run(args, settings).await,
    }
}
