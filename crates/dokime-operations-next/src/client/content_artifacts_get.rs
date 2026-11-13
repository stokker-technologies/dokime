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
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.content.Artifacts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::content::ArtifactsGetRequest, v1::content::ArtifactsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::content::ArtifactsGetRequest) -> Result<v1::content::ArtifactsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let (mut reply,cut)=super::views::read(context,&format!("artifact/{}",request.name),request.at.as_ref(),async {
 let entry=context.host.store().content(&request.name)?;
 if !entry.artifact {return Err(Error::Invalid("Get Artifact: Select an Artifact Identifier".into()));}
 Ok(v1::content::ArtifactsGetResponse {metadata:Some(v1::ArtifactMetadata::decode(entry.metadata.as_slice())?),file:Some(reference(context,&entry)?),..Default::default()})
}).await?;reply.resolved_at=Some(cut);Ok(reply)
    }).await
}

/* === Private Functions === */

pub(super) fn reference(context:&Context,entry:&dokime_storage::ContentEntry)->Result<harmos_protocol::storage::FileReference>{
 let digest=(0..entry.sha256.len()).step_by(2).map(|offset|u8::from_str_radix(&entry.sha256[offset..offset+2],16).map_err(|_|Error::Invalid("Read Content: Restore Its Stored Digest".into()))).collect::<Result<Vec<_>>>()?;
 Ok(harmos_protocol::storage::FileReference {metadata:Some(harmos_protocol::storage::FileMetadata {id:entry.identifier.clone(),scope:Some(entry.execution.clone()),media_type:if entry.artifact {v1::ArtifactMetadata::decode(entry.metadata.as_slice())?.content_type.unwrap_or_default()}else{"application/octet-stream".into()},expected_length:Some(entry.length),expected_sha256:Some(digest.clone())}),length:entry.length,sha256:digest,position:Some(harmos_protocol::storage::StorageCursor {history:Some(harmos_protocol::storage::StoreHistory {store:context.host.identity().backend_id,epoch:entry.execution.clone()}),sequence:context.host.store().metadata(&entry.execution)?.frontier})})
}

/* === Tests === */
