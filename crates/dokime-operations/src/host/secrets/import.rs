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

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::SecretsImportRequest;
use dokime_protocol::host::SecretsImportResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Import";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SecretsImportRequest) -> Result<SecretsImportResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        if request.validate_only {
            let crate::host::secrets::store::Reply::Preview(preview) =
                crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Preview(request.data)).await?
            else {
                return Err(Error::Invalid("Preview Secrets: Restart the Secret Store".into()));
            };
            return Ok(SecretsImportResponse {
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
        Ok(SecretsImportResponse {
            snapshot: Some(snapshot),
            receipt: Some(receipt),
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SecretsImportRequest, SecretsImportResponse, context, "dokime.host.v1", "Secrets", "Import");
