/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Commands

/* === Modules === */

mod show;

mod list;
mod prompt;
mod report;
mod start;
mod stop;
mod wait;

/* === Re-Exports === */

/* === Imports === */

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    Show(show::Args),
    List(list::Args),
    Start(start::Args),
    Stop(stop::Args),
    #[command(alias = "watch")]
    Wait(wait::Args),
    Report(report::Args),
    #[command(subcommand)]
    Prompt(prompt::Command),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    match command {
        Command::Show(args) => show::run(args, endpoint, settings).await,
        Command::List(args) => list::run(args, endpoint, settings).await,
        Command::Start(args) => start::run(args, endpoint, settings).await,
        Command::Stop(args) => stop::run(args, endpoint, settings).await,
        Command::Wait(args) => wait::run(args, endpoint, settings).await,
        Command::Report(args) => report::run(args, endpoint, settings).await,
        Command::Prompt(args) => prompt::run(args, endpoint, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
