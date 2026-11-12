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
mod plan;
mod plugin;
mod proto;
mod server;
mod status;

/* === Re-Exports === */

/* === Imports === */

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
}

/* === Traits === */

/* === Implementations === */

impl Command {
    pub fn action(&self) -> &'static str {
        match self {
            Self::New(_) => "Create Project",
            Self::Server(_) => "Start Host",
            Self::Plugin(_) => "Plugin Command",
            Self::Proto(_) => "Export Protocol",
            Self::Catalog(_) => "Read Catalog",
            Self::Plan(_) => "Materialize Cases",
            Self::Cycle(_) => "Cycle Command",
            Self::Execution(_) => "Execution Command",
            Self::Status(_) => "Read Host Status",
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
    }
}

/* === Private Functions === */

/* === Tests === */
