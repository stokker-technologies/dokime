/* ***
 * Dokime
 * SPDX-License-Identifier: Apache-2.0
 * ***
 */

/* === Modules === */

#[path = "payload/context.rs"]
mod context;
#[path = "payload/portable.rs"]
mod portable;
pub(super) use context::host_override_bytes;
pub(super) use portable::{
    execution_snapshots, host_snapshot, lease_snapshot, parameter_entry, resource_entries, section_snapshot, source_group,
};

/* === Imports === */

use context::{LeaseSource, visit_host, visit_leases};

use crate::{
    runtime::{Error, Result},
    state::RuntimeState,
};
use dokime_domain::model::{
    execution::{
        RecordScope,
        result::{TestCycle, TestExecution},
    },
    identity::{Identifier, Metadata, TestCycleId, TraceabilityId},
    time::Timestamp,
    traceability::{TraceabilityEntry, TraceabilitySnapshot},
    values::Value,
};
use std::fmt::{self, Write};

/* === Types === */

#[derive(Clone, Copy)]
enum Scope<'a> {
    Borrowed(&'a RecordScope),
    Cycle(&'a TestCycleId),
    Empty,
}
struct ScopeSeed<'a>(Scope<'a>);
struct Sanitized<'a>(&'a str);
struct ParameterLabel<'a>(&'a str);

enum EntryValue<'a> {
    Borrowed(&'a Value),
    Text(fmt::Arguments<'a>),
    Integer(i64),
}
enum EntryMetadata<'a> {
    Borrowed(&'a Metadata),
    New { identifier: fmt::Arguments<'a>, label: fmt::Arguments<'a> },
}
struct Entry<'a> {
    metadata: EntryMetadata<'a>,
    value: EntryValue<'a>,
    source: fmt::Arguments<'a>,
    unit: Option<&'a str>,
}

