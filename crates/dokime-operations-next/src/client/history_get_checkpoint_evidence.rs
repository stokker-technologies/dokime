/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// History Get Checkpoint Evidence

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.history.get_checkpoint_evidence";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::HistoryGetCheckpointEvidenceRequest, v1::CheckpointEvidence>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::HistoryGetCheckpointEvidenceRequest) -> Result<v1::CheckpointEvidence> {
    super::complete(ID, context.settings.read_timeout, async {
        let (checkpoint, cut) = super::history::selected(context, request.checkpoint.as_ref())?;
        let all = request.sections.is_empty();
        if request.sections.iter().any(|section| {
            !matches!(
                v1::EvidenceSection::try_from(*section),
                Ok(v1::EvidenceSection::Outputs
                    | v1::EvidenceSection::Artifacts
                    | v1::EvidenceSection::RecordSchemas
                    | v1::EvidenceSection::TelemetryTables)
            )
        }) {
            return Err(Error::Invalid("Get Checkpoint Evidence: Select Valid Evidence Sections".into()));
        }
        let snapshot = super::execution_state::read(context, &cut.execution.identifier)?;
        let schemas = super::projection::schemas(&snapshot)?;
        let artifacts = if all
            || request.sections.contains(&(v1::EvidenceSection::Artifacts as i32))
            || request.sections.contains(&(v1::EvidenceSection::Outputs as i32))
        {
            super::history::artifacts(context, &cut)?
        } else {
            Vec::new()
        };
        let outputs = if all || request.sections.contains(&(v1::EvidenceSection::Outputs as i32)) {
            artifacts
                .iter()
                .map(|artifact| v1::ExecutionFileMetadata {
                    relative_path: artifact.filename.clone(),
                    content_type: artifact.content_type.clone(),
                    kind: v1::ExecutionFileKind::Binary as i32,
                    state: if artifact.state == v1::ArtifactState::Complete as i32 {
                        v1::ExecutionFileState::Finalized
                    } else {
                        v1::ExecutionFileState::Interrupted
                    } as i32,
                    size_bytes: artifact.size_bytes,
                    created_at: artifact.created_at,
                })
                .collect()
        } else {
            Vec::new()
        };
        let artifacts = if all || request.sections.contains(&(v1::EvidenceSection::Artifacts as i32)) { artifacts } else { Vec::new() };
        let record_schemas =
            if all || request.sections.contains(&(v1::EvidenceSection::RecordSchemas as i32)) { schemas.records } else { Vec::new() };
        let telemetry_tables = if all || request.sections.contains(&(v1::EvidenceSection::TelemetryTables as i32)) {
            schemas
                .streams
                .into_iter()
                .map(|schema| v1::CheckpointTelemetryTable {
                    table: schema.stream.clone(),
                    schema: Some(schema),
                })
                .collect()
        } else {
            Vec::new()
        };
        super::history::bounded(
            context,
            v1::CheckpointEvidence {
                checkpoint: Some(checkpoint),
                outputs,
                artifacts,
                record_schemas,
                telemetry_tables,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
