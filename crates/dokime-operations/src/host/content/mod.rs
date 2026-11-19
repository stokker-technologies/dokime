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

pub mod artifacts;
pub mod execution_files;
pub mod transfer;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use arrow_array::{Array, BinaryArray, StringArray, StructArray};
use harmos_operations::storage::{
    CompoundOutcome, ListRows, ListRowsRequest, Mutate, MutateRequest, MutationId, MutationItem, RowOutput, SchemaInput, SetObject,
    SetObjectRequest, Source, set_object_response,
};
use harmos_protocol::model::{FileReference, PageRequest, Receipt};
use harmos_query::arrow::ArrowBatch;
use harmos_query::scalar::ScalarValue;
use harmos_query::transform::{
    Transform,
    expression::Contains,
    filter::{Clause, Compare, Condition, Filter, Term},
};
use harmos_runtime::Identifier;
use harmos_runtime::storage::{
    ObjectDefinition,
    query::{FileSelection, ObjectSelection, ObjectVersions},
};
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use crate::{
    Error, Result,
    host::{Context, files, storage::administration},
};
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::content::{ArtifactsCreateRequest, ArtifactsCreateResponse, ExecutionFileMetadata, ExecutionFileState};
use dokime_protocol::model::{ArtifactMetadata, ArtifactScope, ArtifactState};
use dokime_storage::persisted::{self, Administration, Scope};
use dokime_storage::{ExecutionState, Store, StoredArtifact, StoredExecutionFile};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Message)]
struct ArtifactCommit {
    #[prost(bytes = "vec", tag = "1")]
    stored: Vec<u8>,

    #[prost(string, tag = "2")]
    window: String,

    #[prost(string, tag = "3")]
    mutation: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn validate_store(store: &Store) -> Result<()> {
    if store.has_incompatible_content()? {
        return Err(Error::Precondition(
            "Open Content Storage: Select a Fresh Storage Root; This Root Uses Incompatible File Objects".into(),
        ));
    }
    Ok(())
}

pub async fn validate_root(context: &Context) -> Result<()> {
    let (entries, _) = administration::scan(context, "artifact/", None).await?;
    if !entries.is_empty() {
        return Err(Error::Precondition(
            "Open Artifact Storage: Select a Fresh Storage Root; This Root Uses Incompatible Artifact Objects".into(),
        ));
    }
    Ok(())
}

pub(crate) async fn retained(context: &Context, file: &FileReference) -> Result<bool> {
    let metadata = file.metadata.as_ref().ok_or_else(|| Error::Invalid("Delete File: Supply Its Stored Identifier".into()))?;
    if let Some((kind, owner)) = metadata.scope.as_deref().and_then(|scope| scope.split_once('/')) {
        if kind == "cycle"
            && context
                .host
                .store()
                .client_cycle(owner)?
                .is_some_and(|cycle| cycle.sealed || cycle.phase == dokime_protocol::host::CyclePhase::Sealing as i32)
        {
            return Ok(true);
        }
        if kind == "execution" && context.host.store().metadata(owner)?.state != ExecutionState::Open {
            return Ok(true);
        }
    }
    let item = Identifier::new(metadata.id.clone()).map_err(|source| Error::Invalid(format!("Delete File: {source}")))?;
    let mut token = String::new();
    let mut at = None;
    let mut execution_owned = false;
    loop {
        let response = context
            .storage::<ListRows>(ListRowsRequest {
                source: Some(Source::Files(FileSelection { scope: None, item: Some(item.clone()) })),
                merge: None,
                transforms: Vec::new(),
                at,
                page: Some(PageRequest {
                    page_size: context.settings.page_items as u32,
                    page_token: token,
                    ..Default::default()
                }),
            })
            .await?;
        at = response.resolved_at;
        for row in response.rows {
            let RowOutput::Batch(bytes) = row else { continue };
            let page = ArrowBatch::from_ipc(bytes).map_err(|source| Error::Invalid(format!("Delete File: {source}")))?;
            let scopes = page
                .as_record_batch()
                .column_by_name("scope")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| Error::Precondition("Delete File: Restore Its Stored Scope".into()))?;
            execution_owned |= scopes.iter().flatten().any(|scope| scope.starts_with("execution/") || scope.starts_with("cycle/"));
        }
        token = response.next_page_token;
        if token.is_empty() {
            break;
        }
    }
    // Execution and Cycle Files Can Be Attached to Evidence.
    if !execution_owned {
        return Ok(false);
    }
    let schema =
        ObjectDefinition::of::<Administration>().schema().map_err(|source| Error::Invalid(format!("Check Retained Files: {source}")))?;
    let mut token = String::new();
    let mut at = None;
    let mut retained = false;
    loop {
        let response = context
            .storage::<ListRows>(ListRowsRequest {
                source: Some(Source::Objects(ObjectSelection {
                    schema: Some(schema.clone()),
                    key: None,
                    versions: ObjectVersions::Revisions,
                })),
                merge: None,
                transforms: vec![
                    Transform::Contains(Contains {
                        field: "payload.key".into(),
                        name: "retained_content".into(),
                        value: ScalarValue::Text("content/".into()),
                    }),
                    Transform::Filter(Filter {
                        clauses: vec![Clause {
                            terms: vec![Term {
                                field: "retained_content".into(),
                                condition: Condition::Compare {
                                    op: Compare::Eq,
                                    value: ScalarValue::Bool(true),
                                },
                                negate: false,
                            }],
                        }],
                    }),
                ],
                at,
                page: Some(PageRequest {
                    page_size: context.settings.page_items as u32,
                    page_token: token,
                    ..Default::default()
                }),
            })
            .await?;
        at = response.resolved_at;
        for row in response.rows {
            let RowOutput::Batch(bytes) = row else { continue };
            let page = ArrowBatch::from_ipc(bytes).map_err(|source| Error::Invalid(format!("Check Retained Files: {source}")))?;
            let payload = page
                .as_record_batch()
                .column_by_name("payload")
                .and_then(|column| column.as_any().downcast_ref::<StructArray>())
                .ok_or_else(|| Error::Precondition("Check Retained Files: Restore Its Administration Payload".into()))?;
            let keys = payload
                .column_by_name("key")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| Error::Precondition("Check Retained Files: Restore Its Administration Keys".into()))?;
            let values = payload
                .column_by_name("payload")
                .and_then(|column| column.as_any().downcast_ref::<BinaryArray>())
                .ok_or_else(|| Error::Precondition("Check Retained Files: Restore Its Administration Values".into()))?;
            for index in 0..keys.len() {
                if keys.is_null(index) || values.is_null(index) {
                    return Err(Error::Precondition("Check Retained Files: Restore Its Complete Administration Object".into()));
                }
                let reference = if keys.value(index).starts_with("content/execution-file/") {
                    StoredExecutionFile::decode(values.value(index))?.file
                } else if keys.value(index).starts_with("content/artifact/") {
                    let saved = ArtifactCommit::decode(values.value(index))?;
                    decode_artifact(&saved.stored)?.request.and_then(|request| request.file)
                } else {
                    continue;
                };
                let reference =
                    reference.ok_or_else(|| Error::Precondition("Delete File: Restore Its Retained Content Reference".into()))?;
                if same_file(&reference, file) {
                    retained = true;
                }
            }
        }
        token = response.next_page_token;
        if token.is_empty() {
            return Ok(retained);
        }
    }
}

