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

pub const ID: &str = "dokime.v1.Secrets/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::SecretsGetRequest, v1::SecretsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::SecretsGetRequest) -> Result<v1::SecretsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let catalog=super::secrets_list::handle(context,v1::SecretsListRequest::default()).await?;
let cut=super::secrets_list::cut(context,&request.name,catalog.generation,request.at.as_ref())?;
let super::secrets_store::Reply::Lookup(value)=super::secrets_store::perform(context,super::secrets_store::Action::Lookup(request.name)).await? else {return Err(Error::Invalid("Get Secret: Restart the Secret Store".into()));};
Ok(v1::SecretsGetResponse {value:Some(value),revision:catalog.generation.to_be_bytes().to_vec(),resolved_at:Some(cut)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
