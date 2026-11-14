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

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Secrets/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::SecretsDeleteRequest, dokime_protocol::client::SecretsDeleteResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::SecretsDeleteRequest,
) -> Result<dokime_protocol::client::SecretsDeleteResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let receipt = super::views::receipt(context, &request.request_id)?;
        let super::secrets_store::Reply::Snapshot(snapshot) =
            super::secrets_store::perform(context, super::secrets_store::Action::Remove(request.clone())).await?
        else {
            return Err(Error::Invalid("Delete Secret: Restart the Secret Store".into()));
        };
        context.host.signal_change();
        Ok(dokime_protocol::client::SecretsDeleteResponse {
            receipt: Some(receipt),
            cleanup_pending: snapshot.cleanup_pending,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
