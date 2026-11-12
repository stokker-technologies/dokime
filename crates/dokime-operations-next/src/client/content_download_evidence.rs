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

/* === Constants === */

pub const ID: &str = "dokime.content.download_evidence";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ContentDownloadEvidenceRequest, v1::ContentDownloadEvidenceOutput>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: v1::ContentDownloadEvidenceRequest,
) -> Result<super::Output<v1::ContentDownloadEvidenceOutput>> {
    super::complete(ID, context.settings.read_timeout, async {
        let subject = request.subject.ok_or_else(|| Error::Invalid("Download Evidence: Select an Execution or Cycle".into()))?;
        let store = context.host.store();
        let mut file = tempfile::tempfile().map_err(|error| Error::Invalid(format!("Create Temporary Evidence Export: {error}")))?;
        match subject.kind.as_ref() {
            Some(v1::evidence_subject::Kind::Execution(execution)) => store.write_execution_archive(execution, &mut file)?,
            Some(v1::evidence_subject::Kind::Cycle(cycle)) => {
                let ids = store.identifiers()?;
                let mut count = 0;
                for id in ids {
                    if store.metadata(&id)?.execution.cycle != *cycle {
                        continue;
                    }
                    let mut execution =
                        tempfile::tempfile().map_err(|error| Error::Invalid(format!("Create Temporary Execution Export: {error}")))?;
                    store.write_execution_archive(&id, &mut execution)?;
                    use std::io::{Seek, Write};
                    let length = execution.stream_position().map_err(|error| Error::Invalid(error.to_string()))?;
                    let mut prefix = Vec::new();
                    prost::encoding::encode_key(1, prost::encoding::WireType::LengthDelimited, &mut prefix);
                    prost::encoding::encode_varint(length, &mut prefix);
                    file.write_all(&prefix).map_err(|error| Error::Invalid(error.to_string()))?;
                    execution.rewind().map_err(|error| Error::Invalid(error.to_string()))?;
                    std::io::copy(&mut execution, &mut file).map_err(|error| Error::Invalid(error.to_string()))?;
                    count += 1;
                }
                if count == 0 {
                    return Err(Error::Invalid("Download Evidence: Select an Existing Cycle".into()));
                }
            }
            None => return Err(Error::Invalid("Download Evidence: Select an Execution or Cycle".into())),
        }
        let (file, length, sha256) = super::content_transfer::temporary(file)?;
        let source = super::content_transfer::Source::Temporary(file);
        let metadata = v1::EvidenceExportMetadata {
            subject: Some(subject),
            filename: "evidence.pb".into(),
            schema_version: 1,
            size_bytes: length,
            sha256: sha256.clone(),
        };
        let digest = Some(sha256);
        super::content_transfer::download(
            context,
            super::content_transfer::Download { source, length, expected_digest: digest },
            v1::ContentDownloadEvidenceOutput {
                value: Some(v1::content_download_evidence_output::Value::Item(v1::EvidenceExportFrame {
                    payload: Some(v1::EvidenceExportPayload {
                        kind: Some(v1::evidence_export_payload::Kind::Metadata(metadata)),
                    }),
                })),
            },
            |chunk| v1::ContentDownloadEvidenceOutput {
                value: Some(v1::content_download_evidence_output::Value::Item(v1::EvidenceExportFrame {
                    payload: Some(v1::EvidenceExportPayload {
                        kind: Some(v1::evidence_export_payload::Kind::Chunk(chunk)),
                    }),
                })),
            },
            |complete| v1::ContentDownloadEvidenceOutput {
                value: Some(v1::content_download_evidence_output::Value::Item(v1::EvidenceExportFrame {
                    payload: Some(v1::EvidenceExportPayload {
                        kind: Some(v1::evidence_export_payload::Kind::Complete(complete)),
                    }),
                })),
            },
            || v1::ContentDownloadEvidenceOutput {
                value: Some(v1::content_download_evidence_output::Value::Response(v1::ClientEmpty {})),
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
