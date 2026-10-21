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
use std::{
    collections::{BTreeMap, btree_map::Entry},
    io::{self, Write},
    sync::Arc,
};

// Third Party Imports
use harmos::MemorySize;

// Local Imports
use super::{MAX_BYTES, error, owns_annotation, owns_projection};
use crate::RuntimeState;
use crate::definition::TestCase;
use crate::runtime::Result;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::DocumentId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(in crate::runtime::storage) struct CycleSource {
    cycle: Arc<CycleView>,
    budget: Budget,
}

pub(super) struct CaseSelection<'a> {
    rows: Vec<&'a TestCase>,
    retained_bytes: usize,
    selection_bytes: usize,
}

#[derive(Clone)]
struct Budget {
    remaining: usize,
    allocation_remaining: usize,
    depth: usize,
    quoted: bool,
    escaped: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl CycleSource {}

impl CaseSelection<'_> {
    pub(super) fn clone_values(&self) -> Result<Vec<TestCase>> {
        clone_rows(|| self.rows.iter().copied())
    }
}

impl Budget {
    fn new(remaining: usize) -> Self {
        Self { remaining, allocation_remaining: remaining.saturating_mul(8), depth: 0, quoted: false, escaped: false }
    }
    fn source(&mut self, value: &impl serde::Serialize) -> Result<()> {
        serde_json::to_writer(self, value).map_err(error)
    }
}

