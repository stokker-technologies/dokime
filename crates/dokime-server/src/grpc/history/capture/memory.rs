/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Memory accounting for projected execution capture checkpoints.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use tonic::Status;
// Local Imports
use super::super::construction::checkpoint_heap;
use crate::grpc::history::document::canonical as document;
use crate::grpc::{
    catalog, notes,
    value::{Budget, canonical},
};
use dokime_app::core::{Metadata, RecordScope, Value};
use dokime_app::definition::{MaterializedParameter, TestCase};
use dokime_app::execution::{
    ExecutionImplementation, ExecutionPluginIdentity, ExecutionSnapshot, TestExecution,
};
use dokime_app::runtime::query::execution_history::ExecutionCheckpointDocument;
use dokime_contract::operations::{
    catalog as catalog_wire, common as common_wire, execution as execution_wire,
    history as history_wire, notes as notes_wire, schema as schema_wire, value as value_wire,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn count(source: &ExecutionCheckpointDocument) -> harmos::Result<usize> {
    let bytes = std::mem::size_of::<history_wire::CheckpointDocument>()
        .saturating_add(checkpoint_heap(&source.checkpoint))
        .saturating_add(source.document.cycle.cycle_id.as_str().len())
        .saturating_add(metadata_heap(&source.document.cycle.metadata)?)
        .saturating_add(execution_heap(&source.document.execution)?);
    bounded(bytes)
}

/* === Private Functions === */
fn bounded(bytes: usize) -> harmos::Result<usize> {
    if bytes == usize::MAX {
        return Err(harmos::Error::ResourceExhausted {
            resource: "checkpoint document construction",
        });
    }
    Ok(bytes)
}

fn slots<T>(length: usize) -> usize {
    length
        .saturating_mul(4)
        .saturating_mul(std::mem::size_of::<T>())
}

fn optional(source: &Option<String>) -> usize {
    source.as_ref().map_or(0, String::len)
}

fn metadata_heap(source: &Metadata) -> harmos::Result<usize> {
    Ok(canonical::metadata_memory(source).map_err(failure)?
        - std::mem::size_of::<common_wire::Metadata>())
}

fn scope_heap(source: &RecordScope) -> harmos::Result<usize> {
    Ok(canonical::scope_memory(source).map_err(failure)?
        - std::mem::size_of::<common_wire::EvidenceScope>())
}

fn value_heap(source: &Value, budget: &mut Budget) -> harmos::Result<usize> {
    Ok(canonical::memory(source, budget, 0).map_err(failure)?
        - std::mem::size_of::<value_wire::Value>())
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
        .saturating_add(slots::<execution_wire::CaseParameter>(
            source.parameters.len(),
        ));
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
    let mut bytes = slots::<execution_wire::MaterializedParameter>(parameters.len());
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
                catalog::unit_memory(unit)? - std::mem::size_of::<schema_wire::MeasurementUnit>(),
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
    let mut bytes = slots::<history_wire::HistoricalDocument>(source.len());
    for document in source.values() {
        bytes = bytes.saturating_add(
            document::construction_memory(document, scope)?
                - std::mem::size_of::<history_wire::HistoricalDocument>(),
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
                - std::mem::size_of::<Vec<catalog_wire::ParameterSchema>>(),
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
    bytes = bytes.saturating_add(slots::<notes_wire::HealthItem>(
        source.records.health().len(),
    ));
    for health in source.records.health() {
        bytes = bytes.saturating_add(
            notes::health_memory(health)? - std::mem::size_of::<notes_wire::HealthItem>(),
        );
    }
    bytes = bytes.saturating_add(slots::<notes_wire::OperatorNote>(
        source.records.notes().len(),
    ));
    for note in source.records.notes() {
        bytes = bytes.saturating_add(
            notes::note_memory(note)? - std::mem::size_of::<notes_wire::OperatorNote>(),
        );
    }
    bytes = bytes.saturating_add(slots::<history_wire::HistoricalStep>(source.steps.len()));
    for step in &source.steps {
        bytes = bytes
            .saturating_add(step.step_id.as_str().len())
            .saturating_add(metadata_heap(&step.metadata)?)
            .saturating_add(slots::<execution_wire::StepMeasurement>(
                step.measurements.len(),
            ))
            .saturating_add(slots::<execution_wire::StepCriterion>(step.criteria.len()));
        for measurement in &step.measurements {
            bytes = bytes
                .saturating_add(metadata_heap(&measurement.metadata)?)
                .saturating_add(value_heap(&measurement.value, &mut Budget::new())?);
            if let Some(unit) = &measurement.unit {
                bytes = bytes.saturating_add(
                    catalog::unit_memory(unit)?
                        - std::mem::size_of::<schema_wire::MeasurementUnit>(),
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
    bytes = bytes.saturating_add(slots::<history_wire::HistoricalStepAttempt>(
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

fn failure(source: Status) -> harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return harmos::Error::ResourceExhausted {
            resource: "canonical captured execution document",
        };
    }
    harmos::Error::backend(
        "count original captured execution document",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

/* === Tests === */
