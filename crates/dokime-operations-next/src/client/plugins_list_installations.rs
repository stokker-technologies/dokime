/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};

use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.plugins.list_installations";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PluginsListInstallationsRequest, v1::InstallationPage>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PluginsListInstallationsRequest) -> Result<v1::InstallationPage> {
    super::complete(ID, context.settings.read_timeout, async {
        use prost::Message;
        use sha2::{Digest, Sha256};
        let installations = context.host.installations()?;
        let digest = Sha256::digest(
            v1::InstallationPage {
                installations: installations.clone(),
                ..Default::default()
            }
            .encode_to_vec(),
        )
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
        if request.after.as_ref().is_some_and(|after| after.manifest_sha256 != digest) {
            return Err(Error::Stale("Installation Inventory Changed: Restart Listing".into()));
        }
        let after = request.after.map(|after| after.after_install_id).unwrap_or_default();
        if !after.is_empty() && !installations.iter().any(|installation| installation.install_id == after) {
            return Err(Error::Invalid("Continue Installations: Select an Existing After Identifier".into()));
        }
        let total_count = installations.len() as u64;
        let limit = super::projection::limit(request.page_size, context.settings.page_items);
        let mut selected = installations.into_iter().filter(|installation| installation.install_id > after);
        let installations: Vec<_> = selected.by_ref().take(limit).collect();
        let next = selected.next().and_then(|_| {
            installations.last().map(|installation| v1::InstallationCursor {
                manifest_sha256: digest.clone(),
                after_install_id: installation.install_id.clone(),
            })
        });
        Ok(v1::InstallationPage {
            installations,
            next,
            total_count,
            manifest_sha256: digest,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
