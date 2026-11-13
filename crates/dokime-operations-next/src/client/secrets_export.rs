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
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Secrets/Export";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::SecretsExportRequest, v1::SecretsExportResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::SecretsExportRequest) -> Result<v1::SecretsExportResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let catalog = super::secrets_list::handle(context, v1::SecretsListRequest::default()).await?;
        let cut = super::secrets_list::cut(context, "", catalog.generation, request.at.as_ref())?;
        let super::secrets_store::Reply::Archive(archive) =
            super::secrets_store::perform(context, super::secrets_store::Action::Export(request)).await?
        else {
            return Err(Error::Invalid("Export Secrets: Restart the Secret Store".into()));
        };
        Ok(v1::SecretsExportResponse {
            archive: Some(archive),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