trait Entries {
    fn push(&mut self, entry: Entry<'_>) -> Result<()>;
}
trait Groups {
    fn group(
        &mut self, scope: Scope<'_>, at: Option<&Timestamp>, metadata: Option<EntryMetadata<'_>>, seed: fmt::Arguments<'_>,
        keep_empty: bool, visit: &mut dyn FnMut(&mut dyn Entries) -> Result<()>,
    ) -> Result<()>;
}
#[derive(Default)]
struct EntryPlan {
    count: usize,
}
#[derive(Default)]
struct Plan {
    count: usize,
}
struct EntryBuild {
    rows: Vec<TraceabilityEntry>,
}
struct Build {
    rows: Vec<TraceabilitySnapshot>,
}
#[derive(Default)]
struct TextSize {
    bytes: usize,
    visible: bool,
}

/* === Implementations === */

impl Write for TextSize {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.bytes = self.bytes.checked_add(text.len()).ok_or(fmt::Error)?;
        self.visible |= text.chars().any(|character| !character.is_whitespace());
        Ok(())
    }
}
impl<'a> Scope<'a> {
    fn values(self) -> [Option<&'a str>; 8] {
        match self {
            Self::Borrowed(scope) => super::projection::scope_values(scope),
            Self::Cycle(id) => [None, Some(id.as_str()), None, None, None, None, None, None],
            Self::Empty => [None; 8],
        }
    }
    fn origin(self) -> &'a dokime_domain::model::execution::EvidenceOrigin {
        match self {
            Self::Borrowed(scope) => &scope.origin,
            _ => &dokime_domain::model::execution::EvidenceOrigin::Default,
        }
    }
    fn visible(self, selected: Option<&RecordScope>) -> bool {
        let Some(selected) = selected else { return true };
        let fields = self.values().into_iter().zip(super::projection::scope_values(selected));
        let matches = fields.into_iter().all(|(value, selected)| selected.is_none_or(|selected| value == Some(selected)));
        let origin = match self {
            Self::Borrowed(scope) => scope.origin == selected.origin,
            _ => selected.origin == dokime_domain::model::execution::EvidenceOrigin::Default,
        };
        matches && (selected.origin == dokime_domain::model::execution::EvidenceOrigin::Default || origin)
    }
    fn owned(self) -> RecordScope {
        match self {
            Self::Borrowed(scope) => scope.clone(),
            Self::Cycle(id) => RecordScope::default().with_cycle(id.clone()),
            Self::Empty => RecordScope::default(),
        }
    }
}
impl fmt::Display for ScopeSeed<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, value) in self.0.values().into_iter().enumerate() {
            if index != 0 {
                output.write_char('|')?;
            }
            if let Some(value) = value {
                output.write_str(value)?;
            }
        }
        Ok(())
    }
}
impl fmt::Display for Sanitized<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        for character in self.0.chars() {
            output.write_char(if character.is_ascii_alphanumeric() || character == '.' || character == '_' { character } else { '_' })?;
        }
        Ok(())
    }
}
impl fmt::Display for ParameterLabel<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, part) in self.0.split(['.', '_', '-']).filter(|part| !part.is_empty()).enumerate() {
            if index != 0 {
                output.write_char(' ')?;
            }
            let mut characters = part.chars();
            if let Some(first) = characters.next() {
                for character in first.to_uppercase() {
                    output.write_char(character)?;
                }
            }
            output.write_str(characters.as_str())?;
        }
        Ok(())
    }
}
impl EntryValue<'_> {
    fn visible(&self) -> Result<bool> {
        match self {
            Self::Borrowed(value) => Ok(super::projection::resource_traceability_value_visible(value)),
            Self::Text(value) => Ok(text_size(*value)?.visible),
            _ => Ok(true),
        }
    }
    fn owned(self) -> Result<Value> {
        Ok(match self {
            Self::Borrowed(value) => value.clone(),
            Self::Text(value) => Value::Text(text(value)?),
            Self::Integer(value) => Value::Integer(value),
        })
    }
}
impl EntryMetadata<'_> {
    fn owned(self) -> Result<Metadata> {
        match self {
            Self::Borrowed(metadata) => Ok(metadata.clone()),
            Self::New { identifier, label } => {
                let identifier = Identifier::new(text(identifier)?).map_err(|error| Error::Operation(error.to_string()))?;
                Ok(Metadata::new(identifier, text(label)?, None))
            }
        }
    }
}
impl Entries for EntryPlan {
    fn push(&mut self, _entry: Entry<'_>) -> Result<()> {
        self.count = add(self.count, 1)?;
        Ok(())
    }
}
impl Entries for EntryBuild {
    fn push(&mut self, entry: Entry<'_>) -> Result<()> {
        self.rows.push(owned_entry(entry)?);
        Ok(())
    }
}
impl Groups for Plan {
    fn group(
        &mut self, _scope: Scope<'_>, _at: Option<&Timestamp>, _metadata: Option<EntryMetadata<'_>>, _seed: fmt::Arguments<'_>,
        keep_empty: bool, visit: &mut dyn FnMut(&mut dyn Entries) -> Result<()>,
    ) -> Result<()> {
        let mut entries = EntryPlan::default();
        visit(&mut entries)?;
        if entries.count == 0 && !keep_empty {
            return Ok(());
        }
        self.count = add(self.count, 1)?;
        Ok(())
    }
}
impl Groups for Build {
    fn group(
        &mut self, scope: Scope<'_>, at: Option<&Timestamp>, metadata: Option<EntryMetadata<'_>>, seed: fmt::Arguments<'_>,
        keep_empty: bool, visit: &mut dyn FnMut(&mut dyn Entries) -> Result<()>,
    ) -> Result<()> {
        if let Some(snapshot) = build_snapshot(scope, at, metadata, seed, keep_empty, visit)? {
            self.rows.push(snapshot);
        }
        Ok(())
    }
}
/* === Functions === */

fn exhausted() -> Error {
    harmos::Error::ResourceExhausted { resource: "traceability construction" }.into()
}
fn add(left: usize, right: usize) -> Result<usize> {
    left.checked_add(right).ok_or_else(exhausted)
}
fn reserved<T>(count: usize) -> Result<Vec<T>> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(count).map_err(|_| exhausted())?;
    Ok(rows)
}
fn text_size(value: fmt::Arguments<'_>) -> Result<TextSize> {
    let mut size = TextSize::default();
    size.write_fmt(value).map_err(|_| exhausted())?;
    Ok(size)
}
fn text(value: fmt::Arguments<'_>) -> Result<String> {
    let length = text_size(value)?.bytes;
    let mut result = String::new();
    result.try_reserve_exact(length).map_err(|_| exhausted())?;
    result.write_fmt(value).map_err(|_| exhausted())?;
    Ok(result)
}
fn entry(
    sink: &mut dyn Entries, identifier: fmt::Arguments<'_>, label: fmt::Arguments<'_>, value: EntryValue<'_>, source: fmt::Arguments<'_>,
) -> Result<()> {
    sink.push(Entry { metadata: EntryMetadata::New { identifier, label }, value, source, unit: None })
}

