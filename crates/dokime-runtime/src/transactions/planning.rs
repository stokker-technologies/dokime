/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod append;
mod copy;

/* === Re-Exports === */
pub use append::AppendExecutions;
pub use copy::CopyCycle;

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::definition::TestCase;
use crate::records::BuiltinRecordType;
use crate::records::RecordSeverity;
use crate::transactions::records::draft::BuiltinDraft;
use crate::{runtime::Error, state::RuntimeState};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CycleStatus;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;
use dokime_domain::state::records::set::TestRecordSet;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CycleDefinition {
    pub metadata: Metadata,
    pub continue_automatically: bool,
    pub continue_on_failure: bool,
}

#[harmos::transaction(id = "dokime.suite.cases.register", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterCases {
    pub registration_id: CommandId,
    pub cases: Vec<TestCase>,
    pub definition_revisions: BTreeMap<dokime_domain::model::identity::CaseId, String>,
}

// A Cycle Opens Empty; Explicit Execution Identities Join Through AppendExecutions.
#[harmos::transaction(id = "dokime.execution.cycle.create", version = 4, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCycle {
    pub definition: CycleDefinition,
    pub test_cycle_id: TestCycleId,
    pub occurred_at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for RegisterCases {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        if self.cases.is_empty() || self.cases.len() > 1000 {
            return Err("registration requires 1 to 1000 cases".into());
        }
        if self.definition_revisions.len() != self.cases.len() {
            return Err("registration requires one authoritative revision per case".into());
        }
        state.suites.validate_registration(&self.registration_id, &self.cases)?;
        for case in &self.cases {
            let revision = state.case_definition_revision(case)?;
            if self.definition_revisions.get(&case.test_case_id) != Some(&revision) {
                return Err("case definition or provider changed during materialization".into());
            }
            if state.suites.case_revisions().get(&case.test_case_id).is_some_and(|known| known != &revision) {
                return Err("registered case definition is immutable".into());
            }
            let suite = state.suites.suite_for_case(case).ok_or("case owning suite is not installed")?;
            if case.suite.as_ref() != Some(&suite.suite.metadata)
                || !suite.suite.test_case_templates.iter().any(|template| template.test_case_template_id == case.test_case_template_id)
            {
                return Err("case template does not belong to its owning suite".into());
            }
        }
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.suites.register_selection(&self.registration_id, &self.cases).expect("Harmos checked immutable case registration");
        state.suites.capture_case_revisions(&self.definition_revisions);
    }
}

impl Transaction<RuntimeState> for CreateCycle {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        prepare_create_cycle(state, self).map(|_| ()).map_err(|error| error.to_string())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        prepare_create_cycle(state, self).expect("checked cycle plan").1.collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        apply_create_cycle(state, self).expect("Harmos checked cycle creation before applying it");
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn apply_create_cycle(state: &mut RuntimeState, create: &CreateCycle) -> Result<TestCycleId, Error> {
    let (cycle, records) = prepare_create_cycle(state, create)?;
    let cycle_id = cycle.test_cycle_id.clone();
    // Historical Seals Stay In Native Storage; Active Containers Stay Live.
    state.retire_sealed_cycles();
    state.replace_cycle(cycle).expect("cycle projection has valid ownership");
    let scope = RecordScope::default().with_cycle(cycle_id.clone());
    records.apply(state);
    state.record_health(
        scope,
        dokime_domain::model::execution::health::HealthSeverity::Info,
        "Cycle ready for execution.",
        Some("Provider execution is ready.".to_owned()),
        create.occurred_at.clone(),
    );
    Ok(cycle_id)
}

fn prepare_create_cycle(
    state: &RuntimeState, create: &CreateCycle,
) -> Result<(CycleView, crate::transactions::records::draft::Drafts), Error> {
    let cycle = empty_cycle(create.test_cycle_id.clone(), state.session_id.clone(), create.definition.clone(), create.occurred_at.clone());
    state.validate_cycle_view(&cycle).map_err(conflict)?;
    let mut records = crate::transactions::records::draft::Drafts::new(state);
    let scope = RecordScope::default().with_cycle(cycle.test_cycle_id.clone());
    records.draft(
        BuiltinDraft {
            record_type: BuiltinRecordType::TestCyclePrepared,
            source: "runtime".to_owned(),
            severity: RecordSeverity::Info,
            message: Some(format!("Cycle {} prepared.", cycle.metadata.label)),
            details: BTreeMap::from([("lifecycle".to_owned(), Value::from(format!("{:?}", cycle.lifecycle)))]),
            scope: Some(scope),
        },
        create.occurred_at.clone(),
    );
    Ok((cycle, records))
}

fn materialize_executions(
    state: &RuntimeState, cycle: &mut CycleView, cases: &[TestCase],
    ids: impl IntoIterator<Item = dokime_domain::model::identity::TestExecutionId>, settings: &BTreeMap<String, BTreeMap<String, Value>>,
) -> Result<(), Error> {
    for (case, id) in cases.iter().zip(ids) {
        let source = state.suites.suite_for_case(case).map(|suite| suite.source.as_str());
        let plugin = source.and_then(|source| state.execution_identity(source));
        let mut execution = super::provider::materialize_execution(
            state,
            cycle,
            case,
            id,
            plugin,
            source.and_then(|source| settings.get(source)).cloned().unwrap_or_default(),
        )?;
        execution.status = dokime_domain::model::execution::result::ExecutionStatus::Skipped;
        cycle.test_executions.push(execution);
    }
    Ok(())
}

fn empty_cycle(
    test_cycle_id: TestCycleId, session_id: dokime_domain::model::identity::SessionId, definition: CycleDefinition, at: Timestamp,
) -> CycleView {
    let CycleDefinition { metadata, continue_automatically, continue_on_failure } = definition;
    CycleView {
        saved: None,
        test_cycle_id,
        session_id: Some(session_id),
        metadata,
        phase: Default::default(),
        lifecycle: CycleStatus::Ready,
        created_at: at.clone(),
        started_at: None,
        updated_at: at,
        test_executions: Vec::new(),
        records: TestRecordSet::default(),
        continue_automatically,
        continue_on_failure,
        result: None,
    }
}

fn append_lifecycle(cycle: &CycleView, has_new_cases: bool) -> Result<CycleStatus, String> {
    cycle.ensure_active().map_err(str::to_owned)?;
    if matches!(cycle.lifecycle, CycleStatus::Running | CycleStatus::Paused | CycleStatus::Stopping) {
        return Ok(cycle.lifecycle);
    }
    if has_new_cases { cycle.lifecycle.try_transition(CycleStatus::Ready).map_err(|error| error.to_string()) } else { Ok(cycle.lifecycle) }
}

fn append_cases(cycle: &mut CycleView, cases: Vec<TestCase>, at: Timestamp) -> Result<(), String> {
    let has_new_cases = !cases.is_empty();
    let lifecycle = append_lifecycle(cycle, has_new_cases)?;
    for case in cases {
        if let Some(existing) = cycle.cases().find(|known| known.test_case_id == case.test_case_id)
            && existing != &case
        {
            return Err(format!("case {} is immutable", case.test_case_id));
        }
    }
    if has_new_cases {
        cycle.lifecycle = lifecycle;
        cycle.result = None;
    }
    cycle.touch_at(at);
    Ok(())
}

fn ensure_runtime_active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down { Err("runtime is shutting down".to_owned()) } else { Ok(()) }
}

fn conflict(message: String) -> Error {
    Error::conflict("conflict", message)
}

/* === Tests === */
