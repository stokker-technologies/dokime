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

mod add;
mod bundle;
mod dev;
mod inspect;
mod install;
mod list;
mod remove;

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

impl Command {
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Bundle(_) => "Bundle Plugin",
            Self::Dev(_) => "Develop Plugin",
            Self::Inspect(_) => "Inspect Plugin",
            Self::Add(_) => "Add Plugin",
            Self::Install(_) => "Install Plugin",
            Self::List(_) => "List Plugins",
            Self::Remove(_) => "Remove Plugin",
        }
    }
}

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
