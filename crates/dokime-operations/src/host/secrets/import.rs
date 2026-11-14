/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secrets Import

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Import";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::SecretsImportRequest, dokime_protocol::host::SecretsImportResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::SecretsImportRequest,
) -> Result<dokime_protocol::host::SecretsImportResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        if request.validate_only {
            let crate::host::secrets::store::Reply::Preview(preview) =
                crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Preview(request.data)).await?
            else {
                return Err(Error::Invalid("Preview Secrets: Restart the Secret Store".into()));
            };
            return Ok(dokime_protocol::host::SecretsImportResponse {
                preview: Some(preview),
                ..Default::default()
            });
        }
        let receipt = crate::host::views::receipt(context, &request.request_id)?;
        let crate::host::secrets::store::Reply::Snapshot(snapshot) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Import(request)).await?
        else {
            return Err(Error::Invalid("Import Secrets: Restart the Secret Store".into()));
        };
        context.host.signal_change();
        Ok(dokime_protocol::host::SecretsImportResponse {
            snapshot: Some(snapshot),
            receipt: Some(receipt),
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
