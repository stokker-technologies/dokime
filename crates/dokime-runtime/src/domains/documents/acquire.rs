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

// Third Party Imports
use harmos::MemorySize;

// Local Imports
use crate::document::{DocumentSelection, bounded::document_size};
use crate::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Acquired {
    pub records: crate::storage::projection::Pages<dokime_domain::model::records::evidence::RecordEntry>,
    pub artifacts: Option<harmos::storage::ReadBatch<Vec<crate::artifacts::ArtifactRecord>>>,
    pub telemetry: Option<harmos::storage::ReadBatch<Vec<TelemetrySample>>>,
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

impl From<harmos::storage::Error> for DocumentAcquisitionFailure {
    fn from(error: harmos::storage::Error) -> Self {
        source(error)
    }
}

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
    telemetry_needed: bool, artifacts_needed: bool, at: harmos::storage::StorageCursor,
) -> Result<Acquired, DocumentAcquisitionFailure> {
    let limits = crate::document::bounded::DocumentLimits::default();
    let (source_bytes, source_depth) = (limits.source_bytes, limits.depth);
    let mut captured = Acquired {
        records: harmos::storage::ReadBatch::new(Vec::new()),
        artifacts: None,
        telemetry: None,
    };
    let session = runtime.storage().clone();
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
        // Select By Owner First, Then Validate The Selected Payload's Full Scope.
        let scope = crate::evidence_runtime::ScopeSelection {
            test_execution_id: Some(execution.test_execution_id.clone()),
            ..Default::default()
        };
        let mut after = 0;
        let mut pages = harmos::storage::ReadBatch::new(Vec::new());
        loop {
            let rows = runtime.local_storage.scoped_records(at.clone(), scope.clone(), None, after, 0, u64::MAX).await.map_err(source)?;
            let Some(last) = rows.last() else { break };
            after = last.position;
            let page = harmos::storage::ReadBatch::new(rows);
            let page = page.try_map(|rows| {
                let mut selected = Vec::new();
                for row in rows {
                    let mut record = row.record;
                    let scope = execution.records.captured_scope(&record).or(record.scope.as_ref());
                    if selection.foreign_execution_scope(scope) {
                        return Err(DocumentAcquisitionFailure::IncompleteCapture);
                    }
                    if !selection.matches_scope(scope) {
                        continue;
                    }
                    let scope = scope.cloned();
                    record.scope = scope;
                    let encoded = document_size(&record, source_bytes, source_depth).map_err(|_| DocumentAcquisitionFailure::Limit)?;
                    charge(encoded.max(record.memory_size()))?;
                    selected.try_reserve(1).map_err(|_| DocumentAcquisitionFailure::Limit)?;
                    selected.push(record);
                }
                Ok::<_, DocumentAcquisitionFailure>(selected)
            })?;
            pages = crate::storage::projection::push(pages, page).map_err(|error| native_error(error.into()))?;
        }
        captured.records = pages;
    }

    if telemetry_needed {
        let samples = runtime
            .collect_telemetry_at_in(
                &session,
                &crate::evidence_runtime::TelemetrySelection {
                    scope: selection.evidence_scope(),
                    ..Default::default()
                },
                None,
                None,
                &at,
            )
            .await
            .map_err(source)?;
        let samples = samples
            .try_map(|mut samples| {
                samples.retain(|sample| selection.matches_sample(sample));
                for sample in &mut samples {
                    if let Some(scope) = selection.captured_sample_scope(sample) {
                        sample.scope = Some(scope.clone());
                    }
                }
                Ok::<_, harmos::Error>(samples)
            })
            .map_err(source)?;
        charge(samples.memory_size())?;
        for sample in samples.iter() {
            document_size(sample, source_bytes, source_depth).map_err(|_| DocumentAcquisitionFailure::Limit)?;
        }
        captured.telemetry = Some(samples);
    }
    if artifacts_needed {
        let executions =
            if let Some(id) = &selection.scope.test_execution_id {
                vec![id.clone()]
            } else {
                let id = selection.scope.test_cycle_id.as_ref().ok_or(DocumentAcquisitionFailure::Missing)?;
                let cycle = snapshot.state().cycles.get(id).ok_or(DocumentAcquisitionFailure::Missing)?;
                let current = runtime.watch_state().latest();
                if current.state().cycles.get(id).is_none_or(|current| current.execution_ids != cycle.execution_ids) {
                    return Err(DocumentAcquisitionFailure::IncompleteCapture);
                }
                cycle
                    .execution_ids
                    .iter()
                    .filter(|id| {
                        selection.scope.test_case_id.as_ref().is_none_or(|case| {
                            snapshot.state().executions.get(*id).is_some_and(|execution| &execution.test_case_id == case)
                        })
                    })
                    .cloned()
                    .collect()
            };
        let records = runtime.execution_storage.selected_artifact_records(&session, &at, &executions).await.map_err(native_error)?;
        captured.artifacts = Some(
            records
                .try_map(|mut records| -> harmos::Result<_> {
                    crate::document::query::validate_cycle_artifacts(selection, snapshot, &records)?;
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
                    }
                    Ok(records)
                })
                .map_err(|error| native_error(error.into()))?,
        );
    }
    Ok(captured)
}

/* === Private Functions === */

fn artifact_limit(_: impl std::fmt::Display) -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "document artifact source budget",
    }
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
