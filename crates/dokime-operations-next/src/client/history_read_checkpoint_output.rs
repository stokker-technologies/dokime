/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History Read Checkpoint Output

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.read_checkpoint_output";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryReadCheckpointOutputRequest, v1::CheckpointFileWindow>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HistoryReadCheckpointOutputRequest) -> Result<v1::CheckpointFileWindow> {
    super::complete(ID, context.settings.read_timeout, async {
        let (checkpoint, cut) = super::history::selected(context, request.checkpoint.as_ref())?;
        if request.relative_path.is_empty()
            || request.relative_path.starts_with('/')
            || request.relative_path.contains('\\')
            || request.relative_path.contains(':')
            || request.relative_path.split('/').any(|part| part == ".." || part == "." || part.is_empty())
        {
            return Err(Error::Invalid("Read Checkpoint Output: Select a Relative Output Name from GetCheckpointEvidence".into()));
        }
        let metadata = super::history::artifacts(context, &cut)?
            .into_iter()
            .find(|artifact| artifact.filename == request.relative_path)
            .ok_or_else(|| Error::NotFound("Read Checkpoint Output: Select an Output from GetCheckpointEvidence".into()))?;
        let limit = if request.max_bytes == 0 {
            context.settings.message_bytes / 2
        } else {
            (request.max_bytes as usize).min(context.settings.message_bytes / 2)
        };
        let mut data = Vec::new();
        let mut after = 0;
        let mut offset = 0u64;
        loop {
            let page = context.host.store().evidence_page(
                &cut.execution.identifier,
                after,
                cut.frontier,
                std::num::NonZeroUsize::new(context.settings.page_items).unwrap(),
            )?;
            if page.is_empty() {
                break;
            }
            after = page.last().unwrap().position;
            for entry in page {
                if entry.kind != "artifact" {
                    continue;
                }
                let chunk = v1::PublishArtifact::decode(entry.payload.as_slice())?;
                if chunk.transfer_id != metadata.upload_id {
                    continue;
                }
                let end = offset.saturating_add(chunk.data.len() as u64);
                if end > request.offset && data.len() < limit {
                    let start = request.offset.saturating_sub(offset) as usize;
                    let count = (limit - data.len()).min(chunk.data.len() - start);
                    data.extend_from_slice(&chunk.data[start..start + count]);
                }
                offset = end;
            }
            if data.len() == limit {
                break;
            }
        }
        super::history::bounded(
            context,
            v1::CheckpointFileWindow {
                checkpoint: Some(checkpoint),
                content: Some(v1::ExecutionFileWindow {
                    relative_path: request.relative_path,
                    content_type: metadata.content_type,
                    kind: v1::ExecutionFileKind::Binary as i32,
                    state: if metadata.state == v1::ArtifactState::Complete as i32 {
                        v1::ExecutionFileState::Finalized
                    } else {
                        v1::ExecutionFileState::Interrupted
                    } as i32,
                    offset: request.offset,
                    observed_size_bytes: metadata.size_bytes,
                    data,
                }),
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
