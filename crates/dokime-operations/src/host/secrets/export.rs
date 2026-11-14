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
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Export";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::SecretsExportRequest, dokime_protocol::host::SecretsExportResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::SecretsExportRequest,
) -> Result<dokime_protocol::host::SecretsExportResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let catalog = crate::host::secrets::list::handle(context, dokime_protocol::host::SecretsListRequest::default()).await?;
        let cut = crate::host::secrets::list::cut(context, "", catalog.generation, request.at.as_ref())?;
        let crate::host::secrets::store::Reply::Archive(archive) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Export(request)).await?
        else {
            return Err(Error::Invalid("Export Secrets: Restart the Secret Store".into()));
        };
        Ok(dokime_protocol::host::SecretsExportResponse {
            archive: Some(archive),
            resolved_at: Some(cut),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
