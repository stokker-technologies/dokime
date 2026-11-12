/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Resources, Result};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.artifact";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<v1::PublishArtifact, v1::ArtifactReceipt>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: v1::PublishArtifact) -> Result<v1::ArtifactReceipt> {
    resources.check_action_scope(&resources.execution, ID)?;
    dokime_storage::validate_artifact_name(&request.name).map_err(|error| crate::Error::Invalid(error.to_string()))?;
    if request.transfer_id.trim().is_empty()
        || request.transfer_id.len() > 255
        || request.sequence > i64::MAX as u64
        || request.mime.trim().is_empty()
        || request.mime.len() > 256
        || request.mime.chars().any(char::is_control)
    {
        return Err(crate::Error::Invalid("Publish Artifact: Supply a Transfer Identity, Media Type and Valid Sequence".into()));
    }
    if request.data.len() > dokime_storage::CONTENT_CHUNK_BYTES
        || request.data.is_empty() && !request.last
        || request.aborted && (!request.last || !request.data.is_empty() || request.replace)
    {
        return Err(crate::Error::Invalid(
            "Publish Artifact: Send Bounded Chunks; Abort with an Empty Terminal Chunk Without Replacement".into(),
        ));
    }
    let identifier = uuid::Uuid::now_v7().to_string();
    Ok(resources.store.publish_artifact_chunk(&resources.execution, request, &identifier, resources.clock.now_ns())?)
}

/* === Private Functions === */

/* === Tests === */
