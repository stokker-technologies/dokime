/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod copy;

/* === Re-Exports === */
pub use copy::CopyCycle;

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};
// Local Imports
use crate::definition::{TestCase, TestSuite};
use crate::execution::{CycleStatus, CycleView};
use crate::records::TestRecordSet;
use crate::records::{ProductRecordDraft, RecordSeverity};
use crate::{Error, state::RuntimeState};
use dokime::core::{CommandId, Metadata, RecordScope, TestCycleId, Timestamp, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CycleDefinition {
    pub suite: TestSuite,
    pub metadata: Metadata,
    pub cases: Vec<TestCase>,
    pub continue_on_failure: bool,
}

#[harmos::transaction(id = "dokime.suite.cases.register", version = 1, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterCases {
    pub registration_id: CommandId,
    pub cases: Vec<TestCase>,
}

#[harmos::transaction(id = "dokime.execution.cycle.create", version = 3, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateCycle {
    pub mode: CreateCycleMode,
    pub definition: CycleDefinition,
    pub test_cycle_id: TestCycleId,
    pub occurred_at: Timestamp,
    #[serde(default)]
    pub settings: BTreeMap<String, BTreeMap<String, Value>>,
}

#[harmos::transaction(id = "dokime.execution.cycle.append", version = 3, irreversible)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendCycle {
    pub admission_identifier: dokime::core::CommandId,
    pub test_cycle_id: TestCycleId,
    pub source: AppendCycleSource,
    pub occurred_at: Timestamp,
    #[serde(default)]
    pub settings: BTreeMap<String, BTreeMap<String, Value>>,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreateCycleMode {
    Preview,
    Start,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "source")]
pub enum AppendCycleSource {
    Cases {
        cases: Vec<TestCase>,
    },
    Templates {
        suite: Box<TestSuite>,
        cases: Vec<TestCase>,
    },
}

/* === Traits === */

/* === Implementations === */
impl Transaction<RuntimeState> for RegisterCases {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        if self.cases.is_empty() || self.cases.len() > 1000 {
            return Err("registration requires 1 to 1000 cases".into());
        }
        state
            .suites
            .validate_registration(&self.registration_id, &self.cases)?;
        for case in &self.cases {
            let suite = state
                .suites
                .suite_for_case(case)
                .ok_or("case owning suite is not installed")?;
            if case.suite.as_ref() != Some(&suite.suite.metadata)
                || !suite
                    .suite
                    .test_case_templates
                    .iter()
                    .any(|template| template.test_case_template_id == case.test_case_template_id)
            {
                return Err("case template does not belong to its owning suite".into());
            }
        }
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state
            .suites
            .register_selection(&self.registration_id, &self.cases)
            .expect("Harmos checked immutable case registration");
    }
}

impl Transaction<RuntimeState> for CreateCycle {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        prepare_create_cycle(state, self)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        prepare_create_cycle(state, self)
            .expect("checked cycle plan")
            .1
            .collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        apply_create_cycle(state, self).expect("Harmos checked cycle creation before applying it");
    }
}