impl Write for Budget {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.remaining =
            self.remaining.checked_sub(bytes.len()).ok_or_else(|| io::Error::other("presentation exceeds 4 MiB source budget"))?;
        // JSON punctuation is emitted in separate writes; it is not a separate
        // allocation. Keep encoded bytes and conservative allocation work as
        // independent budgets so ordinary authored documents can use the full
        // persisted 4 MiB limit without being rejected by punctuation overhead.
        self.allocation_remaining = self
            .allocation_remaining
            .checked_sub(bytes.len().saturating_add(64))
            .ok_or_else(|| io::Error::other("presentation exceeds allocation budget"))?;
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

/* === Public Functions === */

pub(super) fn preflight(
    state: &RuntimeState, cycle: &CycleView, execution: &TestExecution, brand: Option<&crate::installed_plugins::ResolvedBrand>,
    selection: (&RecordScope, &DocumentId), cases: &CaseSelection<'_>, prepared: Option<&CycleSource>,
) -> Result<()> {
    let (scope, default_id) = selection;
    let mut budget = match prepared {
        Some(prepared) if std::ptr::eq(cycle, prepared.cycle.as_ref()) => prepared.budget.clone(),
        Some(_) => {
            return Err(error("prepared presentation source belongs to another cycle snapshot"));
        }
        None => cycle_budget(cycle)?,
    };
    budget.source(&execution.test_execution_id)?;
    budget.source(&execution.scope)?;
    budget.source(&execution.documents)?;
    budget.source(&execution.traceability)?;
    budget.source(execution.snapshot.parameters())?;
    budget.source(&execution.output_parameters)?;
    budget.source(&state.run_parameters)?;
    // Registered presets can be applied to each of the two default documents.
    let mut remaining = dokime_domain::model::execution::notes::NOTE_SCAN_LIMIT;
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
    if state.prompt_requests.len() > dokime_domain::model::execution::notes::NOTE_SCAN_LIMIT {
        return Err(error("presentation prompt scan budget exceeded"));
    }
    for prompt in prompt_rows(state, scope) {
        budget.source(prompt)?;
    }
    if cases.retained_bytes.checked_add(cases.selection_bytes).is_none_or(|bytes| bytes > MAX_BYTES) {
        return Err(error("presentation cases exceed construction budget"));
    }
    if documents_memory(state, execution, default_id)? > MAX_BYTES {
        return Err(error("presentation documents exceed construction budget"));
    }
    let (_, order) = rows_memory(execution_rows(cycle, execution))?;
    if order > MAX_BYTES {
        return Err(error("presentation execution order exceeds construction budget"));
    }
    let (stated, seed) = crate::traceability::stated_execution_memory(execution, &cycle.updated_at)?;
    if stated.checked_add(seed).is_none_or(|bytes| bytes > MAX_BYTES) {
        return Err(error("authored traceability exceeds construction budget"));
    }
    if annotations_memory(state, scope)? > MAX_BYTES {
        return Err(error("presentation annotations exceed construction budget"));
    }
    crate::installed_plugins::appearance::preflight_appearance(state, brand, budget.remaining).map_err(error)?;
    Ok(())
}

pub(super) fn documents_memory(state: &RuntimeState, execution: &TestExecution, default_id: &DocumentId) -> Result<usize> {
    let ids = execution.documents.keys().filter(|id| *id != default_id).chain(std::iter::once(default_id));
    let mut count = 0usize;
    let mut bytes = default_id.memory_size();
    if bytes == usize::MAX {
        return Err(error("default document selector memory overflows"));
    }
    for id in ids {
        let Some(construction) = crate::documents::construction_memory(&state.suites, execution, id) else {
            continue;
        };
        let key = id.memory_size();
        if construction == usize::MAX || key == usize::MAX {
            return Err(error("document construction memory overflows"));
        }
        count = count.checked_add(1).ok_or_else(|| error("document count overflows"))?;
        bytes = bytes
            .checked_add(construction.saturating_sub(std::mem::size_of::<crate::document::DocumentDefinition>()))
            .and_then(|bytes| bytes.checked_add(id.memory_size().saturating_sub(std::mem::size_of::<DocumentId>())))
            .ok_or_else(|| error("document construction memory overflows"))?;
    }
    bytes
        .checked_add(harmos::btree_map_memory::<DocumentId, crate::document::DocumentDefinition>(count))
        .ok_or_else(|| error("document map construction memory overflows"))
}

pub(super) fn select_cases<'a>(cycle: &'a CycleView, terminal: Option<&'a TestExecution>) -> Result<CaseSelection<'a>> {
    let members = cycle.test_executions.len();
    if members > dokime_domain::model::execution::notes::NOTE_SCAN_LIMIT {
        return Err(error("presentation case selection exceeds scan budget"));
    }
    let slots = members.checked_add(usize::from(terminal.is_some())).ok_or_else(|| error("presentation case slots overflow"))?;
    let references = slots
        .checked_mul(std::mem::size_of::<&TestCase>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<CaseSelection<'_>>()))
        .ok_or_else(|| error("presentation case selection memory overflows"))?;
    let selection_bytes = harmos::btree_map_memory::<&CaseId, &TestCase>(members)
        .checked_add(references)
        .ok_or_else(|| error("presentation case selection memory overflows"))?;
    if selection_bytes > MAX_BYTES {
        return Err(error("presentation case selection exceeds construction budget"));
    }

    let mut rows = Vec::new();
    rows.try_reserve_exact(slots).map_err(error)?;
    if rows.capacity() > slots {
        return Err(error("presentation case reference slots exceed original bound"));
    }
    let mut seen = BTreeMap::new();
    let mut retained_bytes = std::mem::size_of::<Vec<TestCase>>();
    for execution in &cycle.test_executions {
        let case = execution.snapshot.case();
        if let Entry::Vacant(entry) = seen.entry(&case.test_case_id) {
            let bytes = case.memory_size();
            if bytes == usize::MAX {
                return Err(error("presentation case memory is unbounded"));
            }
            retained_bytes = retained_bytes.checked_add(bytes).ok_or_else(|| error("presentation case memory overflows"))?;
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
            let bytes = case.memory_size();
            if bytes == usize::MAX {
                return Err(error("presentation terminal case memory is unbounded"));
            }
            retained_bytes = retained_bytes.checked_add(bytes).ok_or_else(|| error("presentation case memory overflows"))?;
            rows.push(case);
        }
    }
    if retained_bytes.checked_add(selection_bytes).is_none_or(|bytes| bytes > MAX_BYTES) {
        return Err(error("presentation cases exceed construction budget"));
    }
    Ok(CaseSelection { rows, retained_bytes, selection_bytes })
}

pub(super) fn execution_rows<'a>(
    cycle: &'a CycleView, execution: &'a TestExecution,
) -> impl Iterator<Item = &'a dokime_domain::model::identity::TestExecutionId> {
    let present = cycle.test_executions.iter().any(|member| member.test_execution_id == execution.test_execution_id);
    cycle.test_executions.iter().map(|member| &member.test_execution_id).chain((!present).then_some(&execution.test_execution_id))
}

