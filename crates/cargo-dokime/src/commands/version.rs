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
use std::path::PathBuf;
// Third Party Imports

// Local Imports
use crate::{commands::backend, error::Error};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Debug, clap::Args)]
#[command(about = "Report the installed backend and SDK identity")]
pub struct Args {
    #[arg(long, help = "Emit stable host identity for development tools.")]
    json: bool,
    #[arg(long, help = "Backend override used for the identity probe.")]
    dokime: Option<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn run(args: Args) -> Result<(), Error> {
    if !args.json {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let installation = backend::resolve(args.dokime.as_deref())?;
    let identity = backend::inspect(installation.executable()).await?;
    println!(
        "{}",
        serde_json::json!({
            "version": identity.version,
            "sdk_revision": identity.sdk_revision,
            "dev_protocol": identity.dev_protocol,
        })
    );
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
