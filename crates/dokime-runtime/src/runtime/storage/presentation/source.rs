/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{
    collections::{BTreeMap, btree_map::Entry},
    io::{self, Write},
};

use super::{error, owns_annotation, owns_projection};
use crate::RuntimeState;
use crate::definition::TestCase;
use crate::runtime::Result;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::TestExecution;

pub(super) struct CaseSelection<'a> {
    rows: Vec<&'a TestCase>,
}

struct Budget {
    remaining: usize,
    depth: usize,
    quoted: bool,
    escaped: bool,
}

impl CaseSelection<'_> {
    pub(super) fn clone_values(&self) -> Result<Vec<TestCase>> {
        clone_rows(|| self.rows.iter().copied())
    }
}

impl Budget {
    fn new(remaining: usize) -> Self {
        Self {
            remaining,
            depth: 0,
            quoted: false,
            escaped: false,
        }
    }
    fn source(&mut self, value: &impl serde::Serialize) -> Result<()> {
        serde_json::to_writer(self, value).map_err(error)
    }
}

impl Write for Budget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.remaining =
            self.remaining.checked_sub(bytes.len()).ok_or_else(|| io::Error::other("presentation exceeds configured source budget"))?;
        for byte in bytes {
            if self.quoted {
                if self.escaped {
                    self.escaped = false;
                } else if *byte == b'\\' {
                    self.escaped = true;
                } else if *byte == b'"' {
                    self.quoted = false;
                }
            } else {
                match byte {
                    b'"' => self.quoted = true,
                    b'{' | b'[' => {
                        self.depth += 1;
                        if self.depth > 64 {
                            return Err(io::Error::other("presentation source exceeds depth 64"));
                        }
                    }
                    b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                    _ => {}
                }
            }
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn preflight(
    state: &RuntimeState, cycle: &CycleView, execution: &TestExecution, brand: Option<&crate::installed_plugins::ResolvedBrand>,
    scope: &RecordScope,
) -> Result<()> {
    let mut budget = cycle_budget(cycle)?;
    budget.source(&execution.test_execution_id)?;
    budget.source(&execution.scope)?;
    budget.source(&execution.documents)?;
    budget.source(&execution.traceability)?;
    budget.source(execution.snapshot.parameters())?;
    budget.source(&execution.output_parameters)?;
    budget.source(&state.run_parameters)?;
    // Registered presets can be applied to each of the two default documents.
    let mut remaining = 10_000_usize;
    for suite in state.suites.suites() {
        remaining = remaining
            .checked_sub(1usize.saturating_add(suite.suite.test_case_templates.len()))
            .ok_or_else(|| error("presentation template scan budget exceeded"))?;
    }
    budget.source(execution.snapshot.case())?;
    budget.source(&execution.snapshot.suite())?;
    budget.source(&execution.snapshot.plugin())?;
    budget.source(&execution.snapshot.host_version())?;
    budget.source(&execution.started_at)?;
    budget.source(&execution.finished_at)?;
    if let Some(template) = crate::documents::selected_execution_template(execution) {
        budget.source(&template.metadata.label)?;
    }
    // This also bounds empty owners and excluded entries without inspecting their
    // payloads. Historical capture need not have a still-current selected owner.
    for set in state.annotation_sets(None, true).map_err(error)? {
        for note in set.notes().iter().filter(move |item| owns_annotation(&item.scope, scope)) {
            budget.source(note)?;
        }
        for health in set.health().iter().filter(move |item| owns_annotation(&item.scope, scope)) {
            budget.source(health)?;
        }
    }
    if state.prompt_requests.len() > 10_000 {
        return Err(error("presentation prompt scan budget exceeded"));
    }
    for prompt in prompt_rows(state, scope) {
        budget.source(prompt)?;
    }
    crate::installed_plugins::appearance::preflight_appearance(state, brand, budget.remaining).map_err(error)?;
    Ok(())
}

pub(super) fn select_cases<'a>(cycle: &'a CycleView, terminal: Option<&'a TestExecution>) -> Result<CaseSelection<'a>> {
    let members = cycle.test_executions.len();
    if members > 10_000 {
        return Err(error("presentation case selection exceeds scan budget"));
    }
    let slots = members.checked_add(usize::from(terminal.is_some())).ok_or_else(|| error("presentation case slots overflow"))?;
    let mut rows = Vec::new();
    rows.try_reserve_exact(slots).map_err(error)?;
    let mut seen = BTreeMap::new();
    for execution in &cycle.test_executions {
        let case = execution.snapshot.case();
        if let Entry::Vacant(entry) = seen.entry(&case.test_case_id) {
            entry.insert(case);
            rows.push(case);
        }
    }
    if let Some(terminal) = terminal {
        let case = terminal.snapshot.case();
        if let Some(member) = seen.get(&terminal.test_case_id) {
            if *member != case {
                return Err(error("cycle case context conflicts with execution snapshot"));
            }
        } else {
            rows.push(case);
        }
    }
    Ok(CaseSelection { rows })
}

pub(super) fn execution_rows<'a>(
    cycle: &'a CycleView, execution: &'a TestExecution,
) -> impl Iterator<Item = &'a dokime_domain::model::identity::TestExecutionId> {
    let present = cycle.test_executions.iter().any(|member| member.test_execution_id == execution.test_execution_id);
    cycle.test_executions.iter().map(|member| &member.test_execution_id).chain((!present).then_some(&execution.test_execution_id))
}

