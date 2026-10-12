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

// Local Imports
use super::document;
use crate::grpc::{
    appearance, catalog, notes, prompts,
    value::{self, Budget, projection},
};
use dokime_app::core::{Metadata, RecordScope, Value};
use dokime_app::definition::{MaterializedParameter, TestCase};
use dokime_app::execution::{
    ExecutionImplementation, ExecutionPluginIdentity, ExecutionSnapshot, TestExecution,
};
use dokime_app::runtime::query::execution_history::{
    ExecutionCheckpoint, ExecutionCheckpointContext, ExecutionCheckpointDocument,
};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn document(source: &ExecutionCheckpointDocument) -> harmos::Result<usize> {
    let bytes = std::mem::size_of::<v1::CheckpointDocument>()
        .saturating_add(checkpoint_heap(&source.checkpoint))
        .saturating_add(source.document.cycle.cycle_id.as_str().len())
        .saturating_add(metadata_heap(&source.document.cycle.metadata)?)
        .saturating_add(execution_heap(&source.document.execution)?);
    bounded(bytes)
}

pub(super) fn context(source: &ExecutionCheckpointContext) -> harmos::Result<usize> {
    let mut bytes = std::mem::size_of::<v1::CheckpointContext>()
        .saturating_add(checkpoint_heap(&source.checkpoint))
        .saturating_add(source.session_id.as_str().len())
        .saturating_add(
            source
                .scope
                .test_execution_id
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
        )
        .saturating_add(scope_heap(&source.scope)?)
        .saturating_add(cycle_heap(&source.cycle)?)
        .saturating_add(slots::<v1::HistoricalDocument>(
            source.execution_documents.len(),
        ))
        .saturating_add(slots::<v1::PromptInstance>(source.prompts.len()))
        .saturating_add(slots::<v1::HealthItem>(source.health.len()))
        .saturating_add(slots::<v1::OperatorNote>(source.notes.len()))
        .saturating_add(slots::<v1::TraceabilitySnapshot>(source.traceability.len()))
        .saturating_add(
            appearance::historical_memory(&source.appearance)?
                - std::mem::size_of::<v1::AppearanceSnapshot>(),
        );
    for document in source.execution_documents.values() {
        bytes = bytes.saturating_add(
            document::construction_memory(document, &source.scope)?
                - std::mem::size_of::<v1::HistoricalDocument>(),
        );
    }
    for prompt in &source.prompts {
        let memory = prompts::historical_instance_memory(prompt).map_err(|error| {
            if error.code() == tonic::Code::ResourceExhausted {
                return harmos::Error::ResourceExhausted {
                    resource: "checkpoint prompt construction",
                };
            }
            harmos::Error::backend(
                "count captured prompt",
                harmos::BackendFailure::Corrupt,
                error,
            )
        })?;
        bytes = bytes.saturating_add(memory - std::mem::size_of::<v1::PromptInstance>());
    }
    for health in &source.health {
        bytes = bytes.saturating_add(
            notes::historical_health_memory(health)? - std::mem::size_of::<v1::HealthItem>(),
        );
    }
    for note in &source.notes {
        bytes = bytes.saturating_add(
            notes::historical_note_memory(note)? - std::mem::size_of::<v1::OperatorNote>(),
        );
    }
    for snapshot in &source.traceability {
        bytes = bytes.saturating_add(
            traceability(snapshot)? - std::mem::size_of::<v1::TraceabilitySnapshot>(),
        );
    }
    bounded(bytes)
}

