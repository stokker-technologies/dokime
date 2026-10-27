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

// Local Imports
use crate::definition::{MaterializedParameter, ParameterSource};
use crate::runtime::Result as ProjectionResult;
use crate::state::RuntimeState;
use crate::telemetry::TelemetrySample;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::execution::scope_visible_in;
use dokime_domain::model::identity::TraceabilityId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::traceability::TraceabilityEntry;
use dokime_domain::model::traceability::TraceabilitySection;
use dokime_domain::model::traceability::TraceabilitySnapshot;
use dokime_domain::model::values::Value;

/* === Constants === */

pub(super) const HOST_TRACEABILITY_FIELDS: &[(&str, &str, &str, &str)] = &[
    ("host_name", "host.hostname", "Hostname", "traceability.host.info"),
    ("user", "host.user", "Username", "traceability.host.user"),
    ("git_user_name", "host.git_user_name", "Git User Name", "traceability.host.user"),
    ("git_user_email", "host.git_user_email", "Git User Email", "traceability.host.user"),
    ("os_name", "host.os_name", "OS", "traceability.host.info"),
    ("os_version", "host.os_version", "OS Version", "traceability.host.info"),
    ("kernel_version", "host.kernel_version", "Kernel Version", "traceability.host.info"),
    ("cpu_architecture", "host.cpu_architecture", "CPU Architecture", "traceability.host.cpu"),
    ("cpu_name", "host.cpu_name", "CPU Name", "traceability.host.cpu"),
    ("cpu_brand", "host.cpu_brand", "CPU Brand", "traceability.host.cpu"),
    ("cpu_vendor_id", "host.cpu_vendor", "CPU Vendor", "traceability.host.cpu"),
    ("cpu_frequency_mhz", "host.cpu_frequency_mhz", "CPU Frequency", "traceability.host.cpu"),
    ("physical_cpu_count", "host.physical_cpu_count", "Physical CPU Count", "traceability.host.cpu"),
    ("logical_cpu_count", "host.logical_cpu_count", "Logical CPU Count", "traceability.host.cpu"),
    ("memory_total_bytes", "host.memory_total_bytes", "Memory Total", "traceability.host.ram"),
    ("memory_used_bytes", "host.memory_used_bytes", "Memory Used", "traceability.host.ram"),
    ("memory_pressure_percent", "host.memory_pressure", "Memory Pressure", "traceability.host.ram"),
    ("system_uptime_s", "host.system_uptime", "System Uptime", "traceability.host.info"),
    ("power_plan_name", "host.power_plan", "Power Plan", "traceability.host.power"),
    ("power_mode", "host.power_mode", "Power Mode", "traceability.host.power"),
    ("power_mode_performance_bias", "host.performance_bias", "Performance Bias", "traceability.host.power"),
    ("battery_percent", "host.battery_percent", "Battery Percent", "traceability.host.power"),
    ("battery_state", "host.battery_state", "Battery State", "traceability.host.power"),
    ("battery_power_plugged", "host.power_plugged", "Plugged In", "traceability.host.power"),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TraceabilityPageCandidate {
    pub traceability_id: String,
    pub entry_count: usize,
    pub estimated_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceabilityPageBudget {
    pub max_entries: usize,
    pub max_bytes: usize,
}

struct Candidate {
    parameter: Option<(RecordScope, String)>,
    additional: bool,
    stated: Option<usize>,
}

struct ParameterTraceabilityGroup {
    scope: RecordScope,
    source: String,
    entries: Vec<TraceabilityEntry>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn resource_traceability(
    state: &RuntimeState, cycle: &CycleView, scope: RecordScope, created_at: Timestamp,
) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    resource_traceability_groups(state, cycle, &scope)?
        .into_iter()
        .map(|entries| traceability_group(scope.clone(), created_at.clone(), entries))
        .collect()
}
pub fn execution_traceability(cycle: &CycleView, execution: &TestExecution, at: Timestamp) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    test_snapshots(cycle, execution.scope.clone(), Some(execution), at)
}
pub fn snapshots_from_state(
    state: &RuntimeState, scope: Option<&RecordScope>, at: Timestamp,
) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    let mut snapshots = derived_snapshots(state, scope, &at)?;
    snapshots.extend(snapshots_from_parameters(&state.run_parameters, state.executions.values(), scope, at)?);
    Ok(snapshots)
}
pub fn snapshots_from_parameters<'a>(
    run_parameters: &'a BTreeMap<String, MaterializedParameter>, executions: impl IntoIterator<Item = &'a TestExecution>,
    scope: Option<&RecordScope>, created_at: Timestamp,
) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    let mut groups = Vec::<ParameterTraceabilityGroup>::new();
    let executions = executions.into_iter().collect::<Vec<_>>();
    collect_parameter_traceability(&mut groups, RecordScope::default(), run_parameters.values(), scope)?;
    for execution in &executions {
        collect_parameter_traceability(&mut groups, execution.scope.clone(), execution_parameter_values(execution), scope)?;
    }
    let mut snapshots = finish_parameter_groups(groups, created_at.clone())?;
    snapshots.extend(stated_snapshots(executions, scope, &created_at)?);
    Ok(snapshots)
}

