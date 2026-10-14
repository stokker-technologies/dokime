/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
// Third Party Imports
use harmos::{
    MemorySize, Protobuf,
    operations::{Context, Operation, Response},
};
use tonic::Status;
// Local Imports
use super::wire;
use crate::grpc::{
    catalog, content,
    value::{self, canonical},
};
use dokime_app::{
    RuntimeHandle,
    runtime::query::execution_history::{ExecutionCheckpoint, ExecutionCheckpointEvidence},
};
use dokime_contract::operations::{
    content as content_wire, history as contract, schema as schema_wire,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}
struct Get {
    resources: Arc<Resources>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for Get {
    type Declaration = contract::GetCheckpointEvidence;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut contract::GetCheckpointEvidenceRequest,
        _: &mut (),
    ) -> Result<Response<contract::CheckpointEvidence>, Status> {
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        let sections = sections(std::mem::take(&mut request.sections))?;
        let maximum = self.resources.maximum;
        let only_outputs = sections.iter().all(|section| {
            matches!(
                section,
                contract::EvidenceSection::Outputs | contract::EvidenceSection::Artifacts
            )
        });
        let response = if only_outputs {
            self.resources
                .runtime
                .execution_checkpoint_outputs_projected(
                    checkpoint,
                    |checkpoint, outputs, allowance| {
                        let graph = outputs_memory(checkpoint, outputs, &sections)?;
                        if graph.saturating_add(checkpoint.memory_size()) > allowance.get() {
                            return Err(exhausted());
                        }
                        let response = checkpoint_outputs(checkpoint, outputs, &sections)
                            .map_err(original_error)?;
                        if response.encoded_len() > maximum {
                            return Err(exhausted());
                        }
                        Ok(response)
                    },
                )
                .await
        } else {
            self.resources
                .runtime
                .execution_checkpoint_evidence(checkpoint, |evidence, allowance| {
                    let graph = evidence_memory(&evidence, &sections)?;
                    if graph.saturating_add(evidence.memory_size()) > allowance.get() {
                        return Err(exhausted());
                    }
                    let response =
                        checkpoint_evidence(evidence, &sections).map_err(original_error)?;
                    if response.encoded_len() > maximum {
                        return Err(exhausted());
                    }
                    Ok(response)
                })
                .await
        }
        .map_err(wire::checkpoint_failure)?;
        response
            .try_into_response()
            .map_err(|source| wire::checkpoint_failure(source.into()))
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Get>(
        "dokime",
        dokime_app::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
            maximum,
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn sections(
    values: Vec<contract::EvidenceSection>,
) -> Result<Vec<contract::EvidenceSection>, Status> {
    if values.is_empty() {
        return Ok(vec![
            contract::EvidenceSection::Outputs,
            contract::EvidenceSection::Artifacts,
            contract::EvidenceSection::RecordSchemas,
            contract::EvidenceSection::TelemetryTables,
        ]);
    }
    value::collection(values.len())?;
    let mut selected = Vec::new();
    for raw in values {
        let section = raw;
        if section == contract::EvidenceSection::Unspecified {
            return Err(Status::invalid_argument(
                "evidence section must be explicitly selected",
            ));
        }
        if !selected.contains(&section) {
            selected.push(section);
        }
    }
    Ok(selected)
}

fn checkpoint_outputs(
    checkpoint: &ExecutionCheckpoint,
    outputs: &[dokime_app::execution::saved::ExecutionOutput],
    sections: &[contract::EvidenceSection],
) -> Result<contract::CheckpointEvidence, Status> {
    let files = if sections.contains(&contract::EvidenceSection::Outputs) {
        collect(outputs.iter().map(|output| {
            let kind = dokime_app::ExecutionFileContentKind::from_metadata(
                &output.relative_path,
                Some(&output.file.metadata.media_type),
            );
            Ok(content_wire::ExecutionFileMetadata {
                relative_path: value::projection::text(&output.relative_path)
                    .map_err(status_failure)?,
                content_type: Some(
                    value::projection::text(&output.file.metadata.media_type)
                        .map_err(status_failure)?,
                ),
                kind: content::files::kind(&kind),
                state: match output.stream_state {
                    Some(dokime_app::execution::saved::SavedStreamState::Finalized) => {
                        content_wire::ExecutionFileState::Finalized
                    }
                    Some(dokime_app::execution::saved::SavedStreamState::Interrupted) => {
                        content_wire::ExecutionFileState::Interrupted
                    }
                    None => content_wire::ExecutionFileState::Streaming,
                },
                size_bytes: output.file.length,
                created_at: Some(canonical::timestamp(&output.created_at)?),
            })
        }))?
    } else {
        Vec::new()
    };
    let count = if sections.contains(&contract::EvidenceSection::Artifacts) {
        outputs
            .iter()
            .filter(|output| {
                matches!(
                    output.content,
                    dokime_app::execution::saved::ExecutionOutputContent::Artifact { .. }
                )
            })
            .count()
    } else {
        0
    };
    let mut artifacts = value::projection::slots(count).map_err(status_failure)?;
    if count != 0 {
        for output in outputs {
            let dokime_app::execution::saved::ExecutionOutputContent::Artifact { record, .. } =
                &output.content
            else {
                continue;
            };
            artifacts.push(super::super::content::metadata(record)?);
        }
    }
    Ok(contract::CheckpointEvidence {
        checkpoint: Some(wire::checkpoint_wire_ref(checkpoint)?),
        outputs: files,
        artifacts,
        record_schemas: Vec::new(),
        telemetry_tables: Vec::new(),
    })
}

fn checkpoint_evidence(
    value: ExecutionCheckpointEvidence,
    sections: &[contract::EvidenceSection],
) -> Result<contract::CheckpointEvidence, Status> {
    let selected = |section| sections.contains(&section);
    let outputs = if selected(contract::EvidenceSection::Outputs) {
        value
            .outputs
            .into_iter()
            .map(file)
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let artifacts = if selected(contract::EvidenceSection::Artifacts) {
        value
            .artifacts
            .iter()
            .map(super::super::content::metadata)
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let record_schemas = if selected(contract::EvidenceSection::RecordSchemas) {
        value
            .record_contracts
            .iter()
            .map(catalog::schemas::record)
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let telemetry_tables = if selected(contract::EvidenceSection::TelemetryTables) {
        value
            .telemetry
            .iter()
            .map(|item| {
                Ok(contract::CheckpointTelemetryTable {
                    table: item.table.clone(),
                    schema: Some(catalog::schemas::telemetry(&item.contract)?),
                })
            })
            .collect::<Result<_, Status>>()?
    } else {
        Vec::new()
    };
    Ok(contract::CheckpointEvidence {
        checkpoint: Some(wire::checkpoint_wire(value.checkpoint)?),
        outputs,
        artifacts,
        record_schemas,
        telemetry_tables,
    })
}

fn file(
    value: dokime_app::ExecutionFileEntry,
) -> Result<content_wire::ExecutionFileMetadata, Status> {
    value::text(&value.relative_path)?;
    if let Some(content_type) = &value.content_type {
        value::text(content_type)?;
    }
    Ok(content_wire::ExecutionFileMetadata {
        relative_path: value.relative_path,
        content_type: value.content_type,
        kind: match value.content_kind {
            dokime_app::ExecutionFileContentKind::Text => content_wire::ExecutionFileKind::Text,
            dokime_app::ExecutionFileContentKind::Binary => content_wire::ExecutionFileKind::Binary,
            dokime_app::ExecutionFileContentKind::Media => content_wire::ExecutionFileKind::Media,
            dokime_app::ExecutionFileContentKind::Unknown => {
                content_wire::ExecutionFileKind::Unknown
            }
        },
        state: match value.stream_state {
            dokime_app::ExecutionFileStreamState::Streaming => {
                content_wire::ExecutionFileState::Streaming
            }
            dokime_app::ExecutionFileStreamState::Finalized => {
                content_wire::ExecutionFileState::Finalized
            }
            dokime_app::ExecutionFileStreamState::Interrupted => {
                content_wire::ExecutionFileState::Interrupted
            }
        },
        size_bytes: value.size_bytes,
        created_at: Some(canonical::timestamp(&value.updated_at)?),
    })
}

fn evidence_memory(
    source: &dokime_app::runtime::query::execution_history::ExecutionCheckpointEvidence,
    sections: &[contract::EvidenceSection],
) -> harmos::Result<usize> {
    let mut bytes = std::mem::size_of::<contract::CheckpointEvidence>()
        .saturating_add(super::construction::checkpoint_heap(&source.checkpoint));
    let failure = |source: tonic::Status| {
        if source.code() == tonic::Code::ResourceExhausted {
            harmos::Error::ResourceExhausted {
                resource: "checkpoint evidence construction",
            }
        } else {
            harmos::Error::backend(
                "count checkpoint evidence",
                harmos::BackendFailure::Corrupt,
                source,
            )
        }
    };
    if sections.contains(&contract::EvidenceSection::Outputs) {
        bytes = bytes.saturating_add(slots::<content_wire::ExecutionFileMetadata>(
            source.outputs.len(),
        ));
        for output in &source.outputs {
            bytes = bytes
                .saturating_add(output.relative_path.len())
                .saturating_add(output.content_type.as_ref().map_or(0, String::len));
        }
    }
    if sections.contains(&contract::EvidenceSection::Artifacts) {
        bytes = bytes.saturating_add(slots::<content_wire::ArtifactMetadata>(
            source.artifacts.len(),
        ));
        for artifact in &source.artifacts {
            bytes = bytes
                .saturating_add(crate::grpc::content::metadata_memory(artifact).map_err(failure)?);
        }
    }
    if sections.contains(&contract::EvidenceSection::RecordSchemas) {
        bytes = bytes.saturating_add(slots::<schema_wire::RecordSchema>(
            source.record_contracts.len(),
        ));
        for contract in &source.record_contracts {
            bytes =
                bytes.saturating_add(catalog::schemas::record_memory(contract).map_err(failure)?);
        }
    }
    if sections.contains(&contract::EvidenceSection::TelemetryTables) {
        bytes = bytes.saturating_add(slots::<contract::CheckpointTelemetryTable>(
            source.telemetry.len(),
        ));
        for table in &source.telemetry {
            bytes = bytes.saturating_add(table.table.len()).saturating_add(
                catalog::schemas::telemetry_memory(&table.contract).map_err(failure)?,
            );
        }
    }
    bounded(bytes)
}

fn outputs_memory(
    checkpoint: &ExecutionCheckpoint,
    source: &[dokime_app::execution::saved::ExecutionOutput],
    sections: &[contract::EvidenceSection],
) -> harmos::Result<usize> {
    if sections.is_empty()
        || sections.iter().any(|section| {
            !matches!(
                section,
                contract::EvidenceSection::Outputs | contract::EvidenceSection::Artifacts
            )
        })
    {
        return Err(harmos::Error::InvalidInput {
            field: "checkpoint output sections",
            reason: "only explicit output and artifact sections are supported",
        });
    }
    let files = sections.contains(&contract::EvidenceSection::Outputs);
    let artifacts = sections.contains(&contract::EvidenceSection::Artifacts);
    let mut bytes = std::mem::size_of::<contract::CheckpointEvidence>()
        .saturating_add(super::construction::checkpoint_heap(checkpoint));
    if files {
        bytes = bytes.saturating_add(slots::<content_wire::ExecutionFileMetadata>(source.len()));
    }
    for output in source {
        if files {
            value::text(&output.relative_path).map_err(|_| harmos::Error::ResourceExhausted {
                resource: "checkpoint output path",
            })?;
            value::text(&output.file.metadata.media_type).map_err(|_| {
                harmos::Error::ResourceExhausted {
                    resource: "checkpoint output media type",
                }
            })?;
            bytes = bytes
                .saturating_add(output.relative_path.len())
                .saturating_add(output.file.metadata.media_type.len());
        }
        if artifacts
            && let dokime_app::execution::saved::ExecutionOutputContent::Artifact { record, .. } =
                &output.content
        {
            let memory = crate::grpc::content::metadata_memory(record).map_err(|error| {
                if error.code() == tonic::Code::ResourceExhausted {
                    return harmos::Error::ResourceExhausted {
                        resource: "checkpoint artifact metadata",
                    };
                }
                harmos::Error::backend(
                    "count captured artifact",
                    harmos::BackendFailure::Corrupt,
                    error,
                )
            })?;
            bytes = bytes.saturating_add(memory);
        }
    }
    bounded(bytes)
}

fn collect<T>(items: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut values = value::projection::slots(items.len()).map_err(status_failure)?;
    for item in items {
        values.push(item?);
    }
    Ok(values)
}
fn slots<T>(length: usize) -> usize {
    length
        .saturating_mul(4)
        .saturating_mul(std::mem::size_of::<T>())
}
fn bounded(bytes: usize) -> harmos::Result<usize> {
    if bytes == usize::MAX {
        Err(exhausted())
    } else {
        Ok(bytes)
    }
}
fn original_error(source: Status) -> harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    harmos::Error::backend(
        "project original checkpoint evidence",
        harmos::BackendFailure::Corrupt,
        source,
    )
}
fn status_failure(source: harmos::Error) -> Status {
    super::wire::checkpoint_failure(source.into())
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "canonical checkpoint evidence",
    }
}

/* === Tests === */
