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
use harmos_packaging::bundle::{Bundle, Limits};

// Local Imports
use crate::terminal::Terminal;
use crate::{Error, Result};
use dokime::domain::model::plugins::wire;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
#[command(about = "Install A Plugin Bundle On A Backend For Next-Boot Activation.")]
pub(crate) struct Args {
    bundle: PathBuf,

    #[command(flatten)]
    connection: crate::connection::Args,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> Result<()> {
    args.connection = args.connection.with_settings(*settings);
    let limits = Limits::default();
    let bytes = dokime::harmos::filesystem::input::read(&args.bundle, limits.archive_bytes)?;
    Bundle::inspect(&bytes, limits).map_err(|error| Error::Input(format!("invalid plugin bundle: {error}")))?;
    let digest = dokime::client::plugin_bundle_digest(&bytes);
    let endpoint = args.connection.resolve()?.selection.target.endpoint().to_owned();
    let client = args.connection.connect().await?;
    let result = client.plugins_install_bundle(bytes, digest.clone(), |_, _| {}).await;
    client.close_and_wait().await;
    let installed = result.map_err(|error| Error::Setup(format!(
        "installation was not confirmed: {error}. Inspect Plugins on {endpoint} before retrying; installation may have completed. No automatic retry or removal was attempted."
    )))?;
    if installed.sha256 != digest || installed.state != wire::PluginInstallationState::Installed || installed.install_id.is_empty() {
        return Err(Error::Setup(format!("unexpected installation response; inspect Plugins on {endpoint} before retrying")));
    }
    Terminal::default().success(format!("Installed plugin {}; restart the host to activate it.", installed.install_id));
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
