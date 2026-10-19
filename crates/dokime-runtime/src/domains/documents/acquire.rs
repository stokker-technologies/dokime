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
use std::{collections::BTreeSet, io::Write, mem::size_of};
// Third Party Imports
use harmos::{MemorySize, streams::PageLimits, streams::ReadCut, streams::ReadLimits};
use sha2::{Digest, Sha256};
// Local Imports
use crate::document::{DocumentDataInput, DocumentSelection, bounded::document_size};
use crate::streams::TelemetryRow;
use crate::telemetry::TelemetrySample;

/* === Constants === */
const SOURCE_BYTES: usize = 16 * 1024 * 1024;
pub(super) const SOURCE_ROWS: usize = 100_000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Acquired {
    pub data: DocumentDataInput,
    pub records: Vec<crate::runtime::CapturedRecord>,
    pub artifacts: Option<Vec<crate::artifacts::ArtifactRecord>>,
    pub telemetry_cut: Option<ReadCut>,
    pub artifacts_sha256: Option<String>,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub(super) enum DocumentAcquisitionFailure {
    #[error("document source or materialization budget exceeded")]
    Limit,
    #[error("document selection not found")]
    Missing,
    #[error("selected evidence capture is incomplete")]
    IncompleteCapture,
    #[error("selected telemetry history has been retained away")]
    Retained,
    #[error("document evidence is unavailable: {0}")]
    Source(String),
}

/* === Traits === */

/* === Implementations === */
impl From<DocumentAcquisitionFailure> for crate::runtime::Error {
    fn from(error: DocumentAcquisitionFailure) -> Self {
        match error {
            DocumentAcquisitionFailure::Limit => Self::overload("document_limit", error.to_string()),
            DocumentAcquisitionFailure::Missing => Self::not_found("document_not_found", error.to_string()),
            DocumentAcquisitionFailure::IncompleteCapture => Self::conflict("document_capture_incomplete", error.to_string()),
            DocumentAcquisitionFailure::Retained => Self::conflict("document_history_retained", error.to_string()),
            DocumentAcquisitionFailure::Source(_) => Self::unavailable("document_source_unavailable", error.to_string()),
        }
    }
}

/* === Public Functions === */
pub(super) async fn acquire(
    runtime: &crate::RuntimeHandle, selection: &DocumentSelection, snapshot: &crate::queries::RuntimeStateSnapshot, records_needed: bool,
    telemetry_needed: bool, artifacts_needed: bool,
) -> Result<Acquired, DocumentAcquisitionFailure> {
    let mut captured =
        Acquired { data: DocumentDataInput::default(), records: Vec::new(), artifacts: None, telemetry_cut: None, artifacts_sha256: None };
    let mut bytes = 0usize;
    let mut charge = |value: usize| {
        bytes = bytes.checked_add(value).filter(|bytes| *bytes <= SOURCE_BYTES).ok_or(DocumentAcquisitionFailure::Limit)?;
        Ok::<_, DocumentAcquisitionFailure>(())
    };
    if records_needed {
        let execution = snapshot
            .state()
            .executions
            .get(selection.scope.test_execution_id.as_ref().ok_or(DocumentAcquisitionFailure::Missing)?)
            .ok_or(DocumentAcquisitionFailure::Missing)?;
        let scopes = execution.records.subscribed_records();
        if scopes.len() > SOURCE_ROWS {
            return Err(DocumentAcquisitionFailure::Limit);
        }
        if scopes.values().any(|scope| !selection.matches_scope(Some(scope))) {
            return Err(DocumentAcquisitionFailure::IncompleteCapture);
        }
        // Borrow Frozen Ownership And Keep Original Charged Pages Through Evaluation.
        charge(scopes.memory_size())?;
        charge(harmos::btree_map_memory::<&dokime_domain::model::identity::RecordId, ()>(scopes.len()))?;
        let mut missing: BTreeSet<_> = scopes.keys().collect();
        let mut reader = runtime.record_source.history(&runtime.state_manager, 0, snapshot.revision()).await.map_err(native_error)?;
        let mut scanned_rows = 0usize;
        let mut scanned_bytes = 0usize;
        while let Some(record) = reader.next().await.map_err(native_error)? {
            scanned_rows = scanned_rows.checked_add(1).filter(|rows| *rows <= SOURCE_ROWS).ok_or(DocumentAcquisitionFailure::Limit)?;
            scanned_bytes = scanned_bytes
                .checked_add(record.memory_size())
                .filter(|bytes| *bytes <= SOURCE_BYTES)
                .ok_or(DocumentAcquisitionFailure::Limit)?;
            missing.remove(&record.record_id);
            let scope = scopes.get(&record.record_id).or(record.scope.as_ref());
            if selection.foreign_execution_scope(scope) {
                return Err(DocumentAcquisitionFailure::IncompleteCapture);
            }
            if selection.matches_scope(scope) {
                document_size(&*record, SOURCE_BYTES, 32).map_err(|_| DocumentAcquisitionFailure::Limit)?;
                charge(record.memory_size())?;
                if captured.records.len() == captured.records.capacity() {
                    let capacity = captured.records.capacity();
                    let next = capacity.saturating_mul(2).clamp(64, SOURCE_ROWS);
                    charge((next - capacity).saturating_mul(size_of::<crate::runtime::CapturedRecord>()))?;
                    captured.records.try_reserve_exact(next - captured.records.len()).map_err(|_| DocumentAcquisitionFailure::Limit)?;
                    if captured.records.capacity() > next {
                        return Err(DocumentAcquisitionFailure::Limit);
                    }
                }
                captured.records.push(record);
            }
        }
        if !missing.is_empty() {
            return Err(DocumentAcquisitionFailure::IncompleteCapture);
        }
    }
    if telemetry_needed {
        // One source-owned cut retains ordinal order across all scope partitions.
        // Unselected history still consumes bounded source inspection work.
        let mut reader = runtime
            .streams
            .query::<TelemetryRow>(..)
            .reader(ReadLimits { rows: SOURCE_ROWS, ..Default::default() })
            .await
            .map_err(read_error)?;
        captured.telemetry_cut = Some(reader.cut());
        let mut scanned_bytes = 0usize;
        loop {
            let page = reader.next_page(PageLimits::default()).await.map_err(read_error)?;
            scanned_bytes = scanned_bytes
                .checked_add(page.encoded_bytes)
                .filter(|bytes| *bytes <= SOURCE_BYTES)
                .ok_or(DocumentAcquisitionFailure::Limit)?;
            for row in page.rows {
                // The source has already bounded the encoded row and payload;
                // decoding cannot consume another unbounded batch.
                let mut sample: TelemetrySample = crate::persistence::from_slice(&row.value.payload).map_err(source)?;
                sample.sequence = row.value.ordinal;
                if selection.matches_scope(sample.scope.as_ref()) {
                    document_size(&sample, SOURCE_BYTES, 32).map_err(|_| DocumentAcquisitionFailure::Limit)?;
                    charge(sample.memory_size())?;
                    captured.data.telemetry.push(sample);
                }
            }
            if page.complete {
                break;
            }
        }
    }
    if artifacts_needed {
        struct Hasher(Sha256);
        impl Write for Hasher {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.update(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut digest = Hasher(Sha256::new());
        let records = runtime.execution_storage.artifact_records().await.map_err(native_error)?;
        captured.artifacts = Some(
            (|mut records: Vec<crate::artifacts::ArtifactRecord>| -> harmos::Result<_> {
                validate_cycle_artifacts(selection, snapshot, &records)?;
                if records.iter().any(|record| selection.foreign_execution_scope(Some(&record.scope))) {
                    return Err(harmos::Error::InvalidInput {
                        field: "document artifact scope",
                        reason: "artifact descriptor belongs to another cycle or case",
                    });
                }
                // Keep The Original Admitted Buffer And Its Charged Query Owner.
                records.retain(|record| selection.matches_scope(Some(&record.scope)));
                charge(records.memory_size()).map_err(artifact_limit)?;
                for record in &records {
                    document_size(record, SOURCE_BYTES, 32).map_err(artifact_limit)?;
                    serde_json::to_writer(&mut digest, record).map_err(|error| {
                        harmos::Error::backend("encode document artifact fingerprint", harmos::BackendFailure::Corrupt, error)
                    })?;
                    digest.write_all(b"\n").map_err(|error| {
                        harmos::Error::backend("write document artifact fingerprint", harmos::BackendFailure::Corrupt, error)
                    })?;
                }
                charge(size_of::<String>() + 64).map_err(artifact_limit)?;
                captured.artifacts_sha256 = Some(digest.0.finalize().iter().map(|byte| format!("{byte:02x}")).collect());
                Ok(records)
            })(records)
            .map_err(|error| native_error(error.into()))?,
        );
    }
    Ok(captured)
}

/* === Private Functions === */
fn validate_cycle_artifacts(
    selection: &DocumentSelection, snapshot: &crate::queries::RuntimeStateSnapshot, records: &[crate::artifacts::ArtifactRecord],
) -> harmos::Result<()> {
    if selection.scope.test_execution_id.is_some() {
        return Ok(());
    }
    let Some(id) = &selection.scope.test_cycle_id else {
        return Ok(());
    };
    let cycle = snapshot.state().cycles.get(id).ok_or_else(cycle_artifact_scope)?;
    let mut work = 0usize;
    for record in records {
        let Some(execution_id) = &record.scope.test_execution_id else {
            continue;
        };
        let owner = snapshot.state().executions.get(execution_id);
        if record.scope.test_cycle_id.as_ref() != Some(id) && owner.is_none_or(|owner| &owner.test_cycle_id != id) {
            continue;
        }
        work = work
            .checked_add(cycle.execution_ids.len())
            .filter(|work| *work <= crate::document::bounded::DocumentLimits::default().evaluation_work)
            .ok_or_else(|| artifact_limit("cycle artifact ownership work"))?;
        let owner = owner.ok_or_else(cycle_artifact_scope)?;
        if &owner.test_cycle_id != id
            || !cycle.execution_ids.contains(execution_id)
            || !crate::evidence_runtime::scopes_correlate(Some(&record.scope), Some(&owner.scope))
        {
            return Err(cycle_artifact_scope());
        }
    }
    Ok(())
}

fn cycle_artifact_scope() -> harmos::Error {
    harmos::Error::InvalidInput { field: "document artifact scope", reason: "artifact descriptor differs from the captured cycle members" }
}

fn artifact_limit(_: impl std::fmt::Display) -> harmos::Error {
    harmos::Error::ResourceExhausted { resource: "document artifact source budget" }
}

fn source(error: impl std::fmt::Display) -> DocumentAcquisitionFailure {
    DocumentAcquisitionFailure::Source(error.to_string())
}

fn read_error(error: harmos::streams::ReadError) -> DocumentAcquisitionFailure {
    match error {
        harmos::streams::ReadError::Limit(_) | harmos::streams::ReadError::Busy => DocumentAcquisitionFailure::Limit,
        harmos::streams::ReadError::IncompleteCapture => DocumentAcquisitionFailure::IncompleteCapture,
        harmos::streams::ReadError::Retained { .. } => DocumentAcquisitionFailure::Retained,
        error => source(error),
    }
}

fn native_error(error: crate::runtime::Error) -> DocumentAcquisitionFailure {
    match error {
        crate::runtime::Error::Harmos(harmos::Error::InvalidInput { field: "document artifact scope", .. })
        | crate::runtime::Error::Storage(crate::storage::Error::Runtime(harmos::Error::InvalidInput {
            field: "document artifact scope",
            ..
        })) => DocumentAcquisitionFailure::IncompleteCapture,
        crate::runtime::Error::Harmos(harmos::Error::ResourceExhausted { .. })
        | crate::runtime::Error::Storage(crate::storage::Error::Runtime(harmos::Error::ResourceExhausted { .. })) => {
            DocumentAcquisitionFailure::Limit
        }
        error => source(error),
    }
}

/* === Tests === */
