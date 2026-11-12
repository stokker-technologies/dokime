/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use prost::Message;

/* === Constants === */

pub const ID: &str = "dokime.content.download_artifact";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ContentDownloadArtifactRequest, v1::ContentDownloadArtifactOutput>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: v1::ContentDownloadArtifactRequest,
) -> Result<super::Output<v1::ContentDownloadArtifactOutput>> {
    super::complete(ID, context.settings.read_timeout, async {
        if let Some((file, reference)) = context.host.temporary_report(&request.artifact_id)? {
            let (file, length, digest) = super::content_transfer::temporary(file)?;
            let metadata = v1::ArtifactMetadata {
                artifact_id: request.artifact_id.clone(),
                filename: "report.html".into(),
                content_type: Some("text/html".into()),
                size_bytes: length,
                sha256: digest.clone(),
                state: v1::ArtifactState::Complete as i32,
                ..Default::default()
            };
            let expected = reference.sha256.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            if digest != expected {
                return Err(Error::Invalid("Download Temporary Report: Materialize Its Fixed Snapshot Again".into()));
            }
            return super::content_transfer::download(
                context,
                super::content_transfer::Download {
                    source: super::content_transfer::Source::Temporary(file),
                    length,
                    expected_digest: Some(digest),
                },
                v1::ContentDownloadArtifactOutput {
                    value: Some(v1::content_download_artifact_output::Value::Item(v1::ArtifactFrame {
                        payload: Some(v1::ArtifactFramePayload {
                            kind: Some(v1::artifact_frame_payload::Kind::Metadata(metadata)),
                        }),
                    })),
                },
                |chunk| v1::ContentDownloadArtifactOutput {
                    value: Some(v1::content_download_artifact_output::Value::Item(v1::ArtifactFrame {
                        payload: Some(v1::ArtifactFramePayload {
                            kind: Some(v1::artifact_frame_payload::Kind::Chunk(chunk)),
                        }),
                    })),
                },
                |complete| v1::ContentDownloadArtifactOutput {
                    value: Some(v1::content_download_artifact_output::Value::Item(v1::ArtifactFrame {
                        payload: Some(v1::ArtifactFramePayload {
                            kind: Some(v1::artifact_frame_payload::Kind::Complete(complete)),
                        }),
                    })),
                },
                || v1::ContentDownloadArtifactOutput {
                    value: Some(v1::content_download_artifact_output::Value::Response(v1::ClientEmpty {})),
                },
            );
        }
        let store = context.host.store();
        let entry = store.content(&request.artifact_id)?;
        let metadata = v1::ArtifactMetadata::decode(entry.metadata.as_slice())?;
        if metadata.artifact_id != request.artifact_id || metadata.state != v1::ArtifactState::Complete as i32 {
            return Err(Error::Invalid("Download Artifact: Select a Completed Artifact".into()));
        }
        let source = super::content_transfer::Source::Stored { store, identifier: entry.identifier };
        let length = entry.length;
        let digest = Some(entry.sha256);
        super::content_transfer::download(
            context,
            super::content_transfer::Download { source, length, expected_digest: digest },
            v1::ContentDownloadArtifactOutput {
                value: Some(v1::content_download_artifact_output::Value::Item(v1::ArtifactFrame {
                    payload: Some(v1::ArtifactFramePayload {
                        kind: Some(v1::artifact_frame_payload::Kind::Metadata(metadata)),
                    }),
                })),
            },
            |chunk| v1::ContentDownloadArtifactOutput {
                value: Some(v1::content_download_artifact_output::Value::Item(v1::ArtifactFrame {
                    payload: Some(v1::ArtifactFramePayload {
                        kind: Some(v1::artifact_frame_payload::Kind::Chunk(chunk)),
                    }),
                })),
            },
            |complete| v1::ContentDownloadArtifactOutput {
                value: Some(v1::content_download_artifact_output::Value::Item(v1::ArtifactFrame {
                    payload: Some(v1::ArtifactFramePayload {
                        kind: Some(v1::artifact_frame_payload::Kind::Complete(complete)),
                    }),
                })),
            },
            || v1::ContentDownloadArtifactOutput {
                value: Some(v1::content_download_artifact_output::Value::Response(v1::ClientEmpty {})),
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
