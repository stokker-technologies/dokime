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
use std::{collections::HashMap, io::Write, num::NonZeroUsize};

// Third Party Imports
use arrow_array::{Array, BinaryArray, StringArray, UInt64Array};
use harmos_operations::storage::{
    CreateSnapshot, CreateSnapshotRequest, DeleteSnapshot, DeleteSnapshotRequest, GetFileBytes, GetFileBytesRequest, ListRows,
    ListRowsRequest, RowOutput, Source,
};
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::FileReference;
use harmos_protocol::model::{History, OperationContract, PageRequest, Position};
use harmos_query::arrow::ArrowBatch;
use harmos_runtime::storage::{self, query::FileSelection};
use prost::Message;
use tempfile::NamedTempFile;

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::evidence_subject;
use dokime_protocol::host::files::{UploadsAppendRequest, UploadsCreateRequest, UploadsDeleteRequest, UploadsPublishRequest};
use dokime_protocol::host::{EvidenceArchiveRequest, EvidenceArchiveResponse};
use dokime_storage::persisted::Scope;
use dokime_storage::{ArchivedContent, ArchivedNativeFile};

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
        let scope = match request.subject.as_ref().and_then(|subject| subject.kind.as_ref()) {
            Some(evidence_subject::Kind::Execution(execution)) => Scope::Execution(execution),
            Some(evidence_subject::Kind::Cycle(cycle)) => Scope::Cycle(cycle),
            None => return Err(Error::Invalid("Download Evidence: Select an Execution or Cycle".into())),
        }
        .identifier()?;
        let key = format!("archive/{}", crate::host::views::hex(&request.subject.as_ref().map(Message::encode_to_vec).unwrap_or_default()));
        let (mut reply, cut) = crate::host::views::read(context, scope.clone(), &key, request.at.as_ref(), async {
            let subject = request.subject.ok_or_else(|| Error::Invalid("Download Evidence: Select an Execution or Cycle".into()))?;
            let (store, _read_cancellation) = context.host.store().read_session()?;
            let mut file = tempfile::tempfile().map_err(|error| Error::Invalid(format!("Create Temporary Evidence Export: {error}")))?;
            match subject.kind.as_ref() {
                Some(evidence_subject::Kind::Execution(execution)) => {
                    let files = content(context, execution).await?;
                    store.write_execution_archive(execution, &mut file, files)?;
                }
                Some(evidence_subject::Kind::Cycle(cycle)) => {
                    let page = NonZeroUsize::new(context.settings.page_items).unwrap();
                    let mut files = HashMap::new();
                    let mut after = String::new();
                    loop {
                        let executions = store.cycle_execution_page(cycle, &after, page)?;
                        if executions.is_empty() {
                            break;
                        }
                        for execution in executions {
                            after = execution.clone();
                            files.insert(execution.clone(), content(context, &execution).await?);
                        }
                    }
                    store.write_cycle_archive(cycle, page, &mut file, &mut files)?;
                }
                None => return Err(Error::Invalid("Download Evidence: Select an Execution or Cycle".into())),
            }
            let (file, length, sha256) = crate::host::content::transfer::temporary(file)?;
            let digest =
                (0..sha256.len()).step_by(2).map(|index| u8::from_str_radix(&sha256[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
            let created = crate::host::files::uploads::create::handle(
                context,
                UploadsCreateRequest {
                    metadata: Some(FileMetadata {
                        id: format!("temporary-evidence/{}", uuid::Uuid::now_v7()),
                        scope: Some(scope.to_string()),
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
                        UploadsAppendRequest {
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
                    UploadsPublishRequest {
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
                        UploadsDeleteRequest {
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

async fn content(context: &Context, execution: &str) -> Result<ArchivedContent> {
    let mut artifacts = Vec::new();
    for payload in crate::host::content::artifact_payloads(context).await? {
        let entry = crate::host::content::decode_artifact(&payload)?;
        let owner = entry.artifact.as_ref().and_then(|artifact| artifact.scope.as_ref()).and_then(|scope| scope.execution_id.as_deref());
        if owner == Some(execution) {
            artifacts.push(payload);
        }
    }
    let execution_files = crate::host::content::execution_files(context, execution).await?;
    let files = native_files(context, execution).await?;
    Ok(ArchivedContent { files, artifacts, execution_files })
}

async fn native_files(context: &Context, execution: &str) -> Result<Vec<ArchivedNativeFile>> {
    let saved = context.storage::<CreateSnapshot>(CreateSnapshotRequest { at: None }).await?;
    let result = async {
        let scope = Scope::Execution(execution).identifier()?;
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
                let RowOutput::Batch(bytes) = row else { continue };
                let decoded =
                    ArrowBatch::from_ipc(bytes).map_err(|source| Error::Invalid(format!("Read Native File Inventory: {source}")))?;
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
                    // Derived Views and Exports Are Not Source Evidence.
                    let item = text("item")?.value(index);
                    if item.starts_with("view/") || item.starts_with("temporary-evidence/") {
                        continue;
                    }
                    let expected_length = number("expected_length")?;
                    let expected_sha256 = binary("expected_sha256")?;
                    let reference = FileReference {
                        position: Some(Position {
                            history: Some(History {
                                store: text("store")?.value(index).into(),
                                epoch: text("epoch")?.value(index).into(),
                            }),
                            sequence: number("position")?.value(index),
                        }),
                        metadata: Some(FileMetadata {
                            id: item.into(),
                            scope: Some(scope.to_string()),
                            media_type: text("media_type")?.value(index).into(),
                            expected_length: (!expected_length.is_null(index)).then(|| expected_length.value(index)),
                            expected_sha256: (!expected_sha256.is_null(index)).then(|| expected_sha256.value(index).to_vec()),
                        }),
                        length: number("length")?.value(index),
                        sha256: binary("sha256")?.value(index).to_vec(),
                    };
                    let selected = storage::FileReference::try_from(reference.clone())
                        .map_err(|source| Error::Invalid(format!("Export Native File: Restore Its Reference: {source}")))?;
                    let mut output =
                        NamedTempFile::new().map_err(|source| Error::Invalid(format!("Stage Native File Archive: {source}")))?;
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
                    files.push(ArchivedNativeFile { reference, path: output.into_temp_path() });
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