pub(super) fn annotations_memory(state: &RuntimeState, scope: &RecordScope) -> Result<usize> {
    let (_, prompts) = rows_memory(prompt_rows(state, scope))?;
    let (_, notes) = rows_memory(note_rows(state, scope))?;
    let (_, health) = rows_memory(health_rows(state, scope))?;
    prompts.checked_add(notes).and_then(|bytes| bytes.checked_add(health)).ok_or_else(|| error("annotation construction memory overflows"))
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
    T: Clone + MemorySize + 'a,
    I: Iterator<Item = &'a T>,
{
    let (count, bytes) = rows_memory(rows())?;
    if bytes > MAX_BYTES {
        return Err(error("captured rows exceed construction budget"));
    }
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(error)?;
    if values.capacity() > count {
        return Err(error("captured row slots exceed construction budget"));
    }
    for value in rows() {
        if values.len() == count {
            return Err(error("row selection changed during capture"));
        }
        values.push(value.clone());
    }
    if values.len() != count || values.memory_size() > bytes {
        return Err(error("captured values exceed original construction bound"));
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

pub(super) fn grouping_memory(cycle: &CycleView) -> usize {
    let CycleView {
        saved,
        test_cycle_id,
        session_id,
        metadata,
        phase: _,
        lifecycle: _,
        created_at,
        started_at,
        updated_at,
        test_executions: _,
        records,
        continue_automatically: _,
        result: _,
    } = cycle;
    let fields = [
        (saved.memory_size(), std::mem::size_of_val(saved)),
        (test_cycle_id.memory_size(), std::mem::size_of_val(test_cycle_id)),
        (session_id.memory_size(), std::mem::size_of_val(session_id)),
        (metadata.memory_size(), std::mem::size_of_val(metadata)),
        (created_at.memory_size(), std::mem::size_of_val(created_at)),
        (started_at.memory_size(), std::mem::size_of_val(started_at)),
        (updated_at.memory_size(), std::mem::size_of_val(updated_at)),
        (records.memory_size(), std::mem::size_of_val(records)),
    ];
    fields.into_iter().fold(std::mem::size_of::<CycleView>(), |bytes, (owned, inline)| {
        if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
    })
}

/* === Private Functions === */

fn rows_memory<'a, T: MemorySize + 'a>(mut rows: impl Iterator<Item = &'a T>) -> Result<(usize, usize)> {
    rows.try_fold((0usize, std::mem::size_of::<Vec<T>>()), |(count, bytes), row| {
        let count = count.checked_add(1).ok_or_else(|| error("captured row count overflows"))?;
        let bytes = bytes.checked_add(row.memory_size()).ok_or_else(|| error("annotation construction memory overflows"))?;
        Ok((count, bytes))
    })
}

fn cycle_budget(cycle: &CycleView) -> Result<Budget> {
    let mut budget = Budget::new(MAX_BYTES);
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
    if cycle.test_executions.len() > dokime_domain::model::execution::notes::NOTE_SCAN_LIMIT {
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
    let construction = grouping_memory(cycle);
    if construction == usize::MAX || construction > MAX_BYTES {
        return Err(error("presentation grouping exceeds construction budget"));
    }
    Ok(budget)
}

/* === Tests === */