pub(crate) fn bounded_snapshots_from_state(
    state: &RuntimeState, selected_scope: Option<&RecordScope>, after: &str, limit: usize, budget: TraceabilityPageBudget,
    additional: Option<&TraceabilityPageCandidate>, created_at: Timestamp,
) -> Option<(Vec<TraceabilitySnapshot>, bool, bool)> {
    let candidate_limit = limit.saturating_add(1);
    if candidate_limit == 0 {
        return Some((Vec::new(), false, false));
    }

    // Stated sections are cheap to project and few per attempt, so they are
    // built once and then compete for the page on the same terms as a
    // parameter group: by identity first, then against the same budget.
    let mut stated = stated_snapshots(state.executions.values(), selected_scope, &created_at).ok()?;
    stated.extend(derived_snapshots(state, selected_scope, &created_at).ok()?);
    let mut candidates = BTreeMap::<String, Candidate>::new();
    visit_parameter_scopes(state, |scope, parameters| {
        if selected_scope.is_some_and(|selected| !scope.is_visible_in(selected)) {
            return;
        }
        for parameter in parameters
            .filter(|parameter| parameter_traceability_source(parameter).is_some() && resource_traceability_value_visible(&parameter.value))
        {
            let source = parameter_traceability_source(parameter).expect("traceability source was checked before candidate creation");
            let id = traceability_group_id(&scope, source);
            if id.as_str() <= after {
                continue;
            }
            candidates.entry(id).or_insert_with(|| Candidate {
                parameter: Some((scope.clone(), source.to_owned())),
                additional: false,
                stated: None,
            });
            trim_candidates(&mut candidates, candidate_limit);
        }
    });

    for (index, snapshot) in stated.iter().enumerate() {
        let id = snapshot.traceability_id.to_string();
        if id.as_str() <= after {
            continue;
        }
        candidates.entry(id).or_insert(Candidate { parameter: None, additional: false, stated: Some(index) });
        trim_candidates(&mut candidates, candidate_limit);
    }

    if let Some(additional) = additional.filter(|candidate| candidate.traceability_id.as_str() > after) {
        candidates.entry(additional.traceability_id.clone()).and_modify(|candidate| candidate.additional = true).or_insert(Candidate {
            parameter: None,
            additional: true,
            stated: None,
        });
        trim_candidates(&mut candidates, candidate_limit);
    }

    if candidates.is_empty() {
        return Some((Vec::new(), false, false));
    }

    let has_more = candidates.len() > limit;
    let selected_parameters = candidates
        .iter()
        .take(limit)
        .filter_map(|(id, candidate)| candidate.parameter.as_ref().map(|(scope, source)| (id.clone(), (scope.clone(), source.clone()))))
        .collect::<BTreeMap<_, _>>();
    let selected_stated = candidates.iter().take(limit).filter_map(|(_, candidate)| candidate.stated).collect::<Vec<_>>();
    let additional_selected = additional.is_some_and(|additional| {
        additional.traceability_id.as_str() > after
            && candidates.iter().take(limit).any(|(id, candidate)| id == &additional.traceability_id && candidate.additional)
    });

    let mut entry_count = 0_usize;
    let mut estimated_bytes = 0_usize;
    visit_parameter_scopes(state, |scope, parameters| {
        if selected_scope.is_some_and(|selected| !scope.is_visible_in(selected)) {
            return;
        }
        for parameter in parameters
            .filter(|parameter| parameter_traceability_source(parameter).is_some() && resource_traceability_value_visible(&parameter.value))
        {
            let source = parameter_traceability_source(parameter).expect("traceability source was checked before budget calculation");
            let id = traceability_group_id(&scope, source);
            if selected_parameters.contains_key(&id) {
                entry_count = entry_count.saturating_add(1);
                estimated_bytes = estimated_bytes.saturating_add(traceability_entry_bytes(parameter, source));
            }
        }
    });
    for index in &selected_stated {
        let snapshot = &stated[*index];
        entry_count = entry_count.saturating_add(snapshot.entries.len());
        for entry in &snapshot.entries {
            estimated_bytes = estimated_bytes.saturating_add(stated_entry_bytes(entry));
        }
    }
    if additional_selected && let Some(additional) = additional {
        entry_count = entry_count.saturating_add(additional.entry_count);
        estimated_bytes = estimated_bytes.saturating_add(additional.estimated_bytes);
    }
    if entry_count > budget.max_entries || estimated_bytes > budget.max_bytes {
        return None;
    }

    let mut snapshots = selected_parameters
        .into_iter()
        .map(|(id, (scope, _source))| {
            (
                id.clone(),
                TraceabilitySnapshot {
                    traceability_id: TraceabilityId::from_uuid(id).expect("candidate traceability id is a valid UUID"),
                    scope,
                    created_at: created_at.clone(),
                    metadata: None,
                    entries: Vec::new(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut construction_failed = false;
    visit_parameter_scopes(state, |scope, parameters| {
        if selected_scope.is_some_and(|selected| !scope.is_visible_in(selected)) {
            return;
        }
        for parameter in parameters
            .filter(|parameter| parameter_traceability_source(parameter).is_some() && resource_traceability_value_visible(&parameter.value))
        {
            let source = parameter_traceability_source(parameter).expect("traceability source was checked before entry creation");
            let id = traceability_group_id(&scope, source);
            if let Some(snapshot) = snapshots.get_mut(&id) {
                match super::payload::parameter_entry(parameter, source) {
                    Ok(entry) => snapshot.entries.push(entry),
                    Err(_) => {
                        construction_failed = true;
                        return;
                    }
                }
            }
        }
    });
    if construction_failed {
        return None;
    }

    // Into the same map, not appended after it: the page's cursor is the last
    // id it returned, so a stated section that sorted below a parameter group
    // would make the next page re-deliver rows the caller already has.
    for index in selected_stated {
        let snapshot = stated[index].clone();
        snapshots.insert(snapshot.traceability_id.to_string(), snapshot);
    }

    Some((snapshots.into_values().collect(), has_more, additional_selected))
}

pub fn snapshots_from_parameter_scopes<'a>(
    parameters: impl IntoIterator<Item = (RecordScope, &'a BTreeMap<String, MaterializedParameter>)>, scope: Option<&RecordScope>,
    created_at: Timestamp,
) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    let mut groups = Vec::new();
    for (parameter_scope, values) in parameters {
        collect_parameter_traceability(&mut groups, parameter_scope, values.values(), scope)?;
    }
    finish_parameter_groups(groups, created_at)
}
pub(crate) fn snapshots_from_execution_parameters(
    run_parameters: &BTreeMap<String, MaterializedParameter>, execution: &TestExecution, scope: Option<&RecordScope>, created_at: Timestamp,
) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    let mut groups = Vec::new();
    collect_parameter_traceability(&mut groups, RecordScope::default(), run_parameters.values(), scope)?;
    collect_parameter_traceability(&mut groups, execution.scope.clone(), execution_parameter_values(execution), scope)?;
    finish_parameter_groups(groups, created_at)
}
pub fn stated_snapshots<'a>(
    executions: impl IntoIterator<Item = &'a TestExecution>, scope: Option<&RecordScope>, created_at: &Timestamp,
) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    executions
        .into_iter()
        .filter(|execution| scope.is_none_or(|selected| execution.scope.is_visible_in(selected)))
        .flat_map(|execution| execution.traceability.iter().map(move |section| stated_snapshot(&execution.scope, section, created_at)))
        .collect()
}

pub(crate) fn stated_execution_snapshots(
    execution: &TestExecution, created_at: &Timestamp,
) -> crate::runtime::Result<Vec<TraceabilitySnapshot>> {
    let count = execution.traceability.len();
    let mut snapshots = Vec::new();
    snapshots.try_reserve_exact(count).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
    for section in &execution.traceability {
        snapshots.push(stated_snapshot(&execution.scope, section, created_at)?);
    }
    Ok(snapshots)
}

pub fn host_traceability_snapshot_from_samples<'a, I>(
    samples: I, selected_scope: Option<&RecordScope>, created_at: Timestamp,
) -> ProjectionResult<Option<TraceabilitySnapshot>>
where
    I: IntoIterator<Item = &'a TelemetrySample>,
    I::IntoIter: DoubleEndedIterator,
{
    let sample = samples.into_iter().rev().find(|sample| {
        sample.stream.as_str() == "host.system" && selected_scope.is_none_or(|scope| scope_visible_in(sample.scope.as_ref(), scope))
    });
    let Some(sample) = sample else { return Ok(None) };
    super::payload::host_snapshot(&sample.values, &sample.scope.clone().unwrap_or_default(), &created_at)
}