pub(crate) fn retained_state(
    state: &RuntimeState, selected: Option<&RecordScope>, at: &Timestamp,
) -> Result<harmos::storage::ReadBatch<Vec<TraceabilitySnapshot>>> {
    collect_batch(&mut |groups| visit_state(state, selected, Some(at), groups))
}

pub(crate) fn retained_host(
    host: &crate::telemetry::TelemetrySample, at: &Timestamp,
) -> Result<Option<harmos::storage::ReadBatch<Vec<TraceabilitySnapshot>>>> {
    let visible = super::projection::HOST_TRACEABILITY_FIELDS
        .iter()
        .any(|(key, _, _, _)| host.values.get(*key).is_some_and(super::projection::resource_traceability_value_visible));
    if !visible {
        return Ok(None);
    }
    let scope = host.scope.as_ref().map_or(Scope::Empty, Scope::Borrowed);
    collect_batch(&mut |groups| visit_host(&host.values, scope, Some(at), groups)).map(Some)
}

pub(crate) fn retained_leases(
    scope: Option<&RecordScope>, facts: crate::telemetry_runtime::leases::LeaseFacts<'_>, at: &Timestamp,
) -> Result<Option<harmos::storage::ReadBatch<Vec<TraceabilitySnapshot>>>> {
    if facts.iter().next().is_none() {
        return Ok(None);
    }
    let scope = scope.map_or(Scope::Empty, Scope::Borrowed);
    collect_batch(&mut |groups| visit_leases(LeaseSource::Ledger(facts), scope, Some(at), groups)).map(Some)
}

fn collect_batch(visit: &mut dyn FnMut(&mut dyn Groups) -> Result<()>) -> Result<harmos::storage::ReadBatch<Vec<TraceabilitySnapshot>>> {
    let mut plan = Plan::default();
    visit(&mut plan)?;
    harmos::storage::ReadBatch::new(()).try_map(|_| {
        let mut build = Build { rows: reserved(plan.count)? };
        visit(&mut build)?;
        Ok(build.rows)
    })
}

pub(crate) fn replace_source(
    left: harmos::storage::ReadBatch<Vec<TraceabilitySnapshot>>, right: harmos::storage::ReadBatch<Vec<TraceabilitySnapshot>>, source: &str,
) -> Result<harmos::storage::ReadBatch<Vec<TraceabilitySnapshot>>> {
    let keep_entry = |entry: &TraceabilityEntry| {
        entry
            .source
            .as_deref()
            .is_none_or(|candidate| candidate != source && !candidate.strip_prefix(source).is_some_and(|suffix| suffix.starts_with('.')))
    };
    let count = left.iter().filter(|snapshot| snapshot.entries.iter().any(keep_entry)).count();
    let count = add(count, right.len())?;
    left.try_map_with(right, |mut left, right| {
        for snapshot in &mut left {
            snapshot.entries.retain(keep_entry);
        }
        left.retain(|snapshot| !snapshot.entries.is_empty());
        if count <= left.capacity() {
            left.extend(right);
            return Ok(left);
        }
        let mut snapshots = reserved(count)?;
        snapshots.extend(left);
        snapshots.extend(right);
        Ok(snapshots)
    })
}

fn visit_state(state: &RuntimeState, selected: Option<&RecordScope>, at: Option<&Timestamp>, groups: &mut dyn Groups) -> Result<()> {
    visit_derived(state, selected, at, groups)?;
    visit_parameters(state, selected, at, groups)
}

#[derive(Clone, Copy)]
struct Cycle<'a> {
    state: &'a RuntimeState,
    cycle: &'a TestCycle,
    cases: &'a [(&'a dokime_domain::model::identity::CaseId, usize)],
}
impl<'a> Cycle<'a> {
    fn executions(self) -> impl DoubleEndedIterator<Item = &'a TestExecution> {
        self.cycle.execution_ids.iter().filter_map(|id| self.state.executions.get(id))
    }
    fn case_index(self, id: &dokime_domain::model::identity::CaseId) -> Option<usize> {
        self.cases.binary_search_by(|(case, _)| (*case).cmp(id)).ok().map(|index| self.cases[index].1)
    }
    fn case_count(self) -> usize {
        self.cases.len()
    }
    fn selected(self, scope: Scope<'a>) -> Option<&'a TestExecution> {
        let Scope::Borrowed(scope) = scope else { return None };
        let execution = match &scope.test_execution_id {
            Some(id) => self.executions().find(|execution| &execution.test_execution_id == id),
            None => self.executions().rev().find(|execution| scope.test_case_id.as_ref().is_none_or(|id| id == &execution.test_case_id)),
        };
        execution.filter(|execution| scope.test_case_id.as_ref().is_none_or(|id| id == &execution.test_case_id))
    }
}

