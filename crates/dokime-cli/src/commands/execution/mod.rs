/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod export;
mod prompt;
pub(crate) mod recovery;
mod run;
mod start;
mod stop;
mod wait;

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
    #[command(about = "Start One Execution By Its Durable Identity Without Waiting For Completion.")]
    Start(start::Args),

    #[command(about = "Start One Execution And Wait For Its Durable Sealed Outcome.")]
    Run(run::Args),

    #[command(about = "Wait For A Durable Execution Outcome Without Replaying Work.")]
    Wait(wait::Args),

    #[command(subcommand, about = "Discover, Inspect And Answer Prompts For One Execution.")]
    Prompt(prompt::Command),

    #[command(about = "Stop One Execution And Wait For Its Durable Seal; Its Siblings Keep Running.")]
    Stop(stop::Args),

    #[command(about = "Download A Sealed Execution's Evidence While Its Cycle May Stay Active.")]
    Export(export::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    match command {
        Command::Start(args) => start::run(args, settings).await,
        Command::Run(args) => run::run(args, settings).await,
        Command::Wait(args) => wait::run(args, settings).await,
        Command::Prompt(command) => prompt::run(command, settings).await,
        Command::Stop(args) => stop::run(args, settings).await,
        Command::Export(args) => export::run(args, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
