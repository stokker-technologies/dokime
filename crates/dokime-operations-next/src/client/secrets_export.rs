/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secrets Export

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Secrets/Export";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::SecretsExportRequest, dokime_protocol::client::SecretsExportResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::SecretsExportRequest,
) -> Result<dokime_protocol::client::SecretsExportResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let catalog = super::secrets_list::handle(context, dokime_protocol::client::SecretsListRequest::default()).await?;
        let cut = super::secrets_list::cut(context, "", catalog.generation, request.at.as_ref())?;
        let super::secrets_store::Reply::Archive(archive) =
            super::secrets_store::perform(context, super::secrets_store::Action::Export(request)).await?
        else {
            return Err(Error::Invalid("Export Secrets: Restart the Secret Store".into()));
        };
        Ok(dokime_protocol::client::SecretsExportResponse {
            archive: Some(archive),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