fn visit_derived(state: &RuntimeState, selected: Option<&RecordScope>, at: Option<&Timestamp>, groups: &mut dyn Groups) -> Result<()> {
    for cycle in state.cycles.values() {
        let count = cycle.execution_ids.len();
        let mut cases = reserved(count)?;
        for (index, id) in cycle.execution_ids.iter().enumerate() {
            let execution = state.executions.get(id).ok_or_else(|| Error::Operation("cycle execution membership is invalid".into()))?;
            cases.push((&execution.snapshot.case().test_case_id, index));
        }
        cases.sort_unstable_by_key(|(left, index)| (*left, *index));
        cases.dedup_by(|(right, _), (left, _)| left == right);
        cases.sort_unstable_by_key(|(_, index)| *index);
        for (ordinal, (_, index)) in cases.iter_mut().enumerate() {
            *index = ordinal;
        }
        cases.sort_unstable_by_key(|(id, _)| *id);
        let cycle = Cycle { state, cycle, cases: &cases };
        if cycle.cycle.execution_ids.is_empty() {
            let scope = Scope::Cycle(&cycle.cycle.test_cycle_id);
            if scope.visible(selected) {
                visit_cycle(cycle, scope, at, groups)?;
            }
            continue;
        }
        for execution in cycle.executions() {
            let scope = Scope::Borrowed(&execution.scope);
            if scope.visible(selected) {
                visit_cycle(cycle, scope, at, groups)?;
            }
        }
    }
    Ok(())
}

fn visit_cycle<'a>(cycle: Cycle<'a>, scope: Scope<'a>, at: Option<&Timestamp>, groups: &mut dyn Groups) -> Result<()> {
    let execution = cycle.selected(scope);
    let fields = CycleFields {
        created_at: &cycle.cycle.created_at,
        updated_at: &cycle.cycle.updated_at,
        label: &cycle.cycle.metadata.label,
        case_count: cycle.case_count(),
    };
    let index = execution.and_then(|execution| cycle.case_index(&execution.test_case_id));
    visit_test_groups(fields, execution, index, scope, at, groups)?;
    visit_resources(cycle, scope, at, groups)
}