pub async fn artifact(context: &Context, identifier: &str) -> Result<StoredArtifact> {
    let (saved, revision) = administration::message::<ArtifactCommit>(context, &format!("content/artifact/{identifier}")).await?;
    if revision.is_empty() {
        return Err(Error::NotFound("Read Artifact: Select a Committed Artifact Identifier".into()));
    }
    decode_artifact(&saved.stored)
}

pub async fn artifact_records(context: &Context) -> Result<Vec<StoredArtifact>> {
    artifact_payloads(context).await?.into_iter().map(|payload| decode_artifact(&payload)).collect()
}

pub(crate) async fn artifact_payloads(context: &Context) -> Result<Vec<Vec<u8>>> {
    let (entries, _) = administration::scan(context, "content/artifact/", None).await?;
    entries
        .into_iter()
        .map(|entry| {
            let saved = ArtifactCommit::decode(entry.payload.as_slice())?;
            Ok(saved.stored)
        })
        .collect()
}

pub async fn attach_artifact(context: &Context, mut request: ArtifactsCreateRequest) -> Result<ArtifactsCreateResponse> {
    if let Some(selected) = request.scope.as_mut()
        && let Some(execution) = selected.execution_id.as_deref()
    {
        let cycle = context.host.store().metadata(execution)?.execution.cycle;
        if selected.cycle_id.as_ref().is_some_and(|selected| selected != &cycle) {
            return Err(Error::Invalid(format!("Create Artifact: Select Execution {execution} in Its Own Cycle {cycle}")));
        }
        selected.cycle_id = Some(cycle);
    }
    let selected = request.scope.as_ref().ok_or_else(|| Error::Invalid("Create Artifact: Select Its Execution or Cycle Scope".into()))?;
    let scope = artifact_scope(selected)?;
    let reference = request.file.as_ref().ok_or_else(|| Error::Invalid("Create Artifact: Select Published Content".into()))?;
    let file = reference.metadata.as_ref().ok_or_else(|| Error::Invalid("Create Artifact: Select a Named File Reference".into()))?;
    if file.scope.as_deref() != Some(scope.as_str()) {
        return Err(Error::Invalid("Create Artifact: Keep Its Native File Within the Selected Evidence Scope".into()));
    }
    let operation = artifacts::create::ID;
    let identifier = crate::host::views::hex(&Sha256::digest(format!("{operation}/{scope}/{}", request.request_id)));
    let key = format!("content/artifact/{identifier}");
    let (mut saved, revision) = administration::message::<ArtifactCommit>(context, &key).await?;
    if revision.is_empty() {
        context.host.store().require_evidence_owner(selected.cycle_id.as_deref(), selected.execution_id.as_deref(), "Create Artifact")?;
        let mutation = files::transfer::mutation(context, &key, &request.request_id).await?;
        let artifact = ArtifactMetadata {
            artifact_id: identifier,
            upload_id: file.id.clone(),
            scope: request.scope.clone(),
            filename: request.filename.clone(),
            content_type: Some(request.content_type.clone()),
            size_bytes: reference.length,
            sha256: crate::host::views::hex(&reference.sha256),
            state: ArtifactState::Complete as i32,
            created_at: Some(crate::host::projection::timestamp(context.host.now_ns())),
            ..Default::default()
        };
        saved = ArtifactCommit {
            stored: StoredArtifact {
                request: Some(request.clone()),
                artifact: Some(artifact),
            }
            .encode_to_vec(),
            window: mutation.window.to_string(),
            mutation: mutation.key.to_string(),
        };
    }
    let stored = decode_artifact(&saved.stored)?;
    if stored.request.as_ref().map(crate::host::paging::bytes) != Some(crate::host::paging::bytes(&request)) {
        return Err(Error::Conflict("Create Artifact: Use a New Request ID for Changed Content or Metadata".into()));
    }
    let artifact = stored.artifact.clone();
    let mutation = MutationId {
        window: Identifier::new(saved.window.clone()).map_err(|source| Error::Invalid(format!("Create Artifact: {source}")))?,
        key: Identifier::new(saved.mutation.clone()).map_err(|source| Error::Invalid(format!("Create Artifact: {source}")))?,
    };
    let bound = NonZeroUsize::new(context.settings.message_bytes).unwrap();
    let value = Administration {
        key,
        scope,
        payload: saved.encode_to_vec(),
    };
    let value = persisted::catalog().encode(&value, bound).map_err(|source| Error::Invalid(format!("Create Artifact: {source}")))?;
    let response = context
        .storage::<Mutate>(MutateRequest {
            mutation: Some(mutation),
            items: vec![MutationItem::Create(value)],
        })
        .await?;
    if matches!(response.outcome, CompoundOutcome::Conflict { .. }) {
        return Err(Error::Conflict("Create Artifact: Refresh the Artifact Before Saving".into()));
    }
    let mut receipt = response.receipt.ok_or_else(|| Error::Precondition("Create Artifact: Return Its Durable Receipt".into()))?;
    receipt.request_id = request.request_id;
    Ok(ArtifactsCreateResponse { artifact, receipt: Some(receipt) })
}

