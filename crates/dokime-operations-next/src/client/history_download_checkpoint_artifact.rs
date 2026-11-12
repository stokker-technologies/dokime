/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History Download Checkpoint Artifact

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{
    Result,
    client::{Context, Output},
};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.download_checkpoint_artifact";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryDownloadCheckpointArtifactRequest, v1::HistoryDownloadCheckpointArtifactOutput>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: v1::HistoryDownloadCheckpointArtifactRequest,
) -> Result<Output<v1::HistoryDownloadCheckpointArtifactOutput>> {
    let (_, cut) = super::history::selected(context, request.checkpoint.as_ref())?;
    let metadata = super::history::artifact(context, &cut, &request.artifact_id)?;
    let (sender, receiver) = tokio::sync::mpsc::channel(context.settings.stream_buffer.saturating_add(1));
    let terminal =
        sender.clone().reserve_owned().await.map_err(|_| crate::Error::Invalid("Open Watch: Reconnect the Client Stream".into()))?;
    let context = context.clone();
    tokio::spawn(async move {
        let work = async {
            let frame = v1::ArtifactFrame {
                payload: Some(v1::ArtifactFramePayload {
                    kind: Some(v1::artifact_frame_payload::Kind::Metadata(metadata.clone())),
                }),
            };
            super::history::bounded(&context, frame.clone())?;
            if sender
                .send(Ok(v1::HistoryDownloadCheckpointArtifactOutput {
                    value: Some(v1::history_download_checkpoint_artifact_output::Value::Item(frame)),
                }))
                .await
                .is_err()
            {
                return Ok(());
            }
            let mut after = 0;
            let mut offset = 0;
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
                    for data in chunk.data.chunks(context.settings.message_bytes / 2) {
                        let frame = v1::ArtifactFrame {
                            payload: Some(v1::ArtifactFramePayload {
                                kind: Some(v1::artifact_frame_payload::Kind::Chunk(v1::ArtifactChunk { offset, data: data.to_vec() })),
                            }),
                        };
                        offset += data.len() as u64;
                        if sender
                            .send(Ok(v1::HistoryDownloadCheckpointArtifactOutput {
                                value: Some(v1::history_download_checkpoint_artifact_output::Value::Item(frame)),
                            }))
                            .await
                            .is_err()
                        {
                            return Ok(());
                        }
                    }
                }
            }
            if offset != metadata.size_bytes {
                return Err(crate::Error::Conflict(
                    "Download Checkpoint Artifact: Restore the Complete Sealed Evidence Before Retrying".into(),
                ));
            }
            let complete = v1::ArtifactCompletion {
                size_bytes: metadata.size_bytes,
                sha256: metadata.sha256,
            };
            let frame = v1::ArtifactFrame {
                payload: Some(v1::ArtifactFramePayload {
                    kind: Some(v1::artifact_frame_payload::Kind::Complete(complete)),
                }),
            };
            if sender
                .send(Ok(v1::HistoryDownloadCheckpointArtifactOutput {
                    value: Some(v1::history_download_checkpoint_artifact_output::Value::Item(frame)),
                }))
                .await
                .is_ok()
            {
                let _ = sender
                    .send(Ok(v1::HistoryDownloadCheckpointArtifactOutput {
                        value: Some(v1::history_download_checkpoint_artifact_output::Value::Response(v1::ClientEmpty {})),
                    }))
                    .await;
            }
            Ok::<_, crate::Error>(())
        };
        match tokio::time::timeout(context.settings.read_timeout, work).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let _ = sender.send(Err(super::status(error))).await;
            }
            Err(_) => {
                terminal.send(Err(tonic::Status::deadline_exceeded("Download Checkpoint Artifact: Retry Within the Read Deadline")));
            }
        }
    });
    Ok(tokio_stream::wrappers::ReceiverStream::new(receiver))
}

/* === Private Functions === */

/* === Tests === */
