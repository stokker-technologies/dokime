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
use std::io::Write;

// Third Party Imports
use harmos::MemorySize;
use sha2::{Digest, Sha256};

// Local Imports
use crate::document::{DocumentDataInput, DocumentSelection, bounded::document_size};
use crate::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Acquired {
    pub data: DocumentDataInput,
    pub records: crate::storage::ReadRows<dokime_domain::model::records::evidence::RecordEntry>,
    pub artifacts: Option<Vec<crate::artifacts::ArtifactRecord>>,
    pub telemetry_cut: Option<harmos::storage::StorageCursor>,
    pub telemetry: Option<harmos::storage::ReadBatch<Vec<TelemetrySample>>>,
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
            DocumentAcquisitionFailure::Source(_) => Self::unavailable("document_source_unavailable", error.to_string()),
        }
    }
}

/* === Public Functions === */

pub(super) async fn acquire(
    runtime: &crate::RuntimeHandle, selection: &DocumentSelection, snapshot: &crate::queries::RuntimeStateSnapshot, records_needed: bool,
    telemetry_needed: bool, artifacts_needed: bool,
) -> Result<Acquired, DocumentAcquisitionFailure> {
    let source_bytes = 16 * 1024 * 1024;
    let source_rows = 100_000;
    let source_depth = 32;
    let evaluation_work = 256_000_000;
    let mut captured = Acquired {
        data: DocumentDataInput::default(),
        records: Default::default(),
        artifacts: None,
        telemetry_cut: None,
        telemetry: None,
        artifacts_sha256: None,
    };
    let session = runtime.storage().clone();
    let at = runtime.saved_through(snapshot.revision()).await.map_err(source)?;
    let mut bytes = 0usize;
    let mut charge = |value: usize| {
        bytes = bytes.checked_add(value).filter(|bytes| *bytes <= source_bytes).ok_or(DocumentAcquisitionFailure::Limit)?;
        Ok::<_, DocumentAcquisitionFailure>(())
    };
    if records_needed {
        let execution = snapshot
            .state()
            .executions
            .get(selection.scope.test_execution_id.as_ref().ok_or(DocumentAcquisitionFailure::Missing)?)
            .ok_or(DocumentAcquisitionFailure::Missing)?;
        let mut reader = session
            .read_records::<crate::storage::records::RecordRow>(harmos::storage::RecordRead {
                at: at.clone(),
                selection: harmos::storage::query::RecordSelection {
                    schema: crate::storage::records::schema(),
                    scope: None,
                    item: None,
                    source_start: None,
                    source_end: None,
                },
                after: 0,
            })
            .await
            .map_err(source)?;
        let mut pages = harmos::storage::ReadBatch::new(Vec::new());
        let mut count = 0usize;
        while let Some(page) = reader.next().await.map_err(source)? {
            let page = page.try_map(|rows| {
                let mut selected = Vec::new();
                for row in rows {
                    let mut record = row.payload.entry().map_err(source)?;
                    let scope = execution.records.captured_scope(&record).or(record.scope.as_ref());
                    if selection.foreign_execution_scope(scope) {
                        return Err(DocumentAcquisitionFailure::IncompleteCapture);
                    }
                    if !selection.matches_scope(scope) {
                        continue;
                    }
                    let scope = scope.cloned();
                    record.scope = scope;
                    count = count.checked_add(1).filter(|count| *count <= source_rows).ok_or(DocumentAcquisitionFailure::Limit)?;
                    let encoded = document_size(&record, source_bytes, source_depth).map_err(|_| DocumentAcquisitionFailure::Limit)?;
                    charge(encoded.max(record.memory_size()))?;
                    selected.try_reserve(1).map_err(|_| DocumentAcquisitionFailure::Limit)?;
                    selected.push(record);
                }
                Ok::<_, DocumentAcquisitionFailure>(selected)
            })?;
            pages = crate::storage::projection::push(pages, page).map_err(|error| native_error(error.into()))?;
        }
        captured.records = pages.into();
    }

    if telemetry_needed {
        let samples = runtime
            .collect_telemetry_at_in(
                &session,
                &crate::evidence_runtime::TelemetrySelection { scope: selection.evidence_scope(), ..Default::default() },
                Some(source_rows),
                None,
                &at,
            )
            .await
            .map_err(source)?;
        let samples = samples
            .try_map(|mut samples| {
                samples.retain(|sample| selection.matches_scope(sample.scope.as_ref()));
                Ok::<_, harmos::Error>(samples)
            })
            .map_err(source)?;
        charge(samples.memory_size())?;
        for sample in samples.iter() {
            document_size(sample, source_bytes, source_depth).map_err(|_| DocumentAcquisitionFailure::Limit)?;
        }
        captured.telemetry_cut = Some(at);
        captured.telemetry = Some(samples);
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
                validate_cycle_artifacts(selection, snapshot, &records, evaluation_work)?;
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
                    document_size(record, source_bytes, source_depth).map_err(artifact_limit)?;
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
    evaluation_work: usize,
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
            .filter(|work| *work <= evaluation_work)
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
