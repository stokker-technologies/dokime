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
use std::{path::PathBuf, process::Stdio};
// Third Party Imports
use tokio::process::Command;
// Local Imports
use crate::{
    commands::{backend, installation},
    error::Error,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
#[command(about = "Launch the native desktop GUI against an existing backend")]
pub struct Args {
    #[arg(
        long,
        visible_alias = "connect",
        help = "Existing native gRPC endpoint. The connect spelling is an alias."
    )]
    endpoint: String,
    #[arg(
        long,
        help = "Expected backend identity supplied to the native connection screen."
    )]
    expected_backend_id: Option<String>,
    #[arg(
        long,
        help = "Backend override whose directory also provides the installed GUI."
    )]
    dokime: Option<PathBuf>,
    #[arg(
        long,
        help = "Explicit GUI executable. The default is the one beside the backend."
    )]
    gui: Option<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> Result<(), Error> {
    let executable = match args.gui.as_deref() {
        // An explicit GUI attaches to an endpoint this machine need not host,
        // so it must not require a discoverable installation of its own.
        Some(path) => installation::gui_override(path)?,
        None => backend::resolve(args.dokime.as_deref())?.gui()?,
    };
    let mut command = Command::new(&executable);
    command
        .arg("--endpoint")
        .arg(args.endpoint)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if let Some(identity) = args.expected_backend_id {
        command.arg("--expected-backend-id").arg(identity);
    }
    let status = command.status().await?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Invalid(format!(
            "Dokime GUI exited with {}",
            status
                .code()
                .map_or_else(|| "a signal".to_owned(), |code| code.to_string())
        )))
    }
}

/* === Private Functions === */

/* === Tests === */
