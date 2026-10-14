/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Memory accounting for projected execution checkpoint contexts.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use tonic::Status;
// Local Imports
use crate::grpc::history::{construction::checkpoint_heap, document::canonical as document};
use crate::grpc::{
    appearance, notes, prompts,
    value::{self, canonical},
};
use dokime_app::{
    core::{Metadata, RecordScope},
    runtime::query::execution_history::ExecutionCheckpointContext,
};
use dokime_contract::operations::{
    appearance as appearance_wire, common, history, notes as note_wire, prompts as prompt_wire,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn count(source: &ExecutionCheckpointContext) -> harmos::Result<usize> {
    let mut bytes = std::mem::size_of::<history::CheckpointContext>()
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
        .saturating_add(slots::<history::HistoricalDocument>(
            source.execution_documents.len(),
        ))
        .saturating_add(slots::<prompt_wire::PromptInstance>(source.prompts.len()))
        .saturating_add(slots::<note_wire::HealthItem>(source.health.len()))
        .saturating_add(slots::<note_wire::OperatorNote>(source.notes.len()))
        .saturating_add(slots::<common::TraceabilitySnapshot>(
            source.traceability.len(),
        ))
        .saturating_add(
            appearance::historical_memory(&source.appearance)?
                - std::mem::size_of::<appearance_wire::AppearanceSnapshot>(),
        );
    for document in source.execution_documents.values() {
        bytes = bytes.saturating_add(
            document::construction_memory(document, &source.scope)?
                - std::mem::size_of::<history::HistoricalDocument>(),
        );
    }
    for prompt in &source.prompts {
        let memory = prompts::canonical::instance_memory(prompt).map_err(|error| {
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
        bytes = bytes.saturating_add(memory - std::mem::size_of::<prompt_wire::PromptInstance>());
    }
    for health in &source.health {
        bytes = bytes.saturating_add(
            notes::health_memory(health)? - std::mem::size_of::<note_wire::HealthItem>(),
        );
    }
    for note in &source.notes {
        bytes = bytes.saturating_add(
            notes::note_memory(note)? - std::mem::size_of::<note_wire::OperatorNote>(),
        );
    }
    for snapshot in &source.traceability {
        bytes = bytes.saturating_add(
            super::super::wire::traceability_memory(snapshot).map_err(failure)?
                - std::mem::size_of::<common::TraceabilitySnapshot>(),
        );
    }
    bounded(bytes)
}

/* === Private Functions === */
fn cycle_heap(source: &dokime_app::runtime::ExecutionCycleContext) -> harmos::Result<usize> {
    let mut bytes = source
        .cycle
        .test_cycle_id
        .as_str()
        .len()
        .saturating_add(metadata_heap(&source.cycle.metadata)?)
        .saturating_add(
            slots::<dokime_contract::operations::execution::CaseSummary>(source.cases.len()),
        )
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
                canonical::metadata_memory(&case.metadata).map_err(failure)?
                    - std::mem::size_of::<common::Metadata>(),
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
    length
        .saturating_mul(4)
        .saturating_mul(std::mem::size_of::<T>())
}

fn metadata_heap(source: &Metadata) -> harmos::Result<usize> {
    Ok(canonical::metadata_memory(source).map_err(failure)?
        - std::mem::size_of::<common::Metadata>())
}

fn scope_heap(source: &RecordScope) -> harmos::Result<usize> {
    Ok(canonical::scope_memory(source).map_err(failure)?
        - std::mem::size_of::<common::EvidenceScope>())
}

fn failure(source: Status) -> harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return harmos::Error::ResourceExhausted {
            resource: "canonical checkpoint context",
        };
    }
    harmos::Error::backend(
        "count original checkpoint context",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

/* === Tests === */
