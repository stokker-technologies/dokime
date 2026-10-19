/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod prompt;
mod run;
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
    #[command(about = "Run One Pending Execution With A Caller-Retained Request Key.")]
    Run(run::Args),

    #[command(about = "Wait For A Durable Execution Outcome Without Replaying Work.")]
    Wait(wait::Args),

    #[command(subcommand, about = "Discover, Inspect And Answer Prompts For One Execution.")]
    Prompt(prompt::Command),

    #[command(about = "Request A Stop Of One Execution; Its Siblings Keep Running.")]
    Stop(stop::Args),

    #[command(about = "Download A Sealed Execution's Evidence While Its Cycle May Stay Active.")]
    Export(super::evidence::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::Run(args) => run::run(args).await,
        Command::Wait(args) => wait::run(args).await,
        Command::Prompt(command) => prompt::run(command).await,
        Command::Stop(args) => stop::run(args).await,
        Command::Export(args) => super::evidence::run(args, dokime_domain::model::exports::EvidenceSubject::Execution).await,
    }
}

/* === Private Functions === */

/* === Tests === */
