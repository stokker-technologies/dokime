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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.Secrets/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::SecretsDeleteRequest, v1::SecretsDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::SecretsDeleteRequest) -> Result<v1::SecretsDeleteResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let receipt = super::views::receipt(context, &request.request_id)?;
        let super::secrets_store::Reply::Snapshot(snapshot) =
            super::secrets_store::perform(context, super::secrets_store::Action::Remove(request.clone())).await?
        else {
            return Err(Error::Invalid("Delete Secret: Restart the Secret Store".into()));
        };
        context.host.signal_change();
        Ok(v1::SecretsDeleteResponse {
            receipt: Some(receipt),
            cleanup_pending: snapshot.cleanup_pending,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
