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

mod list;
mod prompt;
mod report;
mod show;
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

impl Command {
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::List(_) => "List Executions",
            Self::Show(_) => "Read Execution",
            Self::Start(_) => "Start Execution",
            Self::Stop(_) => "Stop Execution",
            Self::Wait(_) => "Watch Execution",
            Self::Report(_) => "Fetch Report",
            Self::Prompt(command) => command.action(),
        }
    }
}

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
