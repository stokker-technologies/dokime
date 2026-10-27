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

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::client::secrets::{KeyringStore, SecretStore};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    connection: crate::connection::Args,

    #[arg(long, help = "Emit The Result As JSON Without Secret Values.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    if args.connection.has_key_file() {
        return Err(crate::Error::Input("Logout does not use --key-file".into()));
    }
    let resolved = args.connection.resolve()?;
    let endpoint = resolved.selection.target.endpoint().to_owned();
    let stored_endpoint = endpoint.clone();
    crate::connection::secrets(args.connection.timeout(), move || KeyringStore.remove(&stored_endpoint)).await?;
    if args.json {
        println!("{}", serde_json::json!({"status": "local_secrets_removed", "endpoint": endpoint}));
    } else {
        println!("Local Secrets Removed: {endpoint}");
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
