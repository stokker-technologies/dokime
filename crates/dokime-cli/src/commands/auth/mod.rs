/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Backend Authentication And Connection Inspection

/* === Modules === */

mod login;
mod logout;
mod status;

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
    #[command(about = "Validate And Save The Key From --key-file Or DOKIME_KEY.")]
    Login(login::Args),

    #[command(about = "Check Authentication And Backend Readiness With A Deadline.")]
    Status(status::Args),

    #[command(about = "Remove Only The Selected Endpoint's Local Secrets.")]
    Logout(logout::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(command: Command, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    match command {
        Command::Login(args) => login::run(args, settings).await,
        Command::Status(args) => status::run(args, settings).await,
        Command::Logout(args) => logout::run(args, settings).await,
    }
}

/* === Private Functions === */

/* === Tests === */
