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
use prost::Message;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.storage.Sources/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::storage::SourcesGetRequest, v1::storage::SourcesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::storage::SourcesGetRequest) -> Result<v1::storage::SourcesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let (mut reply,cut)=super::views::read(context,ID,request.at.as_ref(),async {
 let initial=v1::StorageSources::default();let (payload,revision)=context.host.store().administration_value("storage-sources",&initial.encode_to_vec())?;
 Ok(v1::storage::SourcesGetResponse {value:Some(v1::StorageSources::decode(payload.as_slice())?),revision,..Default::default()})
}).await?;reply.resolved_at=Some(cut);Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
