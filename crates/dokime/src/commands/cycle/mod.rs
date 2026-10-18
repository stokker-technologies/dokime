/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod append;
mod create;
mod request;
mod rerun;
mod run;
mod seal;
mod stop;

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
    #[command(about = "Create An Empty Cycle, Optionally Append Registered Cases And Submit It.")]
    Create(create::Args),
    #[command(about = "Append Planned Cases To An Existing Cycle.")]
    Append(append::Args),
    #[command(about = "Submit An Existing Cycle Once With A Caller-Retained Key.")]
    Run(run::Args),
    #[command(about = "Read Or Wait For A Durable Request Without Replaying Work.")]
    Request(request::Args),
    #[command(about = "Request A Stop Of Every Execution In The Cycle Without Waiting For Sealing.")]
    Stop(stop::Args),

    #[command(about = "Close Admission, Stop This Cycle's Work And Wait For Durable Sealing.")]
    Seal(seal::Args),
    #[command(about = "Append A Case's Next Attempt As A New Execution And Admit It Once.")]
    Rerun(rerun::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::Create(args) => create::run(args).await,
        Command::Append(args) => append::run(args).await,
        Command::Run(args) => run::run(args).await,
        Command::Request(args) => request::run(args).await,
        Command::Stop(args) => stop::run(args).await,
        Command::Seal(args) => seal::run(args).await,
        Command::Rerun(args) => rerun::run(args).await,
    }
}

/* === Private Functions === */

/* === Tests === */
