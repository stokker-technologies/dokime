/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod install;

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    Install(install::Args),
}

pub(crate) async fn run(command: Command, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    match command {
        Command::Install(args) => install::run(args, settings).await,
    }
}