pub(super) fn prompt_rows<'a>(state: &'a RuntimeState, scope: &'a RecordScope) -> impl Iterator<Item = &'a crate::prompts::PromptInstance> {
    state.open_prompts().filter(move |item| owns_projection(&item.scope, scope))
}

pub(super) fn note_rows<'a>(
    state: &'a RuntimeState, scope: &'a RecordScope,
) -> impl Iterator<Item = &'a dokime_domain::model::execution::notes::OperatorNote> {
    state.iter_notes().filter(move |item| owns_annotation(&item.scope, scope))
}

pub(super) fn health_rows<'a>(
    state: &'a RuntimeState, scope: &'a RecordScope,
) -> impl Iterator<Item = &'a dokime_domain::model::execution::health::HealthItem> {
    state.iter_health_items().filter(move |item| owns_annotation(&item.scope, scope))
}

pub(super) fn clone_rows<'a, T, I>(rows: impl Fn() -> I) -> Result<Vec<T>>
where
    T: Clone + 'a,
    I: Iterator<Item = &'a T>,
{
    let count = rows().count();
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(error)?;
    for value in rows() {
        if values.len() == count {
            return Err(error("row selection changed during capture"));
        }
        values.push(value.clone());
    }
    if values.len() != count {
        return Err(error("row selection changed during capture"));
    }
    Ok(values)
}

pub(super) fn grouping(cycle: &CycleView) -> CycleView {
    CycleView {
        saved: cycle.saved.clone(),
        test_cycle_id: cycle.test_cycle_id.clone(),
        session_id: cycle.session_id.clone(),
        metadata: cycle.metadata.clone(),
        phase: cycle.phase,
        lifecycle: cycle.lifecycle,
        created_at: cycle.created_at.clone(),
        started_at: cycle.started_at.clone(),
        updated_at: cycle.updated_at.clone(),
        test_executions: Vec::new(),
        records: cycle.records.clone(),
        continue_automatically: cycle.continue_automatically,
        result: cycle.result.clone(),
    }
}

fn cycle_budget(cycle: &CycleView) -> Result<Budget> {
    let maximum = 64 * 1024 * 1024;
    let mut budget = Budget::new(maximum);
    // Only retained grouping fields and selected parameter/document sources
    // are materialized; execution payloads stay in their independent datasets.
    budget.source(&(
        &cycle.saved,
        &cycle.test_cycle_id,
        &cycle.session_id,
        &cycle.metadata,
        &cycle.lifecycle,
        &cycle.created_at,
        &cycle.started_at,
        &cycle.updated_at,
        &cycle.records,
        &cycle.result,
    ))?;
    if cycle.test_executions.len() > 10_000 {
        return Err(error("presentation execution order exceeds scan budget"));
    }
    let cases = select_cases(cycle, None)?;
    for case in cases.rows {
        budget.source(case)?;
    }
    for execution in &cycle.test_executions {
        budget.source(&execution.test_execution_id)?;
        budget.source(&execution.suite)?;
    }
    Ok(budget)
}