pub(super) fn evidence(
    source: &dokime_app::runtime::query::execution_history::ExecutionCheckpointEvidence,
    sections: &[v1::EvidenceSection],
) -> harmos::Result<usize> {
    let mut bytes = std::mem::size_of::<v1::CheckpointEvidence>()
        .saturating_add(checkpoint_heap(&source.checkpoint));
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
    if sections.contains(&v1::EvidenceSection::Outputs) {
        bytes = bytes.saturating_add(slots::<v1::ExecutionFileMetadata>(source.outputs.len()));
        for output in &source.outputs {
            bytes = bytes
                .saturating_add(output.relative_path.len())
                .saturating_add(output.content_type.as_ref().map_or(0, String::len));
        }
    }
    if sections.contains(&v1::EvidenceSection::Artifacts) {
        bytes = bytes.saturating_add(slots::<v1::ArtifactMetadata>(source.artifacts.len()));
        for artifact in &source.artifacts {
            bytes = bytes
                .saturating_add(crate::grpc::content::metadata_memory(artifact).map_err(failure)?);
        }
    }
    if sections.contains(&v1::EvidenceSection::RecordSchemas) {
        bytes = bytes.saturating_add(slots::<v1::RecordSchema>(source.record_contracts.len()));
        for contract in &source.record_contracts {
            bytes =
                bytes.saturating_add(catalog::schemas::record_memory(contract).map_err(failure)?);
        }
    }
    if sections.contains(&v1::EvidenceSection::TelemetryTables) {
        bytes = bytes.saturating_add(slots::<v1::CheckpointTelemetryTable>(
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

pub(super) fn outputs(
    checkpoint: &ExecutionCheckpoint,
    source: &[dokime_app::execution::saved::ExecutionOutput],
    sections: &[v1::EvidenceSection],
) -> harmos::Result<usize> {
    if sections.is_empty()
        || sections.iter().any(|section| {
            !matches!(
                section,
                v1::EvidenceSection::Outputs | v1::EvidenceSection::Artifacts
            )
        })
    {
        return Err(harmos::Error::InvalidInput {
            field: "checkpoint output sections",
            reason: "only explicit output and artifact sections are supported",
        });
    }
    let files = sections.contains(&v1::EvidenceSection::Outputs);
    let artifacts = sections.contains(&v1::EvidenceSection::Artifacts);
    let mut bytes =
        std::mem::size_of::<v1::CheckpointEvidence>().saturating_add(checkpoint_heap(checkpoint));
    if files {
        bytes = bytes.saturating_add(slots::<v1::ExecutionFileMetadata>(source.len()));
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

pub(super) fn traceability(
    source: &dokime_app::traceability::TraceabilitySnapshot,
) -> harmos::Result<usize> {
    let mut bytes = std::mem::size_of::<v1::TraceabilitySnapshot>()
        .saturating_add(source.traceability_id.as_str().len())
        .saturating_add(scope_heap(&source.scope)?)
        .saturating_add(slots::<v1::TraceabilityEntry>(source.entries.len()));
    if let Some(metadata) = &source.metadata {
        bytes = bytes.saturating_add(metadata_heap(metadata)?);
    }
    let mut budget = Budget::new();
    for entry in &source.entries {
        bytes = bytes
            .saturating_add(metadata_heap(&entry.metadata)?)
            .saturating_add(value_heap(&entry.value, &mut budget)?)
            .saturating_add(optional(&entry.source))
            .saturating_add(optional(&entry.unit));
    }
    bounded(bytes)
}

pub(super) fn checkpoint_heap(source: &ExecutionCheckpoint) -> usize {
    let mut bytes = source
        .execution_id
        .as_str()
        .len()
        .saturating_add(harmos::Message::encoded_len(&source.version));
    for cursor in [&source.position, &source.evidence_through] {
        bytes = bytes
            .saturating_add(cursor.history.store.as_str().len())
            .saturating_add(cursor.history.epoch.as_str().len());
    }
    bytes
}

/* === Private Functions === */

fn cycle_heap(source: &dokime_app::runtime::ExecutionCycleContext) -> harmos::Result<usize> {
    let mut bytes = source
        .cycle
        .test_cycle_id
        .as_str()
        .len()
        .saturating_add(metadata_heap(&source.cycle.metadata)?)
        .saturating_add(slots::<v1::CaseSummary>(source.cases.len()))
        .saturating_add(slots::<String>(source.execution_order.len()));
    for id in &source.execution_order {
        bytes = bytes.saturating_add(id.as_str().len());
    }
    for case in &source.cases {
        value::text(&case.suite_id).map_err(|_| harmos::Error::ResourceExhausted {
            resource: "historical case suite identifier",
        })?;
        bytes = bytes
            .saturating_add(case.suite_id.len())
            .saturating_add(case.test_case_id.as_str().len())
            .saturating_add(case.test_case_template_id.as_str().len())
            .saturating_add(
                projection::metadata_fields_count(&case.metadata)?.memory
                    - std::mem::size_of::<v1::Metadata>(),
            )
            .saturating_add(
                dokime_app::runtime::query::CaseProjection::historical_memory(
                    &source.cycle,
                    &case.test_case_id,
                ),
            );
        if let Some(latest) = source
            .cycle
            .test_executions
            .iter()
            .rev()
            .find(|execution| execution.test_case_id == case.test_case_id)
        {
            bytes = bytes.saturating_add(latest.test_execution_id.as_str().len());
        }
    }
    bounded(bytes)
}

fn bounded(bytes: usize) -> harmos::Result<usize> {
    if bytes == usize::MAX {
        return Err(harmos::Error::ResourceExhausted {
            resource: "checkpoint document construction",
        });
    }
    Ok(bytes)
}

fn slots<T>(length: usize) -> usize {
    length.saturating_mul(std::mem::size_of::<T>())
}

fn optional(source: &Option<String>) -> usize {
    source.as_ref().map_or(0, String::len)
}

fn metadata_heap(source: &Metadata) -> harmos::Result<usize> {
    Ok(projection::metadata_count(source)?.memory - std::mem::size_of::<v1::Metadata>())
}

fn scope_heap(source: &RecordScope) -> harmos::Result<usize> {
    Ok(bounded(projection::scope_memory(source))? - std::mem::size_of::<v1::EvidenceScope>())
}

fn value_heap(source: &Value, budget: &mut Budget) -> harmos::Result<usize> {
    Ok(projection::count(source, budget, 0)?.memory - std::mem::size_of::<v1::Value>())
}

fn plugin_heap(source: &ExecutionPluginIdentity) -> usize {
    let implementation = match &source.implementation {
        ExecutionImplementation::Bundle { content_digest } => content_digest.len(),
        ExecutionImplementation::Embedded { generation } => generation.len(),
    };
    source
        .source
        .len()
        .saturating_add(optional(&source.version))
        .saturating_add(implementation)
}

fn case_heap(source: &TestCase) -> harmos::Result<usize> {
    let mut bytes = source
        .suite_id
        .len()
        .saturating_add(source.test_case_id.as_str().len())
        .saturating_add(source.test_case_template_id.as_str().len())
        .saturating_add(metadata_heap(&source.metadata)?)
        .saturating_add(slots::<v1::CaseParameter>(source.parameters.len()));
    if let Some(suite) = &source.suite {
        bytes = bytes.saturating_add(metadata_heap(suite)?);
    }
    let mut budget = Budget::new();
    for (name, value) in &source.parameters {
        bytes = bytes
            .saturating_add(name.len())
            .saturating_add(value_heap(value, &mut budget)?);
    }
    bounded(bytes)
}

fn materialized(
    parameters: &std::collections::BTreeMap<String, MaterializedParameter>,
) -> harmos::Result<usize> {
    let mut bytes = slots::<v1::MaterializedParameter>(parameters.len());
    let mut budget = Budget::new();
    for (name, parameter) in parameters {
        bytes = bytes
            .saturating_add(name.len())
            .saturating_add(metadata_heap(&parameter.metadata)?)
            .saturating_add(
                parameter
                    .group_id
                    .as_ref()
                    .map_or(0, |id| id.as_str().len()),
            )
            .saturating_add(value_heap(&parameter.value, &mut budget)?);
        if let Some(unit) = &parameter.unit {
            bytes = bytes.saturating_add(
                catalog::unit_memory(unit)? - std::mem::size_of::<v1::MeasurementUnit>(),
            );
        }
    }
    bounded(bytes)
}

fn documents(
    source: &std::collections::BTreeMap<
        dokime_app::core::DocumentId,
        dokime_app::document::DocumentDefinition,
    >,
    scope: &RecordScope,
) -> harmos::Result<usize> {
    let mut bytes = slots::<v1::HistoricalDocument>(source.len());
    for document in source.values() {
        bytes = bytes.saturating_add(
            document::construction_memory(document, scope)?
                - std::mem::size_of::<v1::HistoricalDocument>(),
        );
    }
    bounded(bytes)
}

fn snapshot_heap(source: &ExecutionSnapshot, scope: &RecordScope) -> harmos::Result<usize> {
    let mut bytes = case_heap(source.case())?
        .saturating_add(source.host_version().map_or(0, str::len))
        .saturating_add(source.template_id().map_or(0, |id| id.as_str().len()))
        .saturating_add(
            catalog::parameters_memory(source.parameter_definitions(), &mut Budget::new())?
                - std::mem::size_of::<Vec<v1::ParameterSchema>>(),
        )
        .saturating_add(materialized(source.parameters())?)
        .saturating_add(documents(source.documents(), scope)?);
    if let Some(suite) = source.suite() {
        bytes = bytes.saturating_add(metadata_heap(suite)?);
    }
    if let Some(template) = source.template() {
        bytes = bytes.saturating_add(metadata_heap(&template.metadata)?);
    }
    if let Some(plugin) = source.plugin() {
        bytes = bytes.saturating_add(plugin_heap(plugin));
    }
    bounded(bytes)
}

fn execution_heap(source: &TestExecution) -> harmos::Result<usize> {
    let mut bytes = source
        .test_execution_id
        .as_str()
        .len()
        .saturating_add(source.test_case_id.as_str().len())
        .saturating_add(scope_heap(&source.scope)?)
        .saturating_add(snapshot_heap(&source.snapshot, &source.scope)?)
        .saturating_add(materialized(&source.output_parameters)?)
        .saturating_add(documents(&source.documents, &source.scope)?);
    if let Some(suite) = &source.suite {
        bytes = bytes.saturating_add(metadata_heap(suite)?);
    }
    bytes = bytes.saturating_add(slots::<v1::HealthItem>(source.records.health().len()));
    for health in source.records.health() {
        bytes = bytes.saturating_add(
            notes::historical_health_memory(health)? - std::mem::size_of::<v1::HealthItem>(),
        );
    }
    bytes = bytes.saturating_add(slots::<v1::OperatorNote>(source.records.notes().len()));
    for note in source.records.notes() {
        bytes = bytes.saturating_add(
            notes::historical_note_memory(note)? - std::mem::size_of::<v1::OperatorNote>(),
        );
    }
    bytes = bytes.saturating_add(slots::<v1::HistoricalStep>(source.steps.len()));
    for step in &source.steps {
        bytes = bytes
            .saturating_add(step.step_id.as_str().len())
            .saturating_add(metadata_heap(&step.metadata)?)
            .saturating_add(slots::<v1::StepMeasurement>(step.measurements.len()))
            .saturating_add(slots::<v1::StepCriterion>(step.criteria.len()));
        for measurement in &step.measurements {
            bytes = bytes
                .saturating_add(metadata_heap(&measurement.metadata)?)
                .saturating_add(value_heap(&measurement.value, &mut Budget::new())?);
            if let Some(unit) = &measurement.unit {
                bytes = bytes.saturating_add(
                    catalog::unit_memory(unit)? - std::mem::size_of::<v1::MeasurementUnit>(),
                );
            }
        }
        for criterion in &step.criteria {
            bytes = bytes
                .saturating_add(criterion.criterion_id.as_str().len())
                .saturating_add(metadata_heap(&criterion.metadata)?)
                .saturating_add(optional(&criterion.message));
            if let Some(observed) = &criterion.observed {
                bytes = bytes.saturating_add(value_heap(observed, &mut Budget::new())?);
            }
        }
    }
    bytes = bytes.saturating_add(slots::<v1::HistoricalStepAttempt>(
        source.step_attempts.len(),
    ));
    for attempt in &source.step_attempts {
        bytes = bytes
            .saturating_add(attempt.attempt_id.as_str().len())
            .saturating_add(attempt.step_id.as_str().len());
        if let Some(detail) = &attempt.detail {
            bytes = bytes
                .saturating_add(detail.message.len())
                .saturating_add(detail.location.len())
                .saturating_add(detail.backtrace.len())
                .saturating_add(slots::<String>(detail.chain.len()))
                .saturating_add(slots::<String>(detail.span_trace.len()));
            for field in detail.chain.iter().chain(&detail.span_trace) {
                bytes = bytes.saturating_add(field.len());
            }
        }
    }
    if let Some(start) = &source.active_step_attempt {
        bytes = bytes
            .saturating_add(start.attempt_id.as_str().len())
            .saturating_add(start.step_id.as_str().len());
    }
    if let Some(invalidation) = &source.invalidation {
        bytes = bytes.saturating_add(plugin_heap(&invalidation.previous));
        if let Some(current) = &invalidation.current {
            bytes = bytes.saturating_add(plugin_heap(current));
        }
    }
    bounded(bytes)
}

/* === Tests === */