pub fn bounded_host_traceability_snapshot_from_sample(
    sample: &TelemetrySample, selected_scope: Option<&RecordScope>, max_entries: usize, max_bytes: usize, created_at: Timestamp,
) -> Result<Option<TraceabilitySnapshot>, String> {
    let Some(candidate) = host_traceability_page_candidate(sample, selected_scope) else {
        return Ok(None);
    };
    if candidate.entry_count > max_entries || candidate.estimated_bytes > max_bytes {
        return Err("host traceability sample exceeds the bounded page budget".to_owned());
    }

    super::payload::host_snapshot(&sample.values, &sample.scope.clone().unwrap_or_default(), &created_at).map_err(|error| error.to_string())
}

pub(crate) fn host_traceability_page_candidate(
    sample: &TelemetrySample, selected_scope: Option<&RecordScope>,
) -> Option<TraceabilityPageCandidate> {
    if sample.stream.as_str() != "host.system" || selected_scope.is_some_and(|scope| !scope_visible_in(sample.scope.as_ref(), scope)) {
        return None;
    }

    let mut entry_count = 0_usize;
    let mut estimated_bytes = 0_usize;
    let mut sources = BTreeSet::new();
    for (key, identifier_value, label, source) in HOST_TRACEABILITY_FIELDS {
        let Some(value) = sample.values.get(*key) else {
            continue;
        };
        if !resource_traceability_value_visible(value) {
            continue;
        }
        entry_count = entry_count.saturating_add(1);
        estimated_bytes = estimated_bytes.saturating_add(
            identifier_value
                .len()
                .saturating_add(label.len())
                .saturating_add(source.len())
                .saturating_add(host_traceability_value_bytes(key, value))
                .saturating_add(32),
        );
        sources.insert(*source);
    }
    if entry_count == 0 {
        return None;
    }

    let scope = sample.scope.clone().unwrap_or_default();
    let discriminator = sources.into_iter().collect::<Vec<_>>().join("|");
    Some(TraceabilityPageCandidate {
        traceability_id: TraceabilityId::from_seed(format!("{}|{discriminator}", scope_discriminator(&scope))).to_string(),
        entry_count,
        estimated_bytes,
    })
}

