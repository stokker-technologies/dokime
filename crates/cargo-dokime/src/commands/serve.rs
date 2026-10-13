/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{ffi::OsString, path::PathBuf, process::Stdio};
// Third Party Imports
use tokio::process::Command;
// Local Imports
use crate::{commands::backend, error::Error};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
#[command(
    about = "Run the installed Dokime backend without a GUI",
    trailing_var_arg = true
)]
pub struct Args {
    #[arg(
        long,
        help = "Backend override, written ahead of the forwarded arguments. The default is the installed Dokime server executable."
    )]
    pub(crate) dokime: Option<PathBuf>,
    #[arg(
        allow_hyphen_values = true,
        help = "Arguments for the server process, forwarded verbatim."
    )]
    pub(crate) backend: Vec<OsString>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> Result<(), Error> {
    let installation = backend::resolve(args.dokime.as_deref())?;
    let mut command = Command::new(installation.executable());
    command
        .arg("--color")
        .arg(color())
        .args(&args.backend)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let status = command.status().await?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Invalid(format!(
            "Dokime backend exited with {}",
            status
                .code()
                .map_or_else(|| "a signal".to_owned(), |code| code.to_string())
        )))
    }
}

/* === Private Functions === */
fn color() -> &'static str {
    match dokime_terminal::color_mode() {
        dokime_terminal::ColorMode::Auto => "auto",
        dokime_terminal::ColorMode::Always => "always",
        dokime_terminal::ColorMode::Never => "never",
    }
}

/* === Tests === */
