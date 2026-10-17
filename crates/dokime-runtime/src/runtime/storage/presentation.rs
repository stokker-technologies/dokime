/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod assets;
mod css;
mod source;

/* === Re-Exports === */
pub(super) use assets::capture_logo;
pub(super) use source::CycleSource;

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports
use crate::execution::context::{ExecutionContext, ExecutionCycleContext};
use crate::execution::{CycleView, TestExecution};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::execution::RecordScope;

/* === Constants === */
const MAX_BYTES: usize = 4 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Captured {
    pub(super) context: ExecutionContext,
    pub(super) logo: Option<crate::installed_plugins::RuntimeInstalledPluginAsset>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn capture_prepared(
    state: &crate::RuntimeState,
    cycle: &CycleView,
    execution: &TestExecution,
    brand: Option<&crate::installed_plugins::ResolvedBrand>,
    prepared: Option<&CycleSource>,
) -> Result<Captured> {
    if execution.test_cycle_id != cycle.test_cycle_id
        || execution.scope.test_cycle_id.as_ref() != Some(&execution.test_cycle_id)
        || execution.scope.test_case_id.as_ref() != Some(&execution.test_case_id)
        || execution.scope.test_execution_id.as_ref() != Some(&execution.test_execution_id)
    {
        return Err(error("execution presentation belongs to another cycle"));
    }

    if execution.test_case_id.as_str().len() > dokime_domain::model::execution::notes::NOTE_OWNER_BYTES {
        return Err(error("presentation case identity exceeds source budget"));
    }
    let cycle_scope = RecordScope::default().with_cycle(cycle.test_cycle_id.clone());
    let execution_scope = cycle_scope
        .clone()
        .with_test_case(execution.test_case_id.clone())
        .with_test_execution(execution.test_execution_id.clone());
    let default_id =
        DocumentId::from_identifier(crate::documents::DEFAULT_DOCUMENT_ID).map_err(error)?;
    let cases = source::select_cases(cycle, Some(execution))?;
    source::preflight(
        state,
        cycle,
        execution,
        brand,
        (&execution_scope, &default_id),
        &cases,
        prepared,
    )?;
    let mut execution_documents = BTreeMap::new();
    let ids = execution
        .documents
        .keys()
        .filter(|id| *id != &default_id)
        .chain(std::iter::once(&default_id));
    for id in ids {
        if let Some(definition) =
            crate::documents::captured_authored_document(&state.suites, execution, id)
        {
            execution_documents.insert(id.clone(), definition);
        }
    }
    let execution_order = source::clone_rows(|| source::execution_rows(cycle, execution))?;
    let grouping = source::grouping(cycle);
    let appearance = crate::installed_plugins::appearance::captured_appearance(state, brand);
    css::validate(&appearance)?;
    let logo = appearance
        .logo
        .as_ref()
        .map(|logo| {
            crate::installed_plugins::appearance::captured_logo_asset(state, brand, logo)
                .ok_or_else(|| error("selected appearance logo is unavailable"))
        })
        .transpose()?;
    let context = ExecutionContext {
        scope: execution.scope.clone(),
        session_id: cycle
            .session_id
            .clone()
            .unwrap_or_else(|| state.session_id.clone()),
        cycle: ExecutionCycleContext {
            cycle: grouping,
            cases: cases.clone_values()?,
            execution_order,
        },
        execution_documents,
        traceability: {
            let mut snapshots = crate::traceability::snapshots_from_execution_parameters(
                &state.run_parameters,
                execution,
                Some(&execution_scope),
                cycle.updated_at.clone(),
            );
            // Freeze Resource Evidence Before Appending Derived Terminal Facts
            snapshots.extend(crate::traceability::resource_traceability(
                state,
                cycle,
                execution_scope.clone(),
                cycle.updated_at.clone(),
            ));
            // What the template stated travels with the rest: the report and
            // every historical read of this execution go through this blob, so
            // a section left out here is a section nobody sees again.
            snapshots.extend(crate::traceability::stated_execution_snapshots(
                execution,
                &cycle.updated_at,
                MAX_BYTES,
            )?);
            snapshots.retain(|snapshot| {
                !snapshot.entries.iter().any(|entry| {
                    matches!(
                        entry.source.as_deref(),
                        Some(
                            "traceability.cycle"
                                | "traceability.execution"
                                | "traceability.environment"
                                | "traceability.test_definition"
                                | "traceability.test_execution"
                        )
                    )
                })
            });
            snapshots.extend(crate::traceability::execution_traceability(
                cycle,
                execution,
                cycle.updated_at.clone(),
            ));
            snapshots
        },
        prompts: source::clone_rows(|| source::prompt_rows(state, &execution_scope))?,
        notes: source::clone_rows(|| source::note_rows(state, &execution_scope))?,
        health: source::clone_rows(|| source::health_rows(state, &execution_scope))?,
        appearance,
    };
    Ok(Captured { context, logo })
}

// Annotation context also retains session/global observations. Export selection
// separately decides whether these belong to an explicitly cycle-scoped bundle.
pub(super) fn owns_annotation(scope: &RecordScope, execution: &RecordScope) -> bool {
    if (scope.step_id.is_some() || scope.criterion_id.is_some())
        && scope.test_execution_id.is_none()
    {
        return false;
    }
    (scope.test_cycle_id.is_none()
        && scope.test_case_id.is_none()
        && scope.test_execution_id.is_none())
        || owns_projection(scope, execution)
}

/* === Private Functions === */
// Each execution owns its own projections and copies only its cycle/case ancestors.
// Global items are deliberately outside the cycle export selection.
fn owns_projection(scope: &RecordScope, execution: &RecordScope) -> bool {
    (scope.test_cycle_id.is_some() || scope.test_execution_id.is_some())
        && scope
            .test_cycle_id
            .as_ref()
            .is_none_or(|id| Some(id) == execution.test_cycle_id.as_ref())
        && scope
            .test_case_id
            .as_ref()
            .is_none_or(|id| Some(id) == execution.test_case_id.as_ref())
        && scope
            .test_execution_id
            .as_ref()
            .is_none_or(|id| Some(id) == execution.test_execution_id.as_ref())
}

fn error(value: impl ToString) -> Error {
    Error::Operation(value.to_string())
}

/* === Tests === */
