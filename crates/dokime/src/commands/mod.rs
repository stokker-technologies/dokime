/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod auth;
mod backend;
mod catalog;
mod cycle;
mod evidence;
mod execution;
mod gui;
mod id_file;
mod operation;
mod plan;
mod plugin;
mod serve;
mod status;
pub mod version;

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
    #[command(subcommand, about = "Validate And Manage Local Backend Credentials.")]
    Auth(auth::Command),

    #[command(about = "Check Backend Identity, Authentication And Readiness With A Deadline.")]
    Status(status::Args),

    #[command(subcommand, about = "Manage Saved Backend Profiles Without Connecting.")]
    Backend(backend::Command),

    #[command(subcommand, about = "Submit Cycles And Recover Durable Request Progress.")]
    Cycle(cycle::Command),

    #[command(subcommand, about = "Control One Execution Of A Cycle.")]
    Execution(execution::Command),

    #[command(subcommand, about = "Browse Registered Suites And Their Templates.")]
    Catalog(catalog::Command),

    #[command(subcommand, about = "Discover And Invoke Remote Operations.")]
    Operation(operation::Command),

    #[command(subcommand, about = "Preview And Register Canonical Selection Plans.")]
    Plan(plan::Command),

    #[command(subcommand, about = "Install Plugin Bundles On A Backend.")]
    Plugin(plugin::Command),

    Serve(serve::Args),
    Gui(gui::Args),
    Version(version::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::Auth(command) => auth::run(command).await,
        Command::Status(args) => status::run(args).await,
        Command::Backend(command) => backend::run(command),
        Command::Cycle(command) => cycle::run(command).await,
        Command::Execution(command) => execution::run(command).await,
        Command::Catalog(command) => catalog::run(command).await,
        Command::Operation(command) => operation::run(command).await,
        Command::Plan(command) => plan::run(command).await,
        Command::Plugin(command) => plugin::run(command).await,
        Command::Serve(args) => serve::run(args),
        Command::Gui(args) => gui::run(args),
        Command::Version(args) => {
            version::run(args);
            Ok(())
        }
    }
}

/* === Private Functions === */

/* === Tests === */
