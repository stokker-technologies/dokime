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

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::model;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.Artifacts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::content::ArtifactsGetRequest, dokime_protocol::host::content::ArtifactsGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::content::ArtifactsGetRequest,
) -> Result<dokime_protocol::host::content::ArtifactsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = crate::host::views::read(context, &format!("artifact/{}", request.name), request.at.as_ref(), async {
            let entry = context.host.store().content(&request.name)?;
            if !entry.artifact {
                return Err(Error::Invalid("Get Artifact: Select an Artifact Identifier".into()));
            }
            Ok(dokime_protocol::host::content::ArtifactsGetResponse {
                metadata: Some(dokime_protocol::model::ArtifactMetadata::decode(entry.metadata.as_slice())?),
                file: Some(reference(context, &entry)?),
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

pub(crate) fn reference(context: &Context, entry: &dokime_storage::ContentEntry) -> Result<harmos_protocol::model::FileReference> {
    let digest = (0..entry.sha256.len())
        .step_by(2)
        .map(|offset| {
            u8::from_str_radix(&entry.sha256[offset..offset + 2], 16)
                .map_err(|_| Error::Invalid("Read Content: Restore Its Stored Digest".into()))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(harmos_protocol::model::FileReference {
        metadata: Some(harmos_protocol::model::FileMetadata {
            id: entry.identifier.clone(),
            scope: Some(entry.execution.clone()),
            media_type: if entry.artifact {
                dokime_protocol::model::ArtifactMetadata::decode(entry.metadata.as_slice())?.content_type.unwrap_or_default()
            } else {
                "application/octet-stream".into()
            },
            expected_length: Some(entry.length),
            expected_sha256: Some(digest.clone()),
        }),
        length: entry.length,
        sha256: digest,
        position: Some(harmos_protocol::model::Position {
            history: Some(harmos_protocol::model::History {
                store: context.host.identity().backend_id,
                epoch: entry.execution.clone(),
            }),
            sequence: context.host.store().metadata(&entry.execution)?.frontier,
        }),
    })
}

/* === Tests === */
