/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secrets Delete

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::SecretsDeleteRequest, dokime_protocol::host::SecretsDeleteResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::SecretsDeleteRequest,
) -> Result<dokime_protocol::host::SecretsDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let receipt = crate::host::views::receipt(context, &request.request_id)?;
        let crate::host::secrets::store::Reply::Snapshot(snapshot) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Remove(request.clone())).await?
        else {
            return Err(Error::Invalid("Delete Secret: Restart the Secret Store".into()));
        };
        context.host.signal_change();
        Ok(dokime_protocol::host::SecretsDeleteResponse {
            receipt: Some(receipt),
            cleanup_pending: snapshot.cleanup_pending,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
