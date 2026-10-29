/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod bindings;
mod installer;
mod sidecar;

#[derive(clap::Subcommand)]
pub enum Command {
    #[command(name = "__desktop-development", hide = true)]
    DesktopDevelopment(sidecar::dev::desktop::Args),
    #[command(about = "Generate Client Bindings From Operation Contracts")]
    Bindings(bindings::Args),
    #[command(about = "Package And Inspect Native Installers")]
    Installer(installer::Args),

    #[command(about = "Create, Build, Package And Develop Native Sidecars")]
    Sidecar(sidecar::Args),
}

impl Command {
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn run(self) -> Result<(), crate::error::Error> {
        match self {
            Self::DesktopDevelopment(args) => sidecar::dev::desktop::run(args).await,
            Self::Bindings(args) => bindings::run(args),
            Self::Installer(args) => installer::run(args).await,
            Self::Sidecar(args) => sidecar::run(args).await,
        }
    }
}
