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
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

// Local Imports
use crate::definition::MaterializedParameter;
use crate::expressions::DataSource;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Default)]
pub(crate) struct Context<'a> {
    executions: Executions<'a>,
    selection: Option<&'a RecordScope>,
}

#[derive(Clone, Copy, Serialize)]
pub(super) struct Entry<'a> {
    pub scope: &'a RecordScope,
    pub dimensions: &'a BTreeMap<String, Value>,
    pub inputs: &'a BTreeMap<String, MaterializedParameter>,
    pub outputs: &'a BTreeMap<String, MaterializedParameter>,
}

/* === Enums === */

#[derive(Clone, Copy)]
enum Executions<'a> {
    Slice(&'a [TestExecution]),
    Ordered { ids: &'a [TestExecutionId], values: &'a BTreeMap<TestExecutionId, TestExecution> },
}

/* === Traits === */

/* === Implementations === */

impl<'a> Context<'a> {
    pub(crate) fn new(executions: &'a [TestExecution], selection: &'a RecordScope) -> Self {
        Self { executions: Executions::Slice(executions), selection: Some(selection) }
    }

    pub(crate) fn ordered(
        ids: &'a [TestExecutionId], values: &'a BTreeMap<TestExecutionId, TestExecution>, selection: &'a RecordScope,
    ) -> Option<Self> {
        let mut seen = BTreeSet::new();
        if ids.iter().any(|id| {
            !seen.insert(id)
                || values.get(id).is_none_or(|execution| {
                    &execution.test_execution_id != id
                        || execution.scope.test_execution_id.as_ref() != Some(id)
                        || execution.scope.test_case_id.as_ref() != Some(&execution.test_case_id)
                        || execution.scope.test_cycle_id.as_ref() != Some(&execution.test_cycle_id)
                        || !member(&execution.scope, selection)
                })
        }) {
            return None;
        }
        Some(Self { executions: Executions::Ordered { ids, values }, selection: Some(selection) })
    }

    pub(super) fn entries(self) -> impl DoubleEndedIterator<Item = Entry<'a>> {
        self.executions.iter().filter(move |execution| self.selection.is_none_or(|scope| member(&execution.scope, scope))).map(Entry::from)
    }

    pub(super) fn available(self) -> bool {
        self.selection.is_some()
    }

    pub(super) fn cases(self) -> impl Iterator<Item = Entry<'a>> {
        self.executions.iter().enumerate().filter_map(move |(index, execution)| {
            let selected = |candidate: &TestExecution| self.selection.is_none_or(|scope| member(&candidate.scope, scope));
            let same_case = |candidate: &TestExecution| {
                candidate.test_case_id == execution.test_case_id && candidate.test_cycle_id == execution.test_cycle_id
            };
            if !selected(execution) || self.executions.iter().take(index).any(|candidate| selected(candidate) && same_case(candidate)) {
                return None;
            }
            self.executions.iter().rev().find(|candidate| selected(candidate) && same_case(candidate)).map(Entry::from)
        })
    }

    pub(super) fn len(self) -> usize {
        self.executions.len()
    }

    pub(crate) fn supplies(self, source: &DataSource) -> bool {
        let mut entries = self.entries();
        entries.next().is_some_and(|entry| entry.value(source).is_some()) && entries.all(|entry| entry.value(source).is_some())
    }

    pub(super) fn for_scope(self, scope: Option<&RecordScope>) -> Option<Entry<'a>> {
        let scope = scope?;
        let id = scope.test_execution_id.as_ref()?;
        self.entries().find(|entry| entry.scope.test_execution_id.as_ref() == Some(id) && matches(scope, entry.scope))
    }
}

impl Default for Executions<'_> {
    fn default() -> Self {
        Self::Slice(&[])
    }
}

impl<'a> Executions<'a> {
    fn iter(self) -> impl DoubleEndedIterator<Item = &'a TestExecution> {
        let (slice, ids, values) = match self {
            Self::Slice(slice) => (slice, &[][..], None),
            Self::Ordered { ids, values } => (&[][..], ids, Some(values)),
        };
        slice.iter().chain(ids.iter().filter_map(move |id| values?.get(id)))
    }

    fn len(self) -> usize {
        match self {
            Self::Slice(values) => values.len(),
            Self::Ordered { ids, .. } => ids.len(),
        }
    }
}

impl Serialize for Context<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(None)?;
        for entry in self.entries() {
            sequence.serialize_element(&entry)?;
        }
        sequence.end()
    }
}

impl<'a> Entry<'a> {
    pub(super) fn covers(self, scope: Option<&RecordScope>) -> bool {
        scope.is_some_and(|scope| {
            self.scope.test_execution_id.is_some() && scope.test_execution_id == self.scope.test_execution_id && matches(scope, self.scope)
        })
    }

    pub(super) fn value(self, source: &DataSource) -> Option<&'a Value> {
        let name = match source {
            DataSource::Parameter { name } | DataSource::Dimension { name } => name,
            _ => return None,
        };
        let parameter = self.outputs.get(name).or_else(|| self.inputs.get(name));
        if let Some(parameter) = parameter {
            return Some(&parameter.value);
        }
        match source {
            DataSource::Dimension { .. } => self.dimensions.get(name),
            _ => None,
        }
    }
}

impl<'a> From<&'a TestExecution> for Entry<'a> {
    fn from(execution: &'a TestExecution) -> Self {
        Self {
            scope: &execution.scope,
            dimensions: &execution.snapshot.case().parameters,
            inputs: execution.snapshot.parameters(),
            outputs: &execution.output_parameters,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn member(execution: &RecordScope, selected: &RecordScope) -> bool {
    scoped(execution, selected, true)
}

fn matches(actual: &RecordScope, selected: &RecordScope) -> bool {
    scoped(actual, selected, false)
}

fn scoped(actual: &RecordScope, selected: &RecordScope, planned: bool) -> bool {
    selected.test_execution_id.as_ref().is_none_or(|id| actual.test_execution_id.as_ref() == Some(id))
        && selected.test_cycle_id.as_ref().is_none_or(|id| actual.test_cycle_id.as_ref() == Some(id))
        && selected.test_case_id.as_ref().is_none_or(|id| actual.test_case_id.as_ref() == Some(id))
        && selected.template_id.as_ref().is_none_or(|id| actual.template_id.as_ref() == Some(id))
        && selected.session_id.as_ref().is_none_or(|id| actual.session_id.as_ref().map_or(planned, |session| session == id))
        && selected.suite_id.as_ref().is_none_or(|id| actual.suite_id.as_ref() == Some(id))
}

/* === Tests === */
