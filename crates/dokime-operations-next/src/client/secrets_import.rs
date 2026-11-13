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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Secrets/Import";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::SecretsImportRequest, v1::SecretsImportResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::SecretsImportRequest) -> Result<v1::SecretsImportResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        if request.validate_only {
            let super::secrets_store::Reply::Preview(preview) =
                super::secrets_store::perform(context, super::secrets_store::Action::Preview(request.data)).await?
            else {
                return Err(Error::Invalid("Preview Secrets: Restart the Secret Store".into()));
            };
            return Ok(v1::SecretsImportResponse {
                preview: Some(preview),
                ..Default::default()
            });
        }
        let receipt = super::views::receipt(context, &request.request_id)?;
        let super::secrets_store::Reply::Snapshot(snapshot) =
            super::secrets_store::perform(context, super::secrets_store::Action::Import(request)).await?
        else {
            return Err(Error::Invalid("Import Secrets: Restart the Secret Store".into()));
        };
        context.host.signal_change();
        Ok(v1::SecretsImportResponse {
            snapshot: Some(snapshot),
            receipt: Some(receipt),
            ..Default::default()
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
