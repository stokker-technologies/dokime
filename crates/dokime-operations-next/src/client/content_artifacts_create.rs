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
use sha2::Digest;

/* === Constants === */

pub const ID: &str = "dokime.v1.content.Artifacts/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::content::ArtifactsCreateRequest, v1::content::ArtifactsCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::content::ArtifactsCreateRequest) -> Result<v1::content::ArtifactsCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
let reference=request.file.as_ref().ok_or_else(||Error::Invalid("Create Artifact: Select Published Content".into()))?;
let source=reference.metadata.as_ref().ok_or_else(||Error::Invalid("Create Artifact: Select a Named File Reference".into()))?;
let store=context.host.store();let entry=store.content(&source.id)?;
let verified=super::content_artifacts_get::reference(context,&entry)?;
if &verified!=reference {return Err(Error::Precondition("Create Artifact: Use the Original Published File Reference".into()));}
let scope=request.scope.as_ref().ok_or_else(||Error::Invalid("Create Artifact: Select an Execution Scope".into()))?;
if scope.execution_id.as_ref()!=Some(&entry.execution) {return Err(Error::Invalid("Create Artifact: Keep the File Within Its Execution Scope".into()));}
let id=super::views::hex(&sha2::Sha256::digest(format!("{}/{}",store.identity(),request.request_id)));
let metadata=v1::ArtifactMetadata {artifact_id:id.clone(),scope:Some(scope.clone()),filename:request.filename.clone(),content_type:Some(request.content_type.clone()),created_at:Some(super::projection::timestamp(context.host.now_ns())),..Default::default()};
if let Ok(previous)=store.content(&id) {let artifact=v1::ArtifactMetadata::decode(previous.metadata.as_slice())?;if artifact.filename!=request.filename || artifact.sha256!=entry.sha256{return Err(Error::Conflict("Create Artifact: Use a New Request ID for Changed Metadata or Content".into()));}return Ok(v1::content::ArtifactsCreateResponse {artifact:Some(artifact),receipt:Some(super::views::receipt(&request.request_id)?)});}
store.begin_artifact(metadata)?;let mut offset=0;
while offset<entry.length {let bytes=store.content_window(&entry.identifier,offset,dokime_storage::CONTENT_CHUNK_BYTES)?;store.append_content(&id,offset,&bytes)?;offset+=bytes.len() as u64;}
let artifact=store.finish_artifact(&id,entry.length,&entry.sha256)?;
Ok(v1::content::ArtifactsCreateResponse {artifact:Some(artifact),receipt:Some(super::views::receipt(&request.request_id)?)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