pub async fn execution_files(context: &Context, execution: &str) -> Result<Vec<StoredExecutionFile>> {
    crate::host::execution_state::read(context, execution)?;
    let (entries, _) = administration::scan(context, &format!("content/execution-file/{execution}/"), None).await?;
    entries.into_iter().map(|entry| Ok(StoredExecutionFile::decode(entry.payload.as_slice())?)).collect()
}

pub async fn execution_file(context: &Context, execution: &str, path: &str) -> Result<StoredExecutionFile> {
    crate::host::execution_state::read(context, execution)?;
    dokime_storage::validate_execution_path(path)?;
    let (saved, revision) =
        administration::message::<StoredExecutionFile>(context, &format!("content/execution-file/{execution}/{path}")).await?;
    if revision.is_empty() {
        return Err(Error::NotFound("Read Execution File: Select a Registered Logical Path".into()));
    }
    Ok(saved)
}

pub async fn attach_execution_file(
    context: &Context, execution: &str, mut metadata: ExecutionFileMetadata, file: FileReference,
) -> Result<Receipt> {
    dokime_storage::validate_execution_path(&metadata.relative_path)?;
    let scope = Scope::Execution(execution).identifier()?;
    if file.metadata.as_ref().and_then(|metadata| metadata.scope.as_deref()) != Some(scope.as_str()) {
        return Err(Error::Invalid("Attach Execution File: Select a Native File in This Execution's Scope".into()));
    }
    crate::host::files::get::handle(context, FilesGetRequest { file: Some(file.clone()), at: None }).await?;
    let _guard = context.administration.lock().await;
    context.host.store().require_evidence_owner(None, Some(execution), "Attach Execution File")?;
    metadata.size_bytes = file.length;
    metadata.state = ExecutionFileState::Finalized as i32;
    let key = format!("content/execution-file/{execution}/{}", metadata.relative_path);
    let (_, revision) = administration::message::<StoredExecutionFile>(context, &key).await?;
    let saved = StoredExecutionFile {
        metadata: Some(metadata),
        file: Some(file),
    };
    let request_id = uuid::Uuid::now_v7().to_string();
    let value = Administration {
        key: key.clone(),
        scope: Scope::Execution(execution).identifier()?,
        payload: saved.encode_to_vec(),
    };
    let value = persisted::catalog()
        .encode(&value, NonZeroUsize::new(context.settings.message_bytes).unwrap())
        .map_err(|source| Error::Invalid(format!("Attach Execution File: {source}")))?;
    let definition = ObjectDefinition::of::<Administration>();
    let graph = definition.graph(&Default::default()).map_err(|source| Error::Invalid(format!("Attach Execution File: {source}")))?;
    let mutation = files::transfer::mutation(context, &key, &request_id).await?;
    let response = context
        .storage::<SetObject>(SetObjectRequest {
            mutation: Some(mutation),
            create_if_absent: revision.is_empty(),
            expected_revision: revision,
            schema: Some(SchemaInput {
                key: Some(value.schema.clone()),
                graph: Some(graph),
            }),
            value: Some(value),
        })
        .await?;
    let receipt = match response.outcome {
        Some(set_object_response::Outcome::Applied(saved)) => {
            saved.receipt.ok_or_else(|| Error::Precondition("Attach Execution File: Return Its Durable Receipt".into()))?
        }
        Some(set_object_response::Outcome::Conflict(_)) => {
            return Err(Error::Conflict("Attach Execution File: Refresh Its Logical Path Before Saving".into()));
        }
        None => return Err(Error::Precondition("Attach Execution File: Return Its Mutation Outcome".into())),
    };
    context.host.signal_change();
    Ok(receipt)
}