fn visit_test_entries(
    cycle: CycleFields<'_>, execution: Option<&TestExecution>, case_index: Option<usize>, source: &str, sink: &mut dyn Entries,
) -> Result<()> {
    use dokime_domain::model::traceability::{TEST_DEFINITION_TRACEABILITY, TEST_EXECUTION_TRACEABILITY};
    if source == TEST_EXECUTION_TRACEABILITY.source {
        entry(
            sink,
            format_args!("cycle.created_at"),
            format_args!("Cycle Created At"),
            EntryValue::Text(format_args!("{}", cycle.created_at)),
            format_args!("{source}"),
        )?;
        entry(
            sink,
            format_args!("cycle.updated_at"),
            format_args!("Cycle Updated At"),
            EntryValue::Text(format_args!("{}", cycle.updated_at)),
            format_args!("{source}"),
        )?;
        entry(
            sink,
            format_args!("cycle.label"),
            format_args!("Cycle Label"),
            EntryValue::Text(format_args!("{}", cycle.label)),
            format_args!("{source}"),
        )?;
        entry(
            sink,
            format_args!("cycle.case_count"),
            format_args!("Case Count"),
            EntryValue::Integer(cycle.case_count as i64),
            format_args!("{source}"),
        )?;
    }
    let Some(execution) = execution else { return Ok(()) };
    let snapshot = &execution.snapshot;
    let case = snapshot.case();
    if source == TEST_DEFINITION_TRACEABILITY.source {
        visible_entry(
            sink,
            format_args!("case.label"),
            format_args!("Case"),
            EntryValue::Text(format_args!("{}", case.metadata.label)),
            format_args!("{source}"),
        )?;
        if let Some(index) = case_index {
            visible_entry(
                sink,
                format_args!("case.ordinal"),
                format_args!("Case Position"),
                EntryValue::Integer(index as i64 + 1),
                format_args!("{source}"),
            )?;
        }
        visible_entry(
            sink,
            format_args!("execution.case_id"),
            format_args!("Case ID"),
            EntryValue::Text(format_args!("{}", execution.test_case_id)),
            format_args!("{source}"),
        )?;
        let method = snapshot.template().map(|template| template.metadata.identifier.as_str());
        match method {
            Some(method) => visible_entry(
                sink,
                format_args!("execution.template"),
                format_args!("Test Method"),
                EntryValue::Text(format_args!("{method}")),
                format_args!("{source}"),
            )?,
            None => visible_entry(
                sink,
                format_args!("execution.template"),
                format_args!("Test Method"),
                EntryValue::Text(format_args!("{}", case.test_case_template_id)),
                format_args!("{source}"),
            )?,
        }
        if let Some(suite) = snapshot.suite() {
            visible_entry(
                sink,
                format_args!("execution.suite"),
                format_args!("Suite"),
                EntryValue::Text(format_args!("{}", suite.identifier)),
                format_args!("{source}"),
            )?;
            visible_entry(
                sink,
                format_args!("execution.suite_label"),
                format_args!("Suite Label"),
                EntryValue::Text(format_args!("{}", suite.label)),
                format_args!("{source}"),
            )?;
        }
        if let Some(plugin) = snapshot.plugin() {
            visible_entry(
                sink,
                format_args!("execution.plugin"),
                format_args!("Plugin"),
                EntryValue::Text(format_args!("{}", plugin.source)),
                format_args!("{source}"),
            )?;
            if let Some(version) = &plugin.version {
                visible_entry(
                    sink,
                    format_args!("execution.plugin_version"),
                    format_args!("Plugin Version"),
                    EntryValue::Text(format_args!("{version}")),
                    format_args!("{source}"),
                )?;
            }
            use dokime_domain::model::execution::snapshot::ExecutionImplementation;
            let (identifier, label, value) = match &plugin.implementation {
                ExecutionImplementation::Bundle { content_digest } => ("execution.plugin_digest", "Plugin Digest", content_digest),
                ExecutionImplementation::Embedded { generation } => ("execution.provider_generation", "Provider Generation", generation),
            };
            visible_entry(
                sink,
                format_args!("{identifier}"),
                format_args!("{label}"),
                EntryValue::Text(format_args!("{value}")),
                format_args!("{source}"),
            )?;
        }
        if let Some(version) = snapshot.host_version() {
            visible_entry(
                sink,
                format_args!("execution.dokime_version"),
                format_args!("Dokime Version"),
                EntryValue::Text(format_args!("{version}")),
                format_args!("{source}"),
            )?;
        }
    }
    if source == TEST_EXECUTION_TRACEABILITY.source {
        for (identifier, label, at) in [
            ("execution.started_at", "Started At", execution.started_at.as_ref()),
            ("execution.finished_at", "Finished At", execution.finished_at.as_ref()),
        ] {
            if let Some(at) = at {
                visible_entry(
                    sink,
                    format_args!("{identifier}"),
                    format_args!("{label}"),
                    EntryValue::Text(format_args!("{at}")),
                    format_args!("{source}"),
                )?;
            }
        }
        visible_entry(
            sink,
            format_args!("execution.status"),
            format_args!("Execution Status"),
            EntryValue::Text(format_args!("{:?}", execution.status)),
            format_args!("{source}"),
        )?;
    }
    Ok(())
}

fn visible_entry(
    sink: &mut dyn Entries, identifier: fmt::Arguments<'_>, label: fmt::Arguments<'_>, value: EntryValue<'_>, source: fmt::Arguments<'_>,
) -> Result<()> {
    if !value.visible()? {
        return Ok(());
    }
    entry(sink, identifier, label, value, source)
}

