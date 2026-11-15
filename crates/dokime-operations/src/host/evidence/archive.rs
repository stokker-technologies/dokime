/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Evidence Archive

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use arrow_array::{Array, BinaryArray, StringArray, UInt64Array};
use harmos_protocol::model::{OperationContract, PageRequest};
use harmos_runtime::builtins::storage::{
    CreateSnapshot, CreateSnapshotRequest, DeleteSnapshot, DeleteSnapshotRequest, GetFileBytes, GetFileBytesRequest, ListRows,
    ListRowsRequest, Source,
};
use harmos_runtime::storage::query::FileSelection;
use std::collections::HashMap;
use std::io::Write;

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::EvidenceArchiveRequest;
use dokime_protocol::host::EvidenceArchiveResponse;

use dokime_protocol::host::evidence_subject::Kind as EvidenceSubjectKind;
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::FileReference;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Evidence/Archive";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<EvidenceArchiveRequest, EvidenceArchiveResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: EvidenceArchiveRequest) -> Result<EvidenceArchiveResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let key = format!(
            "archive/{}",
            crate::host::views::hex(&request.subject.as_ref().map(prost::Message::encode_to_vec).unwrap_or_default())
        );
        let (mut reply, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            let subject = request.subject.ok_or_else(|| Error::Invalid("Download Evidence: Select an Execution or Cycle".into()))?;
            let (store, _read_cancellation) = context.host.store().read_session()?;
            let mut file = tempfile::tempfile().map_err(|error| Error::Invalid(format!("Create Temporary Evidence Export: {error}")))?;
            match subject.kind.as_ref() {
                Some(EvidenceSubjectKind::Execution(execution)) => {
                    let files = native_files(context, execution).await?;
                    store.write_execution_archive_with_files(execution, &mut file, files)?;
                }
                Some(EvidenceSubjectKind::Cycle(cycle)) => {
                    let page = std::num::NonZeroUsize::new(context.settings.page_items).unwrap();
                    let mut files = HashMap::new();
                    let mut after = String::new();
                    loop {
                        let executions = store.cycle_execution_page(cycle, &after, page)?;
                        if executions.is_empty() {
                            break;
                        }
                        for execution in executions {
                            after = execution.clone();
                            files.insert(execution.clone(), native_files(context, &execution).await?);
                        }
                    }
                    store.write_cycle_archive_with_files(cycle, page, &mut file, &mut files)?;
                }
                None => return Err(Error::Invalid("Download Evidence: Select an Execution or Cycle".into())),
            }
            let (file, length, sha256) = crate::host::content::transfer::temporary(file)?;
            let digest =
                (0..sha256.len()).step_by(2).map(|index| u8::from_str_radix(&sha256[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
            let created = crate::host::files::uploads::create::handle(
                context,
                dokime_protocol::host::files::UploadsCreateRequest {
                    metadata: Some(FileMetadata {
                        id: format!("temporary-evidence/{}", uuid::Uuid::now_v7()),
                        scope: None,
                        media_type: "application/zip".into(),
                        expected_length: Some(length),
                        expected_sha256: Some(digest.clone()),
                    }),
                    request_id: uuid::Uuid::now_v7().to_string(),
                },
            )
            .await?;
            let upload = created.upload.ok_or_else(|| Error::Precondition("Publish Evidence Archive: Return Its Native Upload".into()))?;
            let result = async {
                let mut offset = 0;
                while offset < length {
                    let count = (length - offset).min(created.chunk_bytes) as usize;
                    let mut data = vec![0; count];
                    crate::host::content::transfer::read_temporary(&file, offset, &mut data)?;
                    crate::host::files::uploads::append::handle(
                        context,
                        dokime_protocol::host::files::UploadsAppendRequest {
                            upload: Some(upload.clone()),
                            offset,
                            data,
                            request_id: uuid::Uuid::now_v7().to_string(),
                        },
                    )
                    .await?;
                    offset += count as u64;
                }
                let published = crate::host::files::uploads::publish::handle(
                    context,
                    dokime_protocol::host::files::UploadsPublishRequest {
                        upload: Some(upload.clone()),
                        length,
                        sha256: digest,
                        request_id: uuid::Uuid::now_v7().to_string(),
                    },
                )
                .await?;
                published.upload.ok_or_else(|| Error::Precondition("Publish Evidence Archive: Return Its Native Reference".into()))
            }
            .await;
            let reference = match result {
                Ok(reference) => reference,
                Err(source) => {
                    let removed = crate::host::files::uploads::delete::handle(
                        context,
                        dokime_protocol::host::files::UploadsDeleteRequest {
                            upload: Some(upload),
                            request_id: uuid::Uuid::now_v7().to_string(),
                        },
                    )
                    .await;
                    if let Err(cleanup) = removed {
                        return Err(Error::Invalid(format!("Publish Evidence Archive: {source}; Remove Its Failed Upload: {cleanup}")));
                    }
                    return Err(source);
                }
            };
            Ok(EvidenceArchiveResponse { file: Some(reference), resolved_at: None })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

async fn native_files(context: &Context, execution: &str) -> Result<Vec<dokime_storage::ArchivedNativeFile>> {
    let saved = context.storage::<CreateSnapshot>(CreateSnapshotRequest { at: None }).await?;
    let result = async {
        let scope = harmos_runtime::Identifier::new(execution)
            .map_err(|source| Error::Invalid(format!("Export Native Files: Select a Valid Execution: {source}")))?;
        let mut files = Vec::new();
        let mut token = String::new();
        loop {
            let page = context
                .storage::<ListRows>(ListRowsRequest {
                    source: Some(Source::Files(FileSelection { scope: Some(scope.clone()), item: None })),
                    merge: None,
                    transforms: Vec::new(),
                    at: saved.resolved_at.clone(),
                    page: Some(PageRequest {
                        page_size: context.settings.page_items as u32,
                        page_token: token,
                        ..Default::default()
                    }),
                })
                .await?;
            for row in page.rows {
                let harmos_query::QueryOutput::Batch(bytes) = row else { continue };
                let decoded = harmos_query::arrow::ArrowBatch::from_ipc(bytes)
                    .map_err(|source| Error::Invalid(format!("Read Native File Inventory: {source}")))?;
                let batch = decoded.as_record_batch();
                let text = |name: &str| -> Result<&StringArray> {
                    batch
                        .column_by_name(name)
                        .and_then(|value| value.as_any().downcast_ref::<StringArray>())
                        .ok_or_else(|| Error::Precondition(format!("Read Native File Inventory: Restore Its {name} Column")))
                };
                let number = |name: &str| -> Result<&UInt64Array> {
                    batch
                        .column_by_name(name)
                        .and_then(|value| value.as_any().downcast_ref::<UInt64Array>())
                        .ok_or_else(|| Error::Precondition(format!("Read Native File Inventory: Restore Its {name} Column")))
                };
                let binary = |name: &str| -> Result<&BinaryArray> {
                    batch
                        .column_by_name(name)
                        .and_then(|value| value.as_any().downcast_ref::<BinaryArray>())
                        .ok_or_else(|| Error::Precondition(format!("Read Native File Inventory: Restore Its {name} Column")))
                };
                for index in 0..batch.num_rows() {
                    let expected_length = number("expected_length")?;
                    let expected_sha256 = binary("expected_sha256")?;
                    let reference = FileReference {
                        position: Some(harmos_protocol::model::Position {
                            history: Some(harmos_protocol::model::History {
                                store: text("store")?.value(index).into(),
                                epoch: text("epoch")?.value(index).into(),
                            }),
                            sequence: number("position")?.value(index),
                        }),
                        metadata: Some(FileMetadata {
                            id: text("item")?.value(index).into(),
                            scope: Some(execution.into()),
                            media_type: text("media_type")?.value(index).into(),
                            expected_length: (!expected_length.is_null(index)).then(|| expected_length.value(index)),
                            expected_sha256: (!expected_sha256.is_null(index)).then(|| expected_sha256.value(index).to_vec()),
                        }),
                        length: number("length")?.value(index),
                        sha256: binary("sha256")?.value(index).to_vec(),
                    };
                    let selected = harmos_runtime::storage::FileReference::try_from(reference.clone())
                        .map_err(|source| Error::Invalid(format!("Export Native File: Restore Its Reference: {source}")))?;
                    let mut output =
                        tempfile::NamedTempFile::new().map_err(|source| Error::Invalid(format!("Stage Native File Archive: {source}")))?;
                    let mut offset = 0;
                    while offset < reference.length {
                        let chunk = context
                            .storage::<GetFileBytes>(GetFileBytesRequest {
                                file: Some(selected.clone()),
                                offset,
                                max_bytes: crate::host::files::transfer::chunk_bytes(context).await? as u64,
                                at: saved.resolved_at.clone(),
                            })
                            .await?;
                        if chunk.data.is_empty() || chunk.data.len() as u64 > reference.length - offset {
                            return Err(Error::Precondition("Export Native File: Restore Its Complete Bounded Bytes".into()));
                        }
                        output.write_all(&chunk.data).map_err(|source| Error::Invalid(format!("Stage Native File Archive: {source}")))?;
                        offset += chunk.data.len() as u64;
                    }
                    files.push(dokime_storage::ArchivedNativeFile { reference, path: output.into_temp_path() });
                }
            }
            token = page.next_page_token;
            if token.is_empty() {
                break;
            }
        }
        Ok(files)
    }
    .await;
    let released = context.storage::<DeleteSnapshot>(DeleteSnapshotRequest { snapshot: saved.snapshot }).await;
    match (result, released) {
        (Err(source), Err(cleanup)) => Err(Error::Invalid(format!("Export Native Files: {source}; Release Its Snapshot: {cleanup}"))),
        (Err(source), Ok(_)) | (Ok(_), Err(source)) => Err(source),
        (Ok(files), Ok(_)) => Ok(files),
    }
}

/* === Tests === */

crate::native::unary!(EvidenceArchiveRequest, EvidenceArchiveResponse, context, "dokime.host.v1", "Evidence", "Archive");
