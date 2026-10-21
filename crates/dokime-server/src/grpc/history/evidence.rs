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
use harmos::operations::{Context, Operation, Response};
use tonic::Status;

// Local Imports
use super::wire;
use crate::grpc::transcode::Transcode;
use crate::grpc::{
    catalog, content,
    value::{self, canonical},
};
use dokime::domain::operations::history as contract;
use dokime::{
    RuntimeHandle,
    runtime::query::execution_history::{ExecutionCheckpoint, ExecutionCheckpointEvidence},
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
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut contract::GetCheckpointEvidence, _: &mut (),
    ) -> Result<Response<dokime::domain::model::history::wire::CheckpointEvidence>, Status> {
        let checkpoint = wire::checkpoint(request.checkpoint.take())?;
        let sections = sections(std::mem::take(&mut request.sections))?;
        let maximum = self.resources.maximum;
        let only_outputs = sections.iter().all(|section| {
            matches!(
                section,
                dokime::domain::model::history::wire::EvidenceSection::Outputs
                    | dokime::domain::model::history::wire::EvidenceSection::Artifacts
            )
        });
        let runtime = &self.resources.runtime;
        let response = if only_outputs {
            let outputs = runtime.execution_checkpoint_outputs(checkpoint.clone()).await.map_err(wire::checkpoint_failure)?;
            checkpoint_outputs(&checkpoint, &outputs, &sections)?
        } else {
            let evidence = runtime.execution_checkpoint_evidence(checkpoint).await.map_err(wire::checkpoint_failure)?;
            checkpoint_evidence(evidence, &sections)?
        };
        if response.wire_len()? > maximum {
            return Err(status_failure(exhausted()));
        }
        Ok(response.into())
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    runtime.operations().register_wire_operation::<Get>(
        "dokime",
        dokime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), maximum }),
    )?;
    Ok(())
}

/* === Private Functions === */

fn sections(
    values: Vec<dokime::domain::model::history::wire::EvidenceSection>,
) -> Result<Vec<dokime::domain::model::history::wire::EvidenceSection>, Status> {
    if values.is_empty() {
        return Ok(vec![
            dokime::domain::model::history::wire::EvidenceSection::Outputs,
            dokime::domain::model::history::wire::EvidenceSection::Artifacts,
            dokime::domain::model::history::wire::EvidenceSection::RecordSchemas,
            dokime::domain::model::history::wire::EvidenceSection::TelemetryTables,
        ]);
    }
    value::collection(values.len())?;
    let mut selected = Vec::new();
    for raw in values {
        let section = raw;
        if section == dokime::domain::model::history::wire::EvidenceSection::Unspecified {
            return Err(Status::invalid_argument("evidence section must be explicitly selected"));
        }
        if !selected.contains(&section) {
            selected.push(section);
        }
    }
    Ok(selected)
}

fn checkpoint_outputs(
    checkpoint: &ExecutionCheckpoint, outputs: &[dokime::domain::model::execution::saved::ExecutionOutput],
    sections: &[dokime::domain::model::history::wire::EvidenceSection],
) -> Result<dokime::domain::model::history::wire::CheckpointEvidence, Status> {
    let files = if sections.contains(&dokime::domain::model::history::wire::EvidenceSection::Outputs) {
        collect(outputs.iter().map(|output| {
            let kind = dokime::ExecutionFileContentKind::from_metadata(&output.relative_path, Some(&output.file.metadata.media_type));
            Ok(dokime::domain::model::content::wire::ExecutionFileMetadata {
                relative_path: value::projection::text(&output.relative_path).map_err(status_failure)?,
                content_type: Some(value::projection::text(&output.file.metadata.media_type).map_err(status_failure)?),
                kind: content::files::kind(&kind),
                state: match output.stream_state {
                    Some(dokime::domain::model::execution::saved::SavedStreamState::Finalized) => {
                        dokime::domain::model::content::wire::ExecutionFileState::Finalized
                    }
                    Some(dokime::domain::model::execution::saved::SavedStreamState::Interrupted) => {
                        dokime::domain::model::content::wire::ExecutionFileState::Interrupted
                    }
                    None => dokime::domain::model::content::wire::ExecutionFileState::Streaming,
                },
                size_bytes: output.file.length,
                created_at: Some(canonical::timestamp(&output.created_at)?),
            })
        }))?
    } else {
        Vec::new()
    };
    let count = if sections.contains(&dokime::domain::model::history::wire::EvidenceSection::Artifacts) {
        outputs
            .iter()
            .filter(|output| matches!(output.content, dokime::domain::model::execution::saved::ExecutionOutputContent::Artifact { .. }))
            .count()
    } else {
        0
    };
    let mut artifacts = value::projection::slots(count).map_err(status_failure)?;
    if count != 0 {
        for output in outputs {
            let dokime::domain::model::execution::saved::ExecutionOutputContent::Artifact { record, .. } = &output.content else {
                continue;
            };
            artifacts.push(super::super::content::metadata(record)?);
        }
    }
    Ok(dokime::domain::model::history::wire::CheckpointEvidence {
        checkpoint: Some(wire::checkpoint_wire_ref(checkpoint)?),
        outputs: files,
        artifacts,
        record_schemas: Vec::new(),
        telemetry_tables: Vec::new(),
    })
}

