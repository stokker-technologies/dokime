/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Artifacts Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};

use dokime_protocol::host::content::ArtifactsGetRequest;
use dokime_protocol::host::content::ArtifactsGetResponse;
use dokime_protocol::model::ArtifactMetadata;
use dokime_storage::ContentEntry;
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::FileReference;
use harmos_protocol::model::History;
use harmos_protocol::model::Position;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.Artifacts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ArtifactsGetRequest, ArtifactsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: ArtifactsGetRequest) -> Result<ArtifactsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = crate::host::views::read(context, &format!("artifact/{}", request.name), request.at.as_ref(), async {
            let stored = context.host.store().artifact(&request.name)?;
            Ok(ArtifactsGetResponse {
                metadata: stored.artifact,
                file: stored.request.and_then(|request| request.file),
                ..Default::default()
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

pub(crate) fn reference(context: &Context, entry: &ContentEntry) -> Result<FileReference> {
    let digest = (0..entry.sha256.len())
        .step_by(2)
        .map(|offset| {
            u8::from_str_radix(&entry.sha256[offset..offset + 2], 16)
                .map_err(|_| Error::Invalid("Read Content: Restore Its Stored Digest".into()))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(FileReference {
        metadata: Some(FileMetadata {
            id: entry.identifier.clone(),
            scope: Some(entry.execution.clone()),
            media_type: if entry.artifact {
                ArtifactMetadata::decode(entry.metadata.as_slice())?.content_type.unwrap_or_default()
            } else {
                "application/octet-stream".into()
            },
            expected_length: Some(entry.length),
            expected_sha256: Some(digest.clone()),
        }),
        length: entry.length,
        sha256: digest,
        position: Some(Position {
            history: Some(History {
                store: context.host.identity().backend_id,
                epoch: entry.execution.clone(),
            }),
            sequence: context.host.store().metadata(&entry.execution)?.frontier,
        }),
    })
}

/* === Tests === */

crate::native::unary!(ArtifactsGetRequest, ArtifactsGetResponse, context, "dokime.host.v1.content", "Artifacts", "Get");
