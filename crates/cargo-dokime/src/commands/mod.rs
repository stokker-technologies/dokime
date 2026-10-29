/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod installer;
mod sidecar;

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
    #[command(name = "__desktop-development", hide = true)]
    DesktopDevelopment(sidecar::dev::desktop::Args),
    #[command(about = "Package And Inspect Native Installers")]
    Installer(installer::Args),

    #[command(about = "Create, Build, Package And Develop Native Sidecars")]
    Sidecar(sidecar::Args),
}

/* === Traits === */

/* === Implementations === */

impl Command {
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn run(self) -> Result<(), crate::error::Error> {
        match self {
            Self::DesktopDevelopment(args) => sidecar::dev::desktop::run(args).await,
            Self::Installer(args) => installer::run(args).await,
            Self::Sidecar(args) => sidecar::run(args).await,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
