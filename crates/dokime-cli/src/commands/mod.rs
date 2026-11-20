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
mod dev;
mod doctor;
mod download;
mod execution;
mod id_file;
mod installer;
mod new;
mod operation;
pub(crate) mod pairing;
mod plan;
mod plugin;
mod proto;
mod server;
mod status;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, settings::Settings};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    New(new::Args),
    Installer(installer::Args),
    Server(server::Args),
    Dev(dev::Args),

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

    #[command(subcommand)]
    Doctor(doctor::Command),

    #[command(subcommand)]
    Access(pairing::Command),

    Status(status::Args),

    #[command(subcommand)]
    Operation(operation::Command),
}

/* === Traits === */

/* === Implementations === */

impl Command {
    pub fn action(&self) -> &'static str {
        match self {
            Self::New(_) => "Create Project",
            Self::Installer(_) => "Package Installer",
            Self::Server(_) => "Start Host",
            Self::Dev(_) => "Develop Plugin",
            Self::Plugin(command) => command.action(),
            Self::Proto(_) => "Export Protocol",
            Self::Catalog(command) => command.action(),
            Self::Plan(command) => command.action(),
            Self::Cycle(command) => command.action(),
            Self::Execution(command) => command.action(),
            Self::Doctor(_) => "Diagnose Storage",
            Self::Access(_) => "Export Access Pairing",
            Self::Status(_) => "Read Host Status",
            Self::Operation(command) => command.action(),
        }
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, mut settings: Settings) -> Result<()> {
    if !matches!(
        command,
        Command::Installer(_) | Command::New(_) | Command::Dev(_) | Command::Server(_) | Command::Doctor(_) | Command::Proto(_)
    ) {
        settings.pairing = settings.remote.as_deref().map(pairing::File::read).transpose()?;
    }
    match command {
        Command::New(args) => new::run(args, settings).await,
        Command::Installer(args) => installer::run(args).await,
        Command::Dev(args) => Box::pin(dev::run(args, settings)).await,
        Command::Server(args) => Box::pin(server::run(args, endpoint, settings)).await,
        Command::Plugin(command) => Box::pin(plugin::run(command, endpoint, settings)).await,
        Command::Proto(command) => proto::run(command),
        Command::Catalog(command) => catalog::run(command, endpoint, settings).await,
        Command::Plan(command) => plan::run(command, endpoint, settings).await,
        Command::Cycle(command) => cycle::run(command, endpoint, settings).await,
        Command::Execution(command) => execution::run(command, endpoint, settings).await,
        Command::Access(command) => pairing::run(command, endpoint, settings).await,
        Command::Doctor(command) => Box::pin(doctor::run(command, settings)).await,
        Command::Status(args) => status::run(args, endpoint, settings).await,
        Command::Operation(command) => operation::run(command, endpoint, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
