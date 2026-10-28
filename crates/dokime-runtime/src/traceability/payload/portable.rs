/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use super::{
    CycleFields, Entries, Entry, EntryMetadata, Groups, Scope, build_snapshot, collect_entries, collect_groups, emit_parameter, exhausted,
    owned_entry, reserved, visit_resource_entries, visit_section, visit_test_groups,
};
use crate::runtime::{Error, Result};
use dokime_domain::model::{
    execution::{
        RecordScope,
        result::{CycleView, TestExecution},
    },
    time::Timestamp,
    traceability::{TraceabilityEntry, TraceabilitySection, TraceabilitySnapshot},
    values::Value,
};

struct OneGroup {
    seen: bool,
    value: Option<TraceabilitySnapshot>,
}

struct OneEntry(Option<TraceabilityEntry>);

impl Groups for OneGroup {
    fn group(
        &mut self, scope: Scope<'_>, at: Option<&Timestamp>, metadata: Option<EntryMetadata<'_>>, seed: std::fmt::Arguments<'_>,
        keep_empty: bool, visit: &mut dyn FnMut(&mut dyn Entries) -> Result<()>,
    ) -> Result<()> {
        if self.seen {
            return Err(Error::Operation("single traceability constructor received multiple groups".into()));
        }
        self.seen = true;
        self.value = build_snapshot(scope, at, metadata, seed, keep_empty, visit)?;
        Ok(())
    }
}

impl Entries for OneEntry {
    fn push(&mut self, entry: Entry<'_>) -> Result<()> {
        if self.0.is_some() {
            return Err(Error::Operation("single traceability constructor received multiple entries".into()));
        }
        self.0 = Some(owned_entry(entry)?);
        Ok(())
    }
}

pub(in crate::traceability) fn execution_snapshots(
    cycle: &CycleView, scope: &RecordScope, admitted: Option<&TestExecution>, at: &Timestamp,
) -> Result<Vec<TraceabilitySnapshot>> {
    let execution = match &scope.test_execution_id {
        Some(id) => admitted
            .filter(|execution| &execution.test_execution_id == id)
            .or_else(|| cycle.test_executions.iter().find(|execution| &execution.test_execution_id == id)),
        None => admitted.filter(|execution| scope.test_case_id.as_ref().is_none_or(|id| id == &execution.test_case_id)).or_else(|| {
            cycle.test_executions.iter().rev().find(|execution| scope.test_case_id.as_ref().is_none_or(|id| id == &execution.test_case_id))
        }),
    }
    .filter(|execution| scope.test_case_id.as_ref().is_none_or(|id| id == &execution.test_case_id));
    let fields = CycleFields {
        created_at: &cycle.created_at,
        updated_at: &cycle.updated_at,
        label: &cycle.metadata.label,
        case_count: cycle.cases().count(),
    };
    let index = execution.and_then(|execution| cycle.cases().position(|case| case.test_case_id == execution.test_case_id));
    collect_groups(&mut |groups| visit_test_groups(fields, execution, index, Scope::Borrowed(scope), Some(at), groups))
}

pub(in crate::traceability) fn resource_entries(
    id: &str, label: &str, details: &BTreeMap<String, Value>, index: usize,
) -> Result<Vec<Vec<TraceabilityEntry>>> {
    let mut groups = reserved(3)?;
    for kind in 0..3 {
        let entries = collect_entries(&mut |sink| visit_resource_entries(id, label, details, index, kind, sink))?;
        if !entries.is_empty() {
            groups.push(entries);
        }
    }
    Ok(groups)
}

pub(in crate::traceability) fn parameter_entry(
    parameter: &crate::definition::MaterializedParameter, source: &str,
) -> Result<TraceabilityEntry> {
    let mut entry = OneEntry(None);
    emit_parameter(parameter, source, &mut entry)?;
    entry.0.ok_or_else(exhausted)
}

pub(in crate::traceability) fn section_snapshot(
    scope: &RecordScope, section: &TraceabilitySection, at: &Timestamp,
) -> Result<TraceabilitySnapshot> {
    collect_one(&mut |groups| visit_section(Scope::Borrowed(scope), section, Some(at), groups))?.ok_or_else(exhausted)
}

pub(in crate::traceability) fn host_snapshot(
    values: &BTreeMap<String, Value>, scope: &RecordScope, at: &Timestamp,
) -> Result<Option<TraceabilitySnapshot>> {
    collect_one(&mut |groups| super::context::visit_host(values, Scope::Borrowed(scope), Some(at), groups))
}

pub(in crate::traceability) fn lease_snapshot(
    leases: &[crate::telemetry_runtime::LeaseFact], scope: &RecordScope, at: &Timestamp,
) -> Result<Option<TraceabilitySnapshot>> {
    collect_one(&mut |groups| {
        super::context::visit_leases(super::context::LeaseSource::Facts(leases), Scope::Borrowed(scope), Some(at), groups)
    })
}

pub(in crate::traceability) fn source_group(
    scope: RecordScope, created_at: Timestamp, entries: Vec<TraceabilityEntry>,
) -> Result<TraceabilitySnapshot> {
    let source = entries
        .first()
        .and_then(|entry| entry.source.as_deref())
        .filter(|source| !source.is_empty())
        .ok_or_else(|| Error::Operation("traceability source group has no source".into()))?;
    if entries.iter().any(|entry| entry.source.as_deref() != Some(source)) {
        return Err(Error::Operation("traceability source group contains multiple sources".into()));
    }
    let seed = super::text(format_args!("{}|{source}", super::ScopeSeed(Scope::Borrowed(&scope))))?;
    Ok(TraceabilitySnapshot {
        traceability_id: dokime_domain::model::identity::TraceabilityId::from_seed(seed),
        scope,
        created_at,
        metadata: None,
        entries,
    })
}

fn collect_one(visit: &mut dyn FnMut(&mut dyn Groups) -> Result<()>) -> Result<Option<TraceabilitySnapshot>> {
    let mut group = OneGroup { seen: false, value: None };
    visit(&mut group)?;
    Ok(group.value)
}