pub fn replace_snapshots_by_source(snapshots: &mut Vec<TraceabilitySnapshot>, source: &str, replacement: TraceabilitySnapshot) {
    for snapshot in snapshots.iter_mut() {
        snapshot.entries.retain(|entry| {
            entry.source.as_deref().is_none_or(|entry_source| {
                entry_source != source && !entry_source.strip_prefix(source).is_some_and(|suffix| suffix.starts_with('.'))
            })
        });
    }
    snapshots.retain(|snapshot| !snapshot.entries.is_empty());
    snapshots.push(replacement);
}

pub fn lease_traceability_snapshot(
    leases: &[crate::telemetry_runtime::LeaseFact], scope: &RecordScope, created_at: Timestamp,
) -> ProjectionResult<Option<TraceabilitySnapshot>> {
    super::payload::lease_snapshot(leases, scope, &created_at)
}

pub(crate) fn scope_discriminator(scope: &RecordScope) -> String {
    let values = scope_values(scope);
    let length = values.iter().map(|value| value.map_or(0, str::len)).sum::<usize>() + values.len() - 1;
    let mut discriminator = String::with_capacity(length);
    for (index, value) in values.into_iter().enumerate() {
        if index != 0 {
            discriminator.push('|');
        }
        if let Some(value) = value {
            discriminator.push_str(value);
        }
    }
    discriminator
}

