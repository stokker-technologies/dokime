/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod stop;

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
    #[command(about = "Request A Stop Of One Execution; Its Siblings Keep Running.")]
    Stop(stop::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(command: Command) -> crate::Result<()> {
    match command {
        Command::Stop(args) => stop::run(args).await,
    }
}

/* === Private Functions === */

/* === Tests === */