fn checkpoint_evidence(
    value: ExecutionCheckpointEvidence, sections: &[dokime::domain::model::history::wire::EvidenceSection],
) -> Result<dokime::domain::model::history::wire::CheckpointEvidence, Status> {
    let selected = |section| sections.contains(&section);
    let outputs = if selected(dokime::domain::model::history::wire::EvidenceSection::Outputs) {
        value.outputs.into_iter().map(file).collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let artifacts = if selected(dokime::domain::model::history::wire::EvidenceSection::Artifacts) {
        value.artifacts.iter().map(super::super::content::metadata).collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let record_schemas = if selected(dokime::domain::model::history::wire::EvidenceSection::RecordSchemas) {
        value.record_contracts.iter().map(catalog::schemas::record).collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let telemetry_tables = if selected(dokime::domain::model::history::wire::EvidenceSection::TelemetryTables) {
        value
            .telemetry
            .iter()
            .map(|item| {
                Ok(dokime::domain::model::history::wire::CheckpointTelemetryTable {
                    table: item.table.clone(),
                    schema: Some(catalog::schemas::telemetry(&item.contract)?),
                })
            })
            .collect::<Result<_, Status>>()?
    } else {
        Vec::new()
    };
    Ok(dokime::domain::model::history::wire::CheckpointEvidence {
        checkpoint: Some(wire::checkpoint_wire(value.checkpoint)?),
        outputs,
        artifacts,
        record_schemas,
        telemetry_tables,
    })
}

fn file(value: dokime::ExecutionFileEntry) -> Result<dokime::domain::model::content::wire::ExecutionFileMetadata, Status> {
    value::text(&value.relative_path)?;
    if let Some(content_type) = &value.content_type {
        value::text(content_type)?;
    }
    Ok(dokime::domain::model::content::wire::ExecutionFileMetadata {
        relative_path: value.relative_path,
        content_type: value.content_type,
        kind: match value.content_kind {
            dokime::ExecutionFileContentKind::Text => dokime::domain::model::content::wire::ExecutionFileKind::Text,
            dokime::ExecutionFileContentKind::Binary => dokime::domain::model::content::wire::ExecutionFileKind::Binary,
            dokime::ExecutionFileContentKind::Media => dokime::domain::model::content::wire::ExecutionFileKind::Media,
            dokime::ExecutionFileContentKind::Unknown => dokime::domain::model::content::wire::ExecutionFileKind::Unknown,
        },
        state: match value.stream_state {
            dokime::ExecutionFileStreamState::Streaming => dokime::domain::model::content::wire::ExecutionFileState::Streaming,
            dokime::ExecutionFileStreamState::Finalized => dokime::domain::model::content::wire::ExecutionFileState::Finalized,
            dokime::ExecutionFileStreamState::Interrupted => dokime::domain::model::content::wire::ExecutionFileState::Interrupted,
        },
        size_bytes: value.size_bytes,
        created_at: Some(canonical::timestamp(&value.updated_at)?),
    })
}

fn collect<T>(items: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut values = value::projection::slots(items.len()).map_err(status_failure)?;
    for item in items {
        values.push(item?);
    }
    Ok(values)
}

fn status_failure(source: harmos::Error) -> Status {
    super::wire::checkpoint_failure(source.into())
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted { resource: "canonical checkpoint evidence" }
}

/* === Tests === */