fn resource_id(record: &dokime_domain::model::records::resources::ResourceRecord) -> &str {
    record.details.get("resource_instance_id").and_then(value_text).unwrap_or(record.resource_id.as_str())
}
fn value_text(value: &Value) -> Option<&str> {
    if let Value::Text(value) = value { Some(value) } else { None }
}
fn resource_records<'a>(
    cycle: Cycle<'a>, scope: Scope<'a>,
) -> impl Iterator<Item = &'a dokime_domain::model::records::resources::ResourceRecord> {
    let owner = match scope {
        Scope::Borrowed(scope) => scope.test_execution_id.as_ref(),
        _ => None,
    }
    .or_else(|| cycle.executions().next_back().map(|execution| &execution.test_execution_id));
    cycle.state.resources.records().filter(move |record| {
        let Some(owner) = owner else { return false };
        let Scope::Borrowed(selected) = scope else { return false };
        record.scope.as_ref().is_some_and(|scope| scope.is_visible_in(selected) && scope.test_execution_id.as_ref() == Some(owner))
    })
}
fn visit_resources<'a>(cycle: Cycle<'a>, scope: Scope<'a>, at: Option<&Timestamp>, groups: &mut dyn Groups) -> Result<()> {
    let count = resource_records(cycle, scope).count();
    let mut resources = reserved(count)?;
    resources.extend(resource_records(cycle, scope).enumerate());
    resources.sort_unstable_by(|(li, left), (ri, right)| {
        resource_id(left)
            .cmp(resource_id(right))
            .then(left.metadata.label.cmp(&right.metadata.label))
            .then(left.resource_id.cmp(&right.resource_id))
            .then(li.cmp(ri))
    });
    let mut start = 0;
    let mut index = 0;
    while start < resources.len() {
        let id = resource_id(resources[start].1);
        let length = resources[start..].iter().take_while(|(_, record)| resource_id(record) == id).count();
        let record = resources[start + length - 1].1;
        let details = &record.details;
        let label = details
            .get("resource_instance_label")
            .and_then(value_text)
            .or_else(|| details.get("resource_name").and_then(value_text))
            .unwrap_or(id);
        for (kind, source) in [(0, "resources"), (1, "devices"), (2, "python_interfaces")] {
            groups.group(
                scope,
                at,
                None,
                format_args!("{}|traceability.{source}.{}", ScopeSeed(scope), Sanitized(id)),
                false,
                &mut |entries| visit_resource_entries(id, label, details, index, kind, entries),
            )?;
        }
        start += length;
        index += 1;
    }
    Ok(())
}
fn visit_resource_entries(
    id: &str, label: &str, details: &std::collections::BTreeMap<String, Value>, index: usize, kind: usize, sink: &mut dyn Entries,
) -> Result<()> {
    let source = match kind {
        0 => "resources",
        1 => "devices",
        _ => "python_interfaces",
    };
    if kind != 0 {
        let (field, prefix) = if kind == 1 { ("device_info", "device_info") } else { ("python_interfaces", "python_interface") };
        let Some(Value::Object(values)) = details.get(field) else { return Ok(()) };
        for (key, value) in values {
            if !super::projection::resource_traceability_value_visible(value) {
                continue;
            }
            entry(
                sink,
                format_args!("resource.{}.{prefix}.{}", Sanitized(id), Sanitized(key)),
                format_args!("{}", ParameterLabel(key)),
                EntryValue::Borrowed(value),
                format_args!("traceability.{source}.{}", Sanitized(id)),
            )?;
        }
        return Ok(());
    }
    entry(
        sink,
        format_args!("resource.{}.name", Sanitized(id)),
        format_args!("Resource"),
        EntryValue::Text(format_args!("{label}")),
        format_args!("traceability.{source}.{}", Sanitized(id)),
    )?;
    if let Some(name) = details.get("resource_name").and_then(value_text).filter(|name| *name != label) {
        entry(
            sink,
            format_args!("resource.{}.resource_name", Sanitized(id)),
            format_args!("Resource Name"),
            EntryValue::Text(format_args!("{name}")),
            format_args!("traceability.{source}.{}", Sanitized(id)),
        )?;
    }
    for (key, label) in [
        ("resource_kind", "Resource Kind"),
        ("path", "Resource Path"),
        ("connection", "Connection"),
        ("endpoint", "Endpoint"),
        ("process_isolation", "Process Isolation"),
        ("traceability_error", "Traceability Error"),
    ] {
        let Some(value) = details.get(key) else { continue };
        let suffix = if key == "resource_kind" { "kind" } else { key };
        entry(
            sink,
            format_args!("resource.{}.{suffix}", Sanitized(id)),
            format_args!("{label}"),
            EntryValue::Borrowed(value),
            format_args!("traceability.{source}.{}", Sanitized(id)),
        )?;
    }
    if let Some(Value::Object(values)) = details.get("traceability") {
        for (key, value) in values {
            if !super::projection::resource_traceability_value_visible(value) {
                continue;
            }
            entry(
                sink,
                format_args!("resource.{}.traceability.{}", Sanitized(id), Sanitized(key)),
                format_args!("{}", ParameterLabel(key)),
                EntryValue::Borrowed(value),
                format_args!("traceability.{source}.{}", Sanitized(id)),
            )?;
        }
    }
    entry(
        sink,
        format_args!("resource.{}.order", Sanitized(id)),
        format_args!("Resource Order"),
        EntryValue::Integer(index as i64 + 1),
        format_args!("traceability.{source}.{}", Sanitized(id)),
    )?;
    Ok(())
}

