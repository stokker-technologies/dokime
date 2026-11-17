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

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::SecretsDeleteRequest;
use dokime_protocol::host::SecretsDeleteResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Secrets/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SecretsDeleteRequest) -> Result<SecretsDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let receipt = crate::host::views::receipt(context, &request.request_id)?;
        let crate::host::secrets::store::Reply::Snapshot(snapshot) =
            crate::host::secrets::store::perform(context, crate::host::secrets::store::Action::Remove(request.clone())).await?
        else {
            return Err(Error::Invalid("Delete Secret: Restart the Secret Store".into()));
        };
        context.host.signal_change();
        Ok(SecretsDeleteResponse {
            receipt: Some(receipt),
            cleanup_pending: snapshot.cleanup_pending,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SecretsDeleteRequest, SecretsDeleteResponse, context, "dokime.host.v1", "Secrets", "Delete");
