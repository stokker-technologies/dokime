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
use std::{collections::HashMap, future::Future, io::Write, num::NonZeroUsize, pin::Pin, time::Instant};

// Third Party Imports
use arrow_array::{Array, BinaryArray, StringArray, UInt64Array};
use harmos_operations::storage::{
    CreateSnapshot, CreateSnapshotRequest, DeleteSnapshot, DeleteSnapshotRequest, GetFileBytes, GetFileBytesRequest, ListRows,
    ListRowsRequest, RowOutput, Source,
};
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::FileReference;
use harmos_protocol::model::{History, PageRequest, Position, ReadCut};
use harmos_query::arrow::ArrowBatch;
use harmos_runtime::{
    Identifier,
    storage::{self, query::FileSelection},
};
use prost::Message;
use tempfile::NamedTempFile;

// Local Imports
use crate::host::storage::administration;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::evidence::{AnnotationOwner, AnnotationOwnerOwner, AnnotationSelection, annotation_owner_owner};
use dokime_protocol::host::evidence_subject;
use dokime_protocol::host::{EvidenceArchiveRequest, EvidenceArchiveResponse, EvidenceSubject};
use dokime_storage::persisted::Scope;
use dokime_storage::{ArchivedContent, ArchivedNativeFile, ExecutionState};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Evidence/Archive";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: EvidenceArchiveRequest) -> Result<EvidenceArchiveResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let subject = request.subject.ok_or_else(|| Error::Invalid("Download Evidence: Select an Execution or Cycle".into()))?;
        let scope = subject_scope(&subject)?;
        let key = format!("archive/{}", crate::host::views::hex(&subject.encode_to_vec()));
        let (mut reply, cut) = crate::host::views::read(context, scope, &key, request.at.as_ref(), async {
            if let Some(evidence_subject::Kind::Cycle(cycle)) = subject.kind.as_ref() {
                let saved = context
                    .host
                    .store()
                    .client_cycle(cycle)?
                    .ok_or_else(|| Error::NotFound("Download Evidence: Select an Existing Cycle".into()))?;
                if !saved.sealed {
                    return Err(Error::Precondition(format!("Download Evidence: Seal Cycle {cycle} and Await Its Completion First")));
                }
                let file = saved.evidence_file.ok_or_else(|| {
                    Error::Precondition(format!("Download Evidence: Restore the Retained Archive for Sealed Cycle {cycle}"))
                })?;
                return Ok(EvidenceArchiveResponse { file: Some(file), resolved_at: None });
            }
            let Some(evidence_subject::Kind::Execution(execution)) = subject.kind.as_ref() else {
                return Err(Error::Invalid("Download Evidence: Select an Execution".into()));
            };
            Ok(EvidenceArchiveResponse {
                file: Some(freeze(context, execution).await?),
                resolved_at: None,
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/// The Sealed Execution's Evidence Archive, Built Once After Its Frozen Report by Its Seal or First Export.
pub async fn freeze(context: &Context, execution: &str) -> Result<FileReference> {
    let cut = context.host.store().cut(execution)?;
    if cut.state != ExecutionState::Sealed {
        return Err(Error::Precondition(format!("Download Evidence: Wait until Execution {execution} Is Sealed")));
    }
    // The Report Is Published First, So Building the Archive Only Reads It.
    crate::host::reports::create::freeze(context, execution).await?;
    let key = format!("execution-evidence/{execution}/{}", crate::host::views::hex(&cut.seal));
    let (saved, revision) = administration::message::<FileReference>(context, &key).await?;
    if !revision.is_empty() {
        return Ok(saved);
    }
    // The Host Settles an Admitted Build After a Caller's Deadline Drops Its Receiver, Unless It Stops.
    let (owned, execution) = (context.clone(), execution.to_owned());
    let receiver = crate::host::stoppable_work(context, "Freeze Evidence", async move { build(&owned, &execution, &key).await })?;
    receiver.await.map_err(|_| Error::Cancelled("Freeze Evidence: Reconnect After the Host Stops".into()))?
}

pub(crate) fn materialize(
    context: &Context, subject: EvidenceSubject, report: Option<FileReference>, scope: Option<Identifier>,
) -> Pin<Box<dyn Future<Output = Result<FileReference>> + Send + '_>> {
    Box::pin(async move {
        let scope = match scope {
            Some(scope) => scope,
            None => subject_scope(&subject)?,
        };
        let phase = Instant::now();
        let mut file = tempfile::tempfile().map_err(|error| Error::Invalid(format!("Create Temporary Evidence Export: {error}")))?;
        // Each Frozen Report Is Published Before the Archive's Read Snapshot Opens.
        match subject.kind.as_ref() {
            Some(evidence_subject::Kind::Execution(execution)) => {
                let (_, report) = crate::host::reports::create::freeze(context, execution).await?;
                let cycle = context.host.store().metadata(execution)?.execution.cycle;
                let mut contents = HashMap::from([(execution.clone(), content(context, execution).await?)]);
                inventory(context, &cycle, &mut contents, &mut ArchivedContent::default()).await?;
                let mut files = contents.remove(execution).unwrap();
                files.report = Some(native_file(context, report, None).await?);
                let (store, _read_cancellation) = context.host.store().read_session()?;
                store.write_execution_archive(execution, &mut file, files)?;
            }
            Some(evidence_subject::Kind::Cycle(cycle)) => {
                let page = NonZeroUsize::new(context.settings.page_items).unwrap();
                let mut files = HashMap::new();
                let mut after = String::new();
                loop {
                    let executions = context.host.store().cycle_execution_page(cycle, &after, page)?;
                    if executions.is_empty() {
                        break;
                    }
                    for execution in executions {
                        after = execution.clone();
                        let mut content = content(context, &execution).await?;
                        let (_, report) = crate::host::reports::create::freeze(context, &execution).await?;
                        content.report = Some(native_file(context, report, None).await?);
                        files.insert(execution.clone(), content);
                    }
                }
                let mut owned = ArchivedContent {
                    files: native_files(context, Scope::Cycle(cycle).identifier()?).await?,
                    ..Default::default()
                };
                inventory(context, cycle, &mut files, &mut owned).await?;
                let report = report.ok_or_else(|| Error::Precondition("Seal Cycle: Materialize Its Report Before Its Archive".into()))?;
                owned.report = Some(native_file(context, report, None).await?);
                let (store, _read_cancellation) = context.host.store().read_session()?;
                store.write_cycle_archive(cycle, page, &mut file, &mut files, owned)?;
            }
            None => return Err(Error::Invalid("Download Evidence: Select an Execution or Cycle".into())),
        }
        tracing::debug!(phase = "archive_assembly", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Archive Stage");
        let phase = Instant::now();
        let (file, length, sha256) = crate::host::content::transfer::temporary(file)?;
        let digest =
            (0..sha256.len()).step_by(2).map(|index| u8::from_str_radix(&sha256[index..index + 2], 16).unwrap()).collect::<Vec<_>>();
        let request_id = uuid::Uuid::now_v7().to_string();
        let reference = crate::host::files::transfer::publication(
            context,
            "Publish Evidence Archive",
            &request_id,
            FileMetadata {
                id: format!("temporary-evidence/{request_id}"),
                scope: Some(scope.to_string()),
                media_type: "application/zip".into(),
                expected_length: Some(length),
                expected_sha256: Some(digest),
            },
            |offset, data| crate::host::content::transfer::read_temporary(&file, offset, data),
            || context.host.shutting_down(),
        )
        .await?;
        tracing::debug!(phase = "archive_upload", elapsed_ms = phase.elapsed().as_secs_f64() * 1000.0, "Archive Stage");
        Ok(reference)
    })
}

/* === Private Functions === */

async fn build(context: &Context, execution: &str, key: &str) -> Result<FileReference> {
    let _admitted = context.archives.lock().await;
    let (saved, revision) = administration::message::<FileReference>(context, key).await?;
    if !revision.is_empty() {
        return Ok(saved);
    }
    let started = Instant::now();
    let subject = EvidenceSubject {
        kind: Some(evidence_subject::Kind::Execution(execution.into())),
    };
    let file = materialize(context, subject, None, None).await?;
    let file = administration::retain(context, key, Scope::Execution(execution).identifier()?, &file).await?;
    tracing::info!(%execution, creation_ms = started.elapsed().as_secs_f64() * 1000.0, "Evidence Frozen");
    Ok(file)
}

fn subject_scope(subject: &EvidenceSubject) -> Result<Identifier> {
    Ok(match subject.kind.as_ref() {
        Some(evidence_subject::Kind::Execution(execution)) => Scope::Execution(execution),
        Some(evidence_subject::Kind::Cycle(cycle)) => Scope::Cycle(cycle),
        None => return Err(Error::Invalid("Download Evidence: Select an Execution or Cycle".into())),
    }
    .identifier()?)
}

async fn content(context: &Context, execution: &str) -> Result<ArchivedContent> {
    let execution_files = crate::host::content::execution_files(context, execution).await?;
    let files = native_files(context, Scope::Execution(execution).identifier()?).await?;
    let snapshot = crate::host::execution_state::read(context, execution)?;
    let template_label = crate::host::execution_state::template_metadata(&snapshot)?.map(|(_, template)| template.label);
    Ok(ArchivedContent {
        template_label,
        files,
        execution_files,
        ..Default::default()
    })
}

async fn inventory(
    context: &Context, cycle: &str, executions: &mut HashMap<String, ArchivedContent>, owned: &mut ArchivedContent,
) -> Result<()> {
    for payload in crate::host::content::artifact_payloads(context).await? {
        let entry = crate::host::content::decode_artifact(&payload)?;
        let scope = entry.artifact.as_ref().and_then(|artifact| artifact.scope.as_ref());
        let Some(scope) = scope else {
            continue;
        };
        if let Some(execution) = scope.execution_id.as_ref() {
            if let Some(content) = executions.get_mut(execution) {
                content.artifacts.push(payload);
            }
        } else if scope.cycle_id.as_deref() == Some(cycle) {
            owned.artifacts.push(payload);
        }
    }
    let selection = AnnotationSelection {
        owner: Some(AnnotationOwner {
            owner: Some(AnnotationOwnerOwner {
                kind: Some(annotation_owner_owner::Kind::CycleId(cycle.into())),
            }),
        }),
        include_context: false,
    };
    for note in crate::host::evidence::notes::list::capture(context, Some(selection)).await?.into_iter().flat_map(|page| page.notes) {
        let execution = note.scope.as_ref().and_then(|scope| scope.execution_id.as_ref());
        if let Some(execution) = execution {
            if let Some(content) = executions.get_mut(execution) {
                content.notes.push(note);
            }
        } else {
            owned.notes.push(note);
        }
    }
    Ok(())
}

async fn native_files(context: &Context, scope: Identifier) -> Result<Vec<ArchivedNativeFile>> {
    let saved = context.storage::<CreateSnapshot>(CreateSnapshotRequest { at: None }).await?;
    let result = async {
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
                    if item.starts_with("view/") || item.starts_with("temporary-evidence/") || item.starts_with("prepared-report/") {
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
                    files.push(native_file(context, reference, saved.resolved_at.clone()).await?);
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

async fn native_file(context: &Context, reference: FileReference, at: Option<ReadCut>) -> Result<ArchivedNativeFile> {
    let snapshot = if at.is_none() { Some(context.storage::<CreateSnapshot>(CreateSnapshotRequest { at: None }).await?) } else { None };
    let at = snapshot.as_ref().and_then(|snapshot| snapshot.resolved_at.clone()).or(at);
    let result = async {
        let selected = storage::FileReference::try_from(reference.clone())
            .map_err(|source| Error::Invalid(format!("Export Native File: Restore Its Reference: {source}")))?;
        let mut output = NamedTempFile::new().map_err(|source| Error::Invalid(format!("Stage Native File Archive: {source}")))?;
        let mut offset = 0;
        let max_bytes = crate::host::files::transfer::chunk_bytes(context).await? as u64;
        while offset < reference.length {
            let chunk = context
                .storage::<GetFileBytes>(GetFileBytesRequest {
                    file: Some(selected.clone()),
                    offset,
                    max_bytes,
                    at: at.clone(),
                })
                .await?;
            if chunk.data.is_empty() || chunk.data.len() as u64 > reference.length - offset {
                return Err(Error::Precondition("Export Native File: Restore Its Complete Bounded Bytes".into()));
            }
            output.write_all(&chunk.data).map_err(|source| Error::Invalid(format!("Stage Native File Archive: {source}")))?;
            offset += chunk.data.len() as u64;
        }
        Ok(ArchivedNativeFile { reference, path: output.into_temp_path() })
    }
    .await;
    let released = match snapshot {
        Some(snapshot) => context.storage::<DeleteSnapshot>(DeleteSnapshotRequest { snapshot: snapshot.snapshot }).await.map(|_| ()),
        None => Ok(()),
    };
    match (result, released) {
        (Err(source), Err(cleanup)) => Err(Error::Invalid(format!("Export Native File: {source}; Release Its Snapshot: {cleanup}"))),
        (Err(source), Ok(())) | (Ok(_), Err(source)) => Err(source),
        (Ok(file), Ok(())) => Ok(file),
    }
}

/* === Tests === */

crate::native::unary!(EvidenceArchiveRequest, EvidenceArchiveResponse, context, "dokime.host.v1", "Evidence", "Archive");
