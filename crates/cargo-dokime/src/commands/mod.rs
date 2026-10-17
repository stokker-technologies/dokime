/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod build;
mod dev;
mod inspect;
mod new;
mod package;

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
    New(new::Args),
    Build(Box<build::Args>),
    Package(package::Args),
    Dev(Box<dev::Args>),
    Inspect(inspect::Args),
}

/* === Traits === */

/* === Implementations === */
impl Command {
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn run(self) -> Result<(), crate::error::Error> {
        match self {
            Self::New(args) => new::run(args),
            Self::Build(args) => build::run(*args).await,
            Self::Package(args) => package::run(args).await,
            Self::Dev(args) => dev::run(*args).await,
            Self::Inspect(args) => inspect::run(args),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
