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

/* === Constants === */

pub const ID: &str = "dokime.plugin.host.v1.content.Artifacts/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<
        dokime_protocol::context::content::ArtifactsCreateRequest,
        dokime_protocol::context::content::ArtifactsCreateResponse,
    >(ID, prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?)
}
pub async fn handle(
    resources: &Resources, request: dokime_protocol::context::content::ArtifactsCreateRequest,
) -> Result<dokime_protocol::context::content::ArtifactsCreateResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let scope = resources.store.metadata(&request.execution)?;
    let metadata = request.metadata.ok_or_else(|| crate::Error::Invalid("Create Artifact: Supply Its Metadata".into()))?;
    let reply = resources.store.attach_artifact(
        ID,
        dokime_protocol::client::content::ArtifactsCreateRequest {
            file: request.file,
            scope: Some(dokime_protocol::v1::ArtifactScope {
                execution_id: Some(request.execution),
                cycle_id: Some(scope.execution.cycle),
                ..Default::default()
            }),
            filename: if metadata.label.is_empty() { metadata.identifier } else { metadata.label },
            content_type: request.media_type,
            request_id: request.request_id,
        },
        resources.clock.now_ns(),
    )?;
    Ok(dokime_protocol::context::content::ArtifactsCreateResponse {
        name: reply.artifact.unwrap().artifact_id,
        receipt: reply.receipt,
    })
}

/* === Private Functions === */

/* === Tests === */
