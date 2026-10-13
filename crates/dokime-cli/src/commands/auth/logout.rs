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
    if args.connection.has_token_file() {
        return Err(crate::Error::Input(
            "Logout does not use --token-file".into(),
        ));
    }
    let resolved = args.connection.resolve()?;
    let endpoint = resolved.selection.target.endpoint().to_owned();
    let stored_endpoint = endpoint.clone();
    crate::connection::credentials(args.connection.timeout(), move || {
        KeyringStore.remove(&stored_endpoint)
    })
    .await?;
    if args.json {
        println!(
            "{}",
            serde_json::json!({"status": "local_credentials_removed", "endpoint": endpoint})
        );
    } else {
        println!("Local Credentials Removed: {endpoint}");
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