/* === Private Functions === */

pub(super) fn scope_values(scope: &RecordScope) -> [Option<&str>; 8] {
    [
        scope.session_id.as_ref().map(|id| id.as_str()),
        scope.test_cycle_id.as_ref().map(|id| id.as_str()),
        scope.test_case_id.as_ref().map(|id| id.as_str()),
        scope.test_execution_id.as_ref().map(|id| id.as_str()),
        scope.step_id.as_ref().map(|id| id.as_str()),
        scope.criterion_id.as_ref().map(|id| id.as_str()),
        scope.template_id.as_ref().map(|id| id.as_str()),
        scope.suite_id.as_ref().map(|id| id.as_str()),
    ]
}

fn traceability_seed_bytes<'a>(scope: &RecordScope, parts: impl Iterator<Item = &'a str> + Clone, separator: char) -> usize {
    let values = scope_values(scope);
    let scope_bytes = values.iter().fold(values.len(), |bytes, value| bytes.saturating_add(value.map_or(0, str::len)));
    let separators = parts.clone().count().saturating_sub(1).saturating_mul(separator.len_utf8());
    parts.fold(scope_bytes.saturating_add(separators), |bytes, part| bytes.saturating_add(part.len()))
}

fn traceability_seed<'a>(scope: &RecordScope, parts: impl Iterator<Item = &'a str> + Clone, separator: char) -> String {
    let values = scope_values(scope);
    let length = traceability_seed_bytes(scope, parts.clone(), separator);
    let mut seed = String::with_capacity(length);
    for (index, value) in values.into_iter().enumerate() {
        if index != 0 {
            seed.push('|');
        }
        if let Some(value) = value {
            seed.push_str(value);
        }
    }
    seed.push('|');
    for (index, part) in parts.enumerate() {
        if index != 0 {
            seed.push(separator);
        }
        seed.push_str(part);
    }
    seed
}

fn autogenerated_traceability(
    state: &RuntimeState, cycle: &CycleView, scope: RecordScope, created_at: Timestamp,
) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    let mut snapshots = test_snapshots(cycle, scope.clone(), None, created_at.clone())?;
    snapshots.extend(resource_traceability(state, cycle, scope, created_at)?);
    Ok(snapshots)
}
fn test_snapshots(
    cycle: &CycleView, scope: RecordScope, admitted: Option<&TestExecution>, created_at: Timestamp,
) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    super::payload::execution_snapshots(cycle, &scope, admitted, &created_at)
}
fn derived_snapshots(state: &RuntimeState, selected: Option<&RecordScope>, at: &Timestamp) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    let mut snapshots = Vec::new();
    for cycle in state.cycle_views() {
        let scopes = if cycle.test_executions.is_empty() {
            vec![RecordScope::default().with_cycle(cycle.test_cycle_id.clone())]
        } else {
            cycle.test_executions.iter().map(|execution| execution.scope.clone()).collect()
        };
        for scope in scopes {
            if selected.is_some_and(|selected| !scope.is_visible_in(selected)) {
                continue;
            }
            snapshots.extend(autogenerated_traceability(state, &cycle, scope, at.clone())?);
        }
    }
    Ok(snapshots)
}
fn stated_snapshot(scope: &RecordScope, section: &TraceabilitySection, created_at: &Timestamp) -> ProjectionResult<TraceabilitySnapshot> {
    super::payload::section_snapshot(scope, section, created_at)
}

