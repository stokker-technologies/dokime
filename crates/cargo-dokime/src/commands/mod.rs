/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod backend;
mod build;
mod dev;
mod environment;
mod gui;
mod inspect;
mod install;
mod installation;
mod new;
mod serve;
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
pub enum Command {
    #[command(about = "Run the installed Dokime backend.")]
    Serve(serve::Args),
    #[command(about = "Report the installed backend and SDK identity.")]
    Version(version::Args),
    #[command(about = "Launch the native desktop GUI against an existing backend.")]
    Gui(gui::Args),
    New(new::Args),
    Build(build::Args),
    Dev(Box<dev::Args>),
    Doctor(dev::DoctorArgs),
    Inspect(inspect::Args),
    Install(install::Args),
}

/* === Traits === */

/* === Implementations === */
impl Command {
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn run(self) -> Result<(), crate::error::Error> {
        match self {
            Self::Serve(args) => serve::run(args).await,
            Self::Version(args) => version::run(args).await,
            Self::Gui(args) => gui::run(args).await,
            Self::New(args) => new::run(args),
            Self::Build(args) => build::run(args).await,
            Self::Dev(args) => dev::run(*args).await,
            Self::Doctor(args) => dev::doctor(args).await,
            Self::Inspect(args) => inspect::run(args),
            Self::Install(args) => install::run(args).await,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
