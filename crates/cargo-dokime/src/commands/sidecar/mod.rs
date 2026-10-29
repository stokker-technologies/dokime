/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod build;
pub(super) mod dev;
mod inspect;
mod new;
mod package;

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    New(new::Args),
    Build(Box<build::Args>),
    Package(package::Args),
    Dev(Box<dev::Args>),
    Inspect(inspect::Args),
}

pub async fn run(args: Args) -> Result<(), crate::error::Error> {
    match args.command {
        Command::New(args) => new::run(args),
        Command::Build(args) => build::run(*args).await,
        Command::Package(args) => package::run(args).await,
        Command::Dev(args) => dev::run(*args).await,
        Command::Inspect(args) => inspect::run(args),
    }
}
