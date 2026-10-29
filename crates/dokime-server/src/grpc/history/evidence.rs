/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::grpc::transcode::Transcode;

use std::sync::Arc;

use dokime::harmos::{Context, Response};
use tonic::Status;

use super::wire;
use crate::grpc::{
    catalog, content,
    value::{self, canonical},
};
use dokime::app::LiveRuntime;
use dokime::operations::history as contract;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::query::execution_history::{ExecutionCheckpoint, ExecutionCheckpointEvidence};

struct Resources {
    runtime: LiveRuntime,
    maximum: usize,
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(contract::GetCheckpointEvidence),
}

impl Serve<contract::GetCheckpointEvidence> for Resources {
    fn serve<'a>(
        &'a self, mut request: contract::GetCheckpointEvidence, _: Context<'a, contract::GetCheckpointEvidence>,
    ) -> Served<'a, dokime::domain::model::history::wire::CheckpointEvidence> {
        Box::pin(async move {
            let checkpoint = wire::checkpoint(request.checkpoint.take())?;
            let sections = sections(std::mem::take(&mut request.sections))?;
            let maximum = self.maximum;
            let only_outputs = !sections.record_schemas && !sections.telemetry_tables;
            let runtime = &dokime::app::live_runtime(&self.runtime).await?;
            let response = if only_outputs {
                let outputs = runtime.execution_checkpoint_outputs(&checkpoint).await.map_err(wire::checkpoint_failure)?;
                outputs
                    .project(|outputs| {
                        checkpoint_outputs(&checkpoint, outputs, &sections, maximum).map_err(|error| original_error(error).into())
                    })
                    .map_err(wire::checkpoint_failure)?
            } else {
                let evidence = runtime.execution_checkpoint_evidence(checkpoint).await.map_err(wire::checkpoint_failure)?;
                evidence
                    .try_map(|evidence| -> dokime::runtime::Result<_> {
                        checkpoint_evidence(evidence, &sections, maximum, runtime.schema_settings())
                            .map_err(|error| original_error(error).into())
                    })
                    .map_err(wire::checkpoint_failure)?
            };
            if response.wire_len()? > self.maximum {
                return Err(Status::resource_exhausted("historical response exceeds transport limit"));
            }
            Ok::<_, Status>(Response::from_owner(response.owner()))
        })
    }
}

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });

    setup.resource::<Host<contract::GetCheckpointEvidence>>(Host::<contract::GetCheckpointEvidence>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

#[derive(Clone, Copy)]
struct Sections {
    outputs: bool,
    artifacts: bool,
    record_schemas: bool,
    telemetry_tables: bool,
}

impl Sections {
    fn contains(&self, section: &dokime::domain::model::history::wire::EvidenceSection) -> bool {
        use dokime::domain::model::history::wire::EvidenceSection;
        match section {
            EvidenceSection::Outputs => self.outputs,
            EvidenceSection::Artifacts => self.artifacts,
            EvidenceSection::RecordSchemas => self.record_schemas,
            EvidenceSection::TelemetryTables => self.telemetry_tables,
            EvidenceSection::Unspecified => false,
        }
    }
}

fn sections(values: Vec<dokime::domain::model::history::wire::EvidenceSection>) -> Result<Sections, Status> {
    use dokime::domain::model::history::wire::EvidenceSection;
    let all = values.is_empty();
    let mut selected = Sections {
        outputs: all,
        artifacts: all,
        record_schemas: all,
        telemetry_tables: all,
    };
    for section in values {
        match section {
            EvidenceSection::Outputs => selected.outputs = true,
            EvidenceSection::Artifacts => selected.artifacts = true,
            EvidenceSection::RecordSchemas => selected.record_schemas = true,
            EvidenceSection::TelemetryTables => selected.telemetry_tables = true,
            EvidenceSection::Unspecified => return Err(Status::invalid_argument("evidence section must be explicitly selected")),
        }
    }
    Ok(selected)
}

fn checkpoint_outputs(
    checkpoint: &ExecutionCheckpoint, outputs: &[dokime::domain::model::execution::saved::ExecutionOutput], sections: &Sections,
    maximum: usize,
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
        checkpoint: Some(wire::checkpoint_wire_ref(checkpoint, maximum)?),
        outputs: files,
        artifacts,
        record_schemas: Vec::new(),
        telemetry_tables: Vec::new(),
    })
}

fn checkpoint_evidence(
    value: ExecutionCheckpointEvidence, sections: &Sections, maximum: usize, schema_settings: &dokime::harmos::config::SchemaSettings,
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
                    schema: Some(catalog::schemas::telemetry(&item.contract, schema_settings)?),
                })
            })
            .collect::<Result<_, Status>>()?
    } else {
        Vec::new()
    };
    Ok(dokime::domain::model::history::wire::CheckpointEvidence {
        checkpoint: Some(wire::checkpoint_wire(value.checkpoint, maximum)?),
        outputs,
        artifacts,
        record_schemas,
        telemetry_tables,
    })
}

fn file(value: dokime::ExecutionFileEntry) -> Result<dokime::domain::model::content::wire::ExecutionFileMetadata, Status> {
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

fn status_failure(source: dokime::harmos::Error) -> Status {
    super::wire::checkpoint_failure(source.into())
}

fn original_error(source: Status) -> dokime::harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return exhausted();
    }
    dokime::harmos::Error::backend("project original checkpoint evidence", dokime::harmos::BackendFailure::Corrupt, source)
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted {
        resource: "canonical checkpoint evidence",
    }
}