#[derive(Clone, Copy)]
struct ParameterRow<'a> {
    scope: Scope<'a>,
    source: &'a str,
    parameter: &'a crate::definition::MaterializedParameter,
    ordinal: usize,
}
struct ParameterGroup<'a> {
    scope: Scope<'a>,
    source: &'a str,
    start: usize,
    length: usize,
    ordinal: usize,
}

fn parameter_rows<'a>(state: &'a RuntimeState, selected: Option<&'a RecordScope>) -> impl Iterator<Item = ParameterRow<'a>> {
    let run = state.run_parameters.values().map(|parameter| (Scope::Empty, parameter));
    let executions = state.executions.values().flat_map(|execution| {
        super::projection::execution_parameter_values(execution).map(move |parameter| (Scope::Borrowed(&execution.scope), parameter))
    });
    run.chain(executions)
        .filter_map(move |(scope, parameter)| {
            if !scope.visible(selected) || !super::projection::resource_traceability_value_visible(&parameter.value) {
                return None;
            }
            let source = super::projection::parameter_traceability_source(parameter)?;
            Some((scope, source, parameter))
        })
        .enumerate()
        .map(|(ordinal, (scope, source, parameter))| ParameterRow { scope, source, parameter, ordinal })
}
fn visit_parameters(state: &RuntimeState, selected: Option<&RecordScope>, at: Option<&Timestamp>, groups: &mut dyn Groups) -> Result<()> {
    let count = parameter_rows(state, selected).count();
    let mut rows = reserved(count)?;
    rows.extend(parameter_rows(state, selected));
    rows.sort_unstable_by(|left, right| {
        left.scope
            .values()
            .cmp(&right.scope.values())
            .then(left.scope.origin().cmp(right.scope.origin()))
            .then(left.source.cmp(right.source))
            .then(left.ordinal.cmp(&right.ordinal))
    });
    let mut descriptors = reserved(count)?;
    let mut start = 0;
    while start < rows.len() {
        let row = rows[start];
        let length = rows[start..]
            .iter()
            .take_while(|next| {
                next.scope.values() == row.scope.values() && next.scope.origin() == row.scope.origin() && next.source == row.source
            })
            .count();
        descriptors.push(ParameterGroup { scope: row.scope, source: row.source, start, length, ordinal: row.ordinal });
        start += length;
    }
    descriptors.sort_unstable_by_key(|group| group.ordinal);
    for group in descriptors {
        groups.group(group.scope, at, None, format_args!("{}|{}", ScopeSeed(group.scope), group.source), false, &mut |entries| {
            for row in &rows[group.start..group.start + group.length] {
                emit_parameter(row.parameter, group.source, entries)?;
            }
            Ok(())
        })?;
    }
    for execution in state.executions.values() {
        let scope = Scope::Borrowed(&execution.scope);
        if !scope.visible(selected) {
            continue;
        }
        for section in &execution.traceability {
            visit_section(scope, section, at, groups)?;
        }
    }
    Ok(())
}
struct StatedSource<'a>(Option<&'a str>);
impl fmt::Display for StatedSource<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str("traceability.template")?;
        if let Some(source) = self.0.filter(|source| !source.is_empty()) {
            output.write_char('.')?;
            write!(output, "{}", Sanitized(source))?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct CountGroups {
    count: usize,
}
#[derive(Default)]
struct CountEntries {
    count: usize,
}
impl Entries for CountEntries {
    fn push(&mut self, _entry: Entry<'_>) -> Result<()> {
        self.count = add(self.count, 1)?;
        Ok(())
    }
}
impl Groups for CountGroups {
    fn group(
        &mut self, _scope: Scope<'_>, _at: Option<&Timestamp>, _metadata: Option<EntryMetadata<'_>>, _seed: fmt::Arguments<'_>,
        keep_empty: bool, visit: &mut dyn FnMut(&mut dyn Entries) -> Result<()>,
    ) -> Result<()> {
        let mut entries = CountEntries::default();
        visit(&mut entries)?;
        if keep_empty || entries.count != 0 {
            self.count = add(self.count, 1)?;
        }
        Ok(())
    }
}
pub(crate) fn count(state: &RuntimeState) -> Result<usize> {
    let mut count = CountGroups::default();
    visit_state(state, None, None, &mut count)?;
    Ok(count.count)
}

fn owned_entry(entry: Entry<'_>) -> Result<TraceabilityEntry> {
    Ok(TraceabilityEntry {
        metadata: entry.metadata.owned()?,
        value: entry.value.owned()?,
        source: Some(text(entry.source)?),
        unit: entry.unit.map(|unit| text(format_args!("{unit}"))).transpose()?,
    })
}
fn build_snapshot(
    scope: Scope<'_>, at: Option<&Timestamp>, metadata: Option<EntryMetadata<'_>>, seed: fmt::Arguments<'_>, keep_empty: bool,
    visit: &mut dyn FnMut(&mut dyn Entries) -> Result<()>,
) -> Result<Option<TraceabilitySnapshot>> {
    let mut plan = EntryPlan::default();
    visit(&mut plan)?;
    if plan.count == 0 && !keep_empty {
        return Ok(None);
    }
    let mut entries = EntryBuild { rows: reserved(plan.count)? };
    visit(&mut entries)?;
    let at = at.ok_or_else(exhausted)?;
    Ok(Some(TraceabilitySnapshot {
        traceability_id: TraceabilityId::from_seed(text(seed)?),
        scope: scope.owned(),
        created_at: at.clone(),
        metadata: metadata.map(EntryMetadata::owned).transpose()?,
        entries: entries.rows,
    }))
}
fn collect_groups(visit: &mut dyn FnMut(&mut dyn Groups) -> Result<()>) -> Result<Vec<TraceabilitySnapshot>> {
    let mut plan = Plan::default();
    visit(&mut plan)?;
    let mut build = Build { rows: reserved(plan.count)? };
    visit(&mut build)?;
    Ok(build.rows)
}
fn collect_entries(visit: &mut dyn FnMut(&mut dyn Entries) -> Result<()>) -> Result<Vec<TraceabilityEntry>> {
    let mut plan = EntryPlan::default();
    visit(&mut plan)?;
    let mut build = EntryBuild { rows: reserved(plan.count)? };
    visit(&mut build)?;
    Ok(build.rows)
}
fn emit_parameter(parameter: &crate::definition::MaterializedParameter, source: &str, sink: &mut dyn Entries) -> Result<()> {
    sink.push(Entry {
        metadata: EntryMetadata::Borrowed(&parameter.metadata),
        value: EntryValue::Borrowed(&parameter.value),
        source: format_args!("{source}"),
        unit: parameter.unit.as_ref().map(|unit| unit.symbol.as_str()).filter(|unit| !unit.is_empty()),
    })
}
fn visit_section(
    scope: Scope<'_>, section: &dokime_domain::model::traceability::TraceabilitySection, at: Option<&Timestamp>, groups: &mut dyn Groups,
) -> Result<()> {
    groups.group(
        scope,
        at,
        Some(EntryMetadata::New { identifier: format_args!("{}", section.identifier), label: format_args!("{}", section.label) }),
        format_args!("{}|traceability.template.{}", ScopeSeed(scope), section.identifier),
        true,
        &mut |entries| {
            for row in &section.rows {
                if !super::projection::resource_traceability_value_visible(&row.value) {
                    continue;
                }
                entries.push(Entry {
                    metadata: EntryMetadata::New { identifier: format_args!("{}", row.identifier), label: format_args!("{}", row.label) },
                    value: EntryValue::Borrowed(&row.value),
                    source: format_args!("{}", StatedSource(row.source.as_deref())),
                    unit: row.unit.as_deref(),
                })?;
            }
            Ok(())
        },
    )
}
#[derive(Clone, Copy)]
struct CycleFields<'a> {
    created_at: &'a Timestamp,
    updated_at: &'a Timestamp,
    label: &'a str,
    case_count: usize,
}
fn visit_test_groups(
    fields: CycleFields<'_>, execution: Option<&TestExecution>, case_index: Option<usize>, scope: Scope<'_>, at: Option<&Timestamp>,
    groups: &mut dyn Groups,
) -> Result<()> {
    use dokime_domain::model::traceability::{TEST_DEFINITION_TRACEABILITY, TEST_EXECUTION_TRACEABILITY};
    let mut sources = [TEST_DEFINITION_TRACEABILITY.source, TEST_EXECUTION_TRACEABILITY.source];
    sources.sort_unstable();
    for source in sources {
        groups.group(scope, at, None, format_args!("{}|{source}", ScopeSeed(scope)), false, &mut |entries| {
            visit_test_entries(fields, execution, case_index, source, entries)
        })?;
    }
    Ok(())
}
