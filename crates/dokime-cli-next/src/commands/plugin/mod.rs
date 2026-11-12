/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Commands

/* === Modules === */

mod list;
mod remove;

mod add;
mod bundle;
mod dev;
mod inspect;
mod install;

/* === Re-Exports === */

/* === Imports === */

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(clap::Subcommand)]
pub(crate) enum Command {
    Remove(remove::Args),
    List(list::Args),
    Bundle(bundle::Args),
    Dev(dev::Args),
    Inspect(inspect::Args),
    Add(add::Args),
    Install(install::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    match command {
        Command::Remove(args) => remove::run(args, endpoint, settings).await,
        Command::List(args) => list::run(args, endpoint, settings).await,
        Command::Bundle(args) => bundle::run(args, endpoint, settings).await,
        Command::Dev(args) => dev::run(args, endpoint, settings).await,
        Command::Inspect(args) => inspect::run(args, endpoint, settings).await,
        Command::Add(args) => add::run(args, endpoint, settings).await,
        Command::Install(args) => install::run(args, endpoint, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
