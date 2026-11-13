/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

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

pub const ID: &str = "dokime.v1.Secrets/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::SecretsSetRequest, v1::SecretsSetResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::SecretsSetRequest) -> Result<v1::SecretsSetResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let receipt=super::views::receipt(&request.request_id)?;
let super::secrets_store::Reply::Snapshot(snapshot)=super::secrets_store::perform(context,super::secrets_store::Action::Put(request.clone())).await? else {return Err(Error::Invalid("Set Secret: Restart the Secret Store".into()));};
context.host.signal_change();Ok(v1::SecretsSetResponse {value:Some(v1::SecretWritable {metadata:request.value.and_then(|value|value.metadata),value:None}),revision:snapshot.generation.to_be_bytes().to_vec(),receipt:Some(receipt)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
