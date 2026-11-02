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

// Third Party Imports

// Local Imports
use dokime::client::profiles::{Error, Profile, Secret};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[command(flatten)]
    config: super::Config,

    #[arg(long, conflicts_with_all = ["label", "endpoint"], help = "Import A Version 1 Remote Access File.")]
    remote: Option<std::path::PathBuf>,

    #[arg(long, required_unless_present = "remote", help = "Saved Backend Profile Identifier.")]
    identifier: Option<String>,

    #[arg(long, required_unless_present = "remote", help = "Human Backend Label.")]
    label: Option<String>,

    #[arg(long, required_unless_present = "remote", help = "Unsecured Lab HTTP Origin.")]
    endpoint: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn run(args: Args) -> crate::Result<()> {
    let store = args.config.store()?;
    if let Some(path) = args.remote {
        let pairing = dokime::client::pairing::read(&path)?;
        let identifier = args.identifier.as_deref().unwrap_or(&pairing.backend_identifier);
        let profile = store.import_pairing(&pairing, identifier, &dokime::client::secrets::KeyringStore)?;
        let selected = dokime::client::profiles::selection(&profile)?;
        let client =
            dokime::client::Client::connect(selected.target.clone(), pairing.access_key.as_deref(), Some(&pairing.backend_identifier))
                .await?;
        store.remember_endpoint(&selected, client.endpoint())?;
        client.close_and_wait().await;
        return Ok(());
    }
    let endpoint = args.endpoint.ok_or_else(|| crate::Error::Input("endpoint is required".into()))?;
    if !endpoint.starts_with("http://") {
        return Err(crate::Error::Input("manual profiles require an unsecured lab HTTP endpoint".into()));
    }
    let profile = Profile {
        identifier: args.identifier.ok_or_else(|| crate::Error::Input("identifier is required".into()))?,
        label: args.label.ok_or_else(|| crate::Error::Input("label is required".into()))?,
        endpoint,
        endpoints: Vec::new(),
        expected_backend_id: None,
        ca_certificate_pem: None,
        secret: Secret::None,
    };
    dokime::client::profiles::selection(&profile)?;
    store.update(move |document| {
        if document.profiles.iter().any(|existing| existing.identifier == profile.identifier) {
            return Err(Error::Invalid("profile already exists".into()));
        }
        document.profiles.push(profile);
        Ok(())
    })?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
