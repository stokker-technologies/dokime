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
use dokime_client::credentials::{KeyringStore, SecretStore};

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
pub(super) async fn run(args: Args) -> crate::Result<()> {
    let resolved = args.connection.resolve()?;
    let token = resolved.token.clone().ok_or_else(|| crate::Error::Input("Login requires --key-file or DOKIME_KEY.".into()))?;
    let client = args.connection.connect_selected(&resolved, Some(token.clone())).await?;
    let backend = client.backend().clone();
    client.close_and_wait().await;
    let endpoint = resolved.selection.target.endpoint().to_owned();
    let stored_endpoint = endpoint.clone();
    crate::connection::credentials(args.connection.timeout(), move || match resolved.store {
        Some(store) => store.remember(&resolved.selection, token.as_str(), &KeyringStore).map_err(crate::Error::from),
        None => KeyringStore.write(&stored_endpoint, token.as_str()).map_err(crate::Error::from),
    })
    .await?;
    if args.json {
        println!("{}", serde_json::json!({"status": "credential_saved", "endpoint": endpoint, "backend_id": backend.backend_id}));
    } else {
        println!("Credentials Saved: {} ({endpoint})", backend.backend_id);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