fn finish_parameter_groups(groups: Vec<ParameterTraceabilityGroup>, created_at: Timestamp) -> ProjectionResult<Vec<TraceabilitySnapshot>> {
    groups
        .into_iter()
        .filter(|group| !group.entries.is_empty())
        .map(|group| traceability_group(group.scope, created_at.clone(), group.entries))
        .collect()
}

fn visit_parameter_scopes(
    state: &RuntimeState, mut visit: impl for<'a> FnMut(RecordScope, Box<dyn Iterator<Item = &'a MaterializedParameter> + 'a>),
) {
    visit(RecordScope::default(), Box::new(state.run_parameters.values()));
    for execution in state.executions.values() {
        let snapshot_parameters = execution.snapshot.parameters();
        let output_parameters = &execution.output_parameters;
        visit(
            execution.scope.clone(),
            Box::new(
                snapshot_parameters
                    .iter()
                    .filter_map(move |(key, parameter)| (!output_parameters.contains_key(key)).then_some(parameter))
                    .chain(output_parameters.values()),
            ),
        );
    }
}

fn trim_candidates(candidates: &mut BTreeMap<String, Candidate>, limit: usize) {
    while candidates.len() > limit {
        let last = candidates.keys().next_back().cloned().expect("candidate map is nonempty after insertion");
        candidates.remove(&last);
    }
}

fn traceability_group_id(scope: &RecordScope, source: &str) -> String {
    let seed = traceability_seed(scope, std::iter::once(source), '|');
    TraceabilityId::from_seed(seed).to_string()
}

fn traceability_entry_bytes(parameter: &MaterializedParameter, source: &str) -> usize {
    entry_cost(
        parameter.metadata.identifier.as_str(),
        &parameter.metadata.label,
        [parameter.metadata.description.as_deref(), Some(source)],
        &parameter.value,
    )
}

fn stated_entry_bytes(entry: &TraceabilityEntry) -> usize {
    entry_cost(entry.metadata.identifier.as_str(), &entry.metadata.label, [entry.source.as_deref(), entry.unit.as_deref()], &entry.value)
}

fn entry_cost(identifier: &str, label: &str, texts: [Option<&str>; 2], value: &Value) -> usize {
    texts
        .into_iter()
        .fold(identifier.len().saturating_add(label.len()), |bytes, text| bytes.saturating_add(text.map_or(0, str::len)))
        .saturating_add(traceability_value_bytes(value))
        .saturating_add(32)
}

fn traceability_value_bytes(value: &Value) -> usize {
    match value {
        Value::Null => 1,
        Value::Bool(_) | Value::Integer(_) | Value::Unsigned(_) | Value::Number(_) => 16,
        Value::Text(value) => value.len().saturating_add(16),
        Value::Datetime(timestamp) => dokime_domain::model::time::datetime_text(*timestamp).len().saturating_add(16),
        Value::Array(values) => values.iter().fold(16, |bytes, value| bytes.saturating_add(traceability_value_bytes(value))),
        Value::Object(values) => {
            values.iter().fold(16, |bytes, (key, value)| bytes.saturating_add(key.len()).saturating_add(traceability_value_bytes(value)))
        }
    }
}

fn traceability_group(
    scope: RecordScope, created_at: Timestamp, entries: Vec<TraceabilityEntry>,
) -> ProjectionResult<TraceabilitySnapshot> {
    super::payload::source_group(scope, created_at, entries)
}

pub(super) fn execution_parameter_values(
    execution: &dokime_domain::model::execution::result::TestExecution,
) -> impl Iterator<Item = &MaterializedParameter> {
    let mut snapshot = execution.snapshot.parameters().iter().peekable();
    let mut output = execution.output_parameters.iter().peekable();
    std::iter::from_fn(move || {
        let ordering = match (snapshot.peek(), output.peek()) {
            (Some((left, _)), Some((right, _))) => left.cmp(right),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => return None,
        };
        match ordering {
            std::cmp::Ordering::Less => snapshot.next().map(|(_, value)| value),
            std::cmp::Ordering::Equal => {
                snapshot.next();
                output.next().map(|(_, value)| value)
            }
            std::cmp::Ordering::Greater => output.next().map(|(_, value)| value),
        }
    })
}