impl Transaction<RuntimeState> for AppendCycle {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        ensure_runtime_active(state)?;
        prepare_append_cycle(state, self)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn apply(&self, state: &mut RuntimeState) {
        apply_append_cycle(state, self).expect("Harmos checked cycle append before applying it");
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn apply_create_cycle(
    state: &mut RuntimeState,
    create: &CreateCycle,
) -> Result<TestCycleId, Error> {
    let (cycle, records) = prepare_create_cycle(state, create)?;
    let cycle_id = cycle.test_cycle_id.clone();
    state
        .suites
        .register_cases(cycle.cases())
        .map_err(conflict)?;
    state
        .replace_cycle(cycle)
        .expect("cycle projection has valid ownership");
    if matches!(create.mode, CreateCycleMode::Start) {
        state.run_parameters.clear();
        let scope = RecordScope::default().with_cycle(cycle_id.clone());
        records.apply(state);
        state.record_health(
            scope,
            crate::execution::HealthSeverity::Info,
            "Cycle ready for execution.",
            Some("Provider execution is ready.".to_owned()),
            create.occurred_at.clone(),
        );
    }
    Ok(cycle_id)
}

fn prepare_create_cycle(
    state: &RuntimeState,
    create: &CreateCycle,
) -> Result<(CycleView, crate::transactions::records::draft::Drafts), Error> {
    let lifecycle = match create.mode {
        CreateCycleMode::Preview => CycleStatus::Previewed,
        CreateCycleMode::Start => CycleStatus::Ready,
    };
    let (mut cycle, selected_cases) = build_cycle(
        create.test_cycle_id.clone(),
        state.session_id.clone(),
        create.definition.clone(),
        lifecycle,
        create.occurred_at.clone(),
    );
    state
        .suites
        .validate_cases(&selected_cases)
        .map_err(conflict)?;
    materialize_cycle_executions(
        state,
        &mut cycle,
        &selected_cases,
        &create.settings,
        Some(&create.definition.suite),
        create.test_cycle_id.as_str(),
    )?;
    state.validate_cycle_view(&cycle).map_err(conflict)?;
    let mut records = crate::transactions::records::draft::Drafts::new(state);
    if matches!(create.mode, CreateCycleMode::Start) {
        let scope = RecordScope::default().with_cycle(cycle.test_cycle_id.clone());
        records.draft(
            ProductRecordDraft {
                record_type: dokime::core::Identifier::new("test.cycle.prepared")
                    .expect("built-in record identifier is valid"),
                source: "runtime".to_owned(),
                severity: RecordSeverity::Info,
                message: Some(format!("Cycle {} prepared.", cycle.metadata.label)),
                details: BTreeMap::from([(
                    "lifecycle".to_owned(),
                    Value::from(format!("{:?}", cycle.lifecycle)),
                )]),
                scope: Some(scope.clone()),
            },
            create.occurred_at.clone(),
        );
    }
    Ok((cycle, records))
}

fn apply_append_cycle(state: &mut RuntimeState, append: &AppendCycle) -> Result<(), Error> {
    apply_append_cycle_inner(state, append)
}

fn apply_append_cycle_inner(state: &mut RuntimeState, append: &AppendCycle) -> Result<(), Error> {
    let cycle = prepare_append_cycle(state, append)?;
    state
        .suites
        .register_cases(cycle.cases())
        .map_err(conflict)?;
    state
        .replace_cycle(cycle)
        .expect("cycle projection has valid ownership");
    Ok(())
}

fn prepare_append_cycle(state: &RuntimeState, append: &AppendCycle) -> Result<CycleView, Error> {
    let cases = match &append.source {
        AppendCycleSource::Cases { cases } | AppendCycleSource::Templates { cases, .. } => {
            cases.clone()
        }
    };
    let mut cycle = state
        .cycle_view(&append.test_cycle_id)
        .ok_or_else(|| Error::not_found("not_found", "cycle not found"))?;
    state.suites.validate_cases(&cases).map_err(conflict)?;
    append_cases(&mut cycle, cases.clone(), append.occurred_at.clone()).map_err(conflict)?;
    materialize_cycle_executions(
        state,
        &mut cycle,
        &cases,
        &append.settings,
        match &append.source {
            AppendCycleSource::Templates { suite, .. } => Some(suite.as_ref()),
            _ => None,
        },
        append.admission_identifier.as_str(),
    )?;
    state.validate_cycle_view(&cycle).map_err(conflict)?;
    Ok(cycle)
}

fn materialize_cycle_executions(
    state: &RuntimeState,
    cycle: &mut CycleView,
    selected_cases: &[TestCase],
    settings: &BTreeMap<String, BTreeMap<String, Value>>,
    authored_suite: Option<&TestSuite>,
    admission_identifier: &str,
) -> Result<(), Error> {
    for (ordinal, case) in selected_cases.iter().enumerate() {
        let source = state
            .suites
            .suite_for_case(case)
            .map(|suite| suite.source.as_str());
        let plugin = source.and_then(|source| state.execution_identity(source));
        let id = dokime::core::TestExecutionId::from_seed(format!(
            "cycle:{}:case:{}:admission:{}:ordinal:{ordinal}",
            cycle.test_cycle_id, case.test_case_id, admission_identifier
        ));
        let mut execution = super::provider::materialize_execution(
            state,
            cycle,
            case,
            authored_suite.filter(|suite| case.suite.as_ref() == Some(&suite.metadata)),
            id,
            plugin,
            source
                .and_then(|source| settings.get(source))
                .cloned()
                .unwrap_or_default(),
        )?;
        execution.status = crate::execution::ExecutionStatus::Skipped;
        cycle.test_executions.push(execution);
    }
    Ok(())
}

fn build_cycle(
    test_cycle_id: TestCycleId,
    session_id: dokime::core::SessionId,
    definition: CycleDefinition,
    lifecycle: CycleStatus,
    at: Timestamp,
) -> (CycleView, Vec<TestCase>) {
    let CycleDefinition {
        suite: _,
        metadata,
        cases,
        continue_on_failure,
    } = definition;
    (
        CycleView {
            saved: None,
            test_cycle_id,
            session_id: Some(session_id),
            metadata,
            lifecycle,
            created_at: at.clone(),
            started_at: None,
            updated_at: at,
            test_executions: Vec::new(),
            records: TestRecordSet::default(),
            continue_on_failure,
            result: None,
        },
        cases,
    )
}

fn append_lifecycle(cycle: &CycleView, has_new_cases: bool) -> Result<CycleStatus, String> {
    if matches!(
        cycle.lifecycle,
        CycleStatus::Running | CycleStatus::Paused | CycleStatus::Stopping | CycleStatus::Cleared
    ) {
        return Err("cycle cannot be modified from its current state".to_owned());
    }
    if has_new_cases && cycle.lifecycle != CycleStatus::Previewed {
        cycle
            .lifecycle
            .try_transition(CycleStatus::Ready)
            .map_err(|error| error.to_string())
    } else {
        Ok(cycle.lifecycle)
    }
}

fn append_cases(cycle: &mut CycleView, cases: Vec<TestCase>, at: Timestamp) -> Result<(), String> {
    let has_new_cases = !cases.is_empty();
    let lifecycle = append_lifecycle(cycle, has_new_cases)?;
    for case in cases {
        if let Some(existing) = cycle
            .cases()
            .find(|known| known.test_case_id == case.test_case_id)
            && existing != &case
        {
            return Err(format!("case {} is immutable", case.test_case_id));
        }
    }
    if has_new_cases && cycle.lifecycle != CycleStatus::Previewed {
        cycle.saved = None;
        cycle.lifecycle = lifecycle;
        cycle.result = None;
    }
    cycle.touch_at(at);
    Ok(())
}

fn ensure_runtime_active(state: &RuntimeState) -> Result<(), String> {
    if state.shutting_down {
        Err("runtime is shutting down".to_owned())
    } else {
        Ok(())
    }
}

fn conflict(message: String) -> Error {
    Error::conflict("conflict", message)
}

/* === Tests === */
