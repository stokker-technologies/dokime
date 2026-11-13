/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Command Dispatch

/* === Modules === */

mod catalog;
mod cycle;
mod execution;
mod new;
mod operation;
mod plan;
mod plugin;
mod proto;
mod server;
mod status;
mod version;

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
    New(new::Args),
    Server(server::Args),

    #[command(subcommand)]
    Plugin(plugin::Command),

    #[command(subcommand)]
    Proto(proto::Command),

    #[command(subcommand)]
    Catalog(catalog::Command),

    #[command(subcommand)]
    Plan(plan::Command),

    #[command(subcommand)]
    Cycle(cycle::Command),

    #[command(subcommand)]
    Execution(execution::Command),

    Status(status::Args),
    Version(version::Args),

    #[command(subcommand)]
    Operation(operation::Command),
}

/* === Traits === */

/* === Implementations === */

impl Command {
    pub fn action(&self) -> &'static str {
        match self {
            Self::New(_) => "Create Project",
            Self::Server(_) => "Start Host",
            Self::Plugin(command) => command.action(),
            Self::Proto(_) => "Export Protocol",
            Self::Catalog(command) => command.action(),
            Self::Plan(command) => command.action(),
            Self::Cycle(command) => command.action(),
            Self::Execution(command) => command.action(),
            Self::Status(_) => "Read Host Status",
            Self::Version(_) => "Read Build Version",
            Self::Operation(command) => command.action(),
        }
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    match command {
        Command::New(args) => new::run(args).await,
        Command::Server(args) => server::run(args, endpoint, settings).await,
        Command::Plugin(command) => plugin::run(command, endpoint, settings).await,
        Command::Proto(command) => proto::run(command),
        Command::Catalog(command) => catalog::run(command, endpoint, settings).await,
        Command::Plan(command) => plan::run(command, endpoint, settings).await,
        Command::Cycle(command) => cycle::run(command, endpoint, settings).await,
        Command::Execution(command) => execution::run(command, endpoint, settings).await,
        Command::Status(args) => status::run(args, endpoint, settings).await,
        Command::Version(args) => version::run(args),
        Command::Operation(command) => operation::run(command, endpoint, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