/* === Private Functions === */

pub(crate) fn artifact_scope(scope: &ArtifactScope) -> Result<Identifier> {
    if scope.case_id.is_some() {
        return Err(Error::Invalid("Artifact: Select an Execution or Cycle Owner".into()));
    }
    if let Some(execution) = scope.execution_id.as_deref() {
        return Ok(Scope::Execution(execution).identifier()?);
    }
    if let Some(cycle) = scope.cycle_id.as_deref() {
        return Ok(Scope::Cycle(cycle).identifier()?);
    }
    Err(Error::Invalid("Artifact: Select an Execution or Cycle Owner".into()))
}

pub(crate) fn decode_artifact(payload: &[u8]) -> Result<StoredArtifact> {
    let stored = StoredArtifact::decode(payload)?;
    if stored.artifact.is_none() || stored.request.as_ref().and_then(|request| request.file.as_ref()).is_none() {
        return Err(Error::Precondition("Read Artifact: Restore Its Committed Metadata and Native File Reference".into()));
    }
    Ok(stored)
}

fn same_file(left: &FileReference, right: &FileReference) -> bool {
    let identity = left.metadata.as_ref().zip(right.metadata.as_ref()).is_some_and(|(left, right)| left.id == right.id);
    let history = left
        .position
        .as_ref()
        .and_then(|position| position.history.as_ref())
        .zip(right.position.as_ref().and_then(|position| position.history.as_ref()))
        .is_some_and(|(left, right)| left == right);
    identity && history
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::same_file;
    use harmos_protocol::model::{FileMetadata, FileReference, History, Position};

    #[test]
    fn retention_uses_store_identity_even_when_a_caller_changes_file_metadata() {
        let reference = FileReference {
            metadata: Some(FileMetadata {
                id: "evidence".into(),
                media_type: "text/plain".into(),
                ..Default::default()
            }),
            position: Some(Position {
                history: Some(History {
                    store: "root".into(),
                    epoch: "epoch".into(),
                }),
                sequence: 3,
            }),
            ..Default::default()
        };
        let mut altered = reference.clone();
        altered.metadata.as_mut().unwrap().media_type = "application/octet-stream".into();
        assert!(same_file(&reference, &altered));
        altered.position.as_mut().unwrap().history.as_mut().unwrap().epoch = "another-epoch".into();
        assert!(!same_file(&reference, &altered));
        assert!(!same_file(&reference, &FileReference::default()));
    }
}