fn collect_parameter_traceability<'a>(
    groups: &mut Vec<ParameterTraceabilityGroup>, parameter_scope: RecordScope,
    parameters: impl IntoIterator<Item = &'a MaterializedParameter>, selected_scope: Option<&RecordScope>,
) -> ProjectionResult<()> {
    if selected_scope.is_some_and(|selected| !parameter_scope.is_visible_in(selected)) {
        return Ok(());
    }
    for parameter in parameters
        .into_iter()
        .filter(|parameter| parameter_traceability_source(parameter).is_some() && resource_traceability_value_visible(&parameter.value))
    {
        let source = parameter_traceability_source(parameter).expect("traceability source was checked before entry creation");
        let entry = super::payload::parameter_entry(parameter, source)?;
        if let Some(group) = groups.iter_mut().find(|group| group.scope == parameter_scope && group.source == source) {
            group.entries.push(entry);
        } else {
            groups.push(ParameterTraceabilityGroup { scope: parameter_scope.clone(), source: source.to_owned(), entries: vec![entry] });
        }
    }
    Ok(())
}

pub(super) fn parameter_traceability_source(parameter: &MaterializedParameter) -> Option<&str> {
    if parameter.source == ParameterSource::Environment
        || parameter.group_id.as_ref().is_some_and(|id| {
            matches!(
                id.as_str(),
                "traceability.cycle"
                    | "traceability.execution"
                    | "traceability.environment"
                    | "traceability.test_definition"
                    | "traceability.test_execution"
            )
        })
    {
        return None;
    }
    if let Some(group_id) = &parameter.group_id
        && group_id.as_str().starts_with("traceability.")
    {
        return Some(group_id.as_str());
    }

    match parameter.source {
        ParameterSource::Traceability | ParameterSource::Environment | ParameterSource::Resource => Some(parameter.source.as_str()),
        _ => None,
    }
}

fn host_traceability_value_bytes(key: &str, value: &Value) -> usize {
    match super::payload::host_override_bytes(key, value) {
        Ok(Some(bytes)) => bytes.saturating_add(16),
        Ok(None) => traceability_value_bytes(value),
        Err(_) => usize::MAX,
    }
}

fn resource_traceability_groups(
    state: &RuntimeState, cycle: &CycleView, scope: &RecordScope,
) -> ProjectionResult<Vec<Vec<TraceabilityEntry>>> {
    let fallback_execution = cycle.test_executions.last();
    let test_execution_id = scope.test_execution_id.as_ref().or_else(|| fallback_execution.map(|execution| &execution.test_execution_id));
    let Some(test_execution_id) = test_execution_id else {
        return Ok(Vec::new());
    };
    let mut resources = BTreeMap::<&str, (&str, &BTreeMap<String, Value>)>::new();
    for record in state.resources.records_for_scope(scope) {
        if record.scope.as_ref().and_then(|record_scope| record_scope.test_execution_id.as_ref()) != Some(test_execution_id) {
            continue;
        }
        let instance_id = record
            .details
            .get("resource_instance_id")
            .and_then(|value| match value {
                Value::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .unwrap_or(record.resource_id.as_str());
        let label = record
            .details
            .get("resource_instance_label")
            .and_then(|value| match value {
                Value::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .or_else(|| {
                record.details.get("resource_name").and_then(|value| match value {
                    Value::Text(text) => Some(text.as_str()),
                    _ => None,
                })
            })
            .unwrap_or(instance_id);
        resources.insert(instance_id, (label, &record.details));
    }

    if resources.is_empty() {
        return Ok(Vec::new());
    }

    let mut groups = Vec::new();
    for (index, (instance_id, (label, details))) in resources.into_iter().enumerate() {
        groups.extend(resource_traceability_group(instance_id, label, details, index)?);
    }

    Ok(groups)
}

fn resource_traceability_group(
    instance_id: &str, label: &str, details: &BTreeMap<String, Value>, index: usize,
) -> ProjectionResult<Vec<Vec<TraceabilityEntry>>> {
    super::payload::resource_entries(instance_id, label, details, index)
}

pub(super) fn resource_traceability_value_visible(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Text(text) => !text.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        Value::Object(values) => !values.is_empty(),
        Value::Bool(_) | Value::Integer(_) | Value::Unsigned(_) | Value::Number(_) | Value::Datetime(_) => true,
    }
}

/* === Tests === */
