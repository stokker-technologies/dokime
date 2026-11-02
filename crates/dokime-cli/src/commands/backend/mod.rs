/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod add;
mod list;
mod remove;
mod show;
mod r#use;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use dokime::client::profiles::Store;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(super) struct Config {
    #[arg(long, help = "Use This Backend Profile Configuration File.")]
    profiles_file: Option<PathBuf>,
}

/* === Enums === */

#[derive(Debug, clap::Subcommand)]
pub(crate) enum Command {
    #[command(about = "Add A Backend Profile And Verify Remote Imports.")]
    Add(add::Args),

    #[command(about = "List Saved Profiles Without Connecting.")]
    List(list::Args),

    #[command(about = "Inspect A Saved Profile Without Connecting.")]
    Show(show::Args),

    #[command(about = "Choose The Default For Future Connections.")]
    Use(r#use::Args),

    #[command(about = "Remove A Saved Profile And Clear Its Default Selection.")]
    Remove(remove::Args),
}

/* === Traits === */

/* === Implementations === */

impl Config {
    pub(super) fn store(&self) -> crate::Result<Store> {
        Ok(Store::discover(self.profiles_file.as_deref())?)
    }
}

/* === Public Functions === */

pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::Add(args) => add::run(args).await,
        Command::List(args) => list::run(args),
        Command::Show(args) => show::run(args),
        Command::Use(args) => r#use::run(args),
        Command::Remove(args) => remove::run(args),
    }
}

/* === Private Functions === */

/* === Tests === */
