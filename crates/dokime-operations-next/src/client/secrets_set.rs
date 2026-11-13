/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Secrets Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Error, Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.Secrets/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::SecretsSetRequest, dokime_protocol::client::SecretsSetResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::SecretsSetRequest,
) -> Result<dokime_protocol::client::SecretsSetResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let receipt = super::views::receipt(context, &request.request_id)?;
        let super::secrets_store::Reply::Snapshot(snapshot) =
            super::secrets_store::perform(context, super::secrets_store::Action::Put(request.clone())).await?
        else {
            return Err(Error::Invalid("Set Secret: Restart the Secret Store".into()));
        };
        context.host.signal_change();
        Ok(dokime_protocol::client::SecretsSetResponse {
            value: Some(dokime_protocol::client::SecretWritable {
                metadata: request.value.and_then(|value| value.metadata),
                value: None,
            }),
            revision: snapshot.generation.to_be_bytes().to_vec(),
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
