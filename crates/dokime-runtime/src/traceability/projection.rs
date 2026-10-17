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
use harmos::MemorySize;

// Local Imports
use crate::definition::{MaterializedParameter, ParameterSource};
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::execution::result::TestExecution;
use crate::state::RuntimeState;
use dokime_domain::model::traceability::LEASES_TRACEABILITY;
use dokime_domain::model::traceability::TEST_DEFINITION_TRACEABILITY;
use dokime_domain::model::traceability::TEST_EXECUTION_TRACEABILITY;
use dokime_domain::model::traceability::TraceabilityEntry;
use dokime_domain::model::traceability::TraceabilitySnapshot;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::identity::TraceabilityId;
use dokime_domain::model::values::Value;
use dokime_domain::model::execution::scope_visible_in;
use crate::telemetry::TelemetrySample;
use dokime_domain::model::traceability::TraceabilitySection;

/* === Constants === */
const TEMPLATE_TRACEABILITY_SOURCE: &str = "traceability.template";
const HOST_TRACEABILITY_FIELDS: &[(&str, &str, &str, &str)] = &[
    (
        "host_name",
        "host.hostname",
        "Hostname",
        "traceability.host.info",
    ),
    ("user", "host.user", "Username", "traceability.host.user"),
    (
        "git_user_name",
        "host.git_user_name",
        "Git User Name",
        "traceability.host.user",
    ),
    (
        "git_user_email",
        "host.git_user_email",
        "Git User Email",
        "traceability.host.user",
    ),
    ("os_name", "host.os_name", "OS", "traceability.host.info"),
    (
        "os_version",
        "host.os_version",
        "OS Version",
        "traceability.host.info",
    ),
    (
        "kernel_version",
        "host.kernel_version",
        "Kernel Version",
        "traceability.host.info",
    ),
    (
        "cpu_architecture",
        "host.cpu_architecture",
        "CPU Architecture",
        "traceability.host.cpu",
    ),
    (
        "cpu_name",
        "host.cpu_name",
        "CPU Name",
        "traceability.host.cpu",
    ),
    (
        "cpu_brand",
        "host.cpu_brand",
        "CPU Brand",
        "traceability.host.cpu",
    ),
    (
        "cpu_vendor_id",
        "host.cpu_vendor",
        "CPU Vendor",
        "traceability.host.cpu",
    ),
    (
        "cpu_frequency_mhz",
        "host.cpu_frequency_mhz",
        "CPU Frequency",
        "traceability.host.cpu",
    ),
    (
        "physical_cpu_count",
        "host.physical_cpu_count",
        "Physical CPU Count",
        "traceability.host.cpu",
    ),
    (
        "logical_cpu_count",
        "host.logical_cpu_count",
        "Logical CPU Count",
        "traceability.host.cpu",
    ),
    (
        "memory_total_bytes",
        "host.memory_total_bytes",
        "Memory Total",
        "traceability.host.ram",
    ),
    (
        "memory_used_bytes",
        "host.memory_used_bytes",
        "Memory Used",
        "traceability.host.ram",
    ),
    (
        "memory_pressure_percent",
        "host.memory_pressure",
        "Memory Pressure",
        "traceability.host.ram",
    ),
    (
        "system_uptime_s",
        "host.system_uptime",
        "System Uptime",
        "traceability.host.info",
    ),
    (
        "power_plan_name",
        "host.power_plan",
        "Power Plan",
        "traceability.host.power",
    ),
    (
        "power_mode",
        "host.power_mode",
        "Power Mode",
        "traceability.host.power",
    ),
    (
        "power_mode_performance_bias",
        "host.performance_bias",
        "Performance Bias",
        "traceability.host.power",
    ),
    (
        "battery_percent",
        "host.battery_percent",
        "Battery Percent",
        "traceability.host.power",
    ),
    (
        "battery_state",
        "host.battery_state",
        "Battery State",
        "traceability.host.power",
    ),
    (
        "battery_power_plugged",
        "host.power_plugged",
        "Plugged In",
        "traceability.host.power",
    ),
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
    state: &RuntimeState,
    cycle: &CycleView,
    scope: RecordScope,
    created_at: Timestamp,
) -> Vec<TraceabilitySnapshot> {
    resource_traceability_groups(state, cycle, &scope)
        .into_iter()
        .map(|entries| traceability_group(scope.clone(), created_at.clone(), entries))
        .collect()
}

pub fn execution_traceability(
    cycle: &CycleView,
    execution: &TestExecution,
    at: Timestamp,
) -> Vec<TraceabilitySnapshot> {
    test_snapshots(cycle, execution.scope.clone(), Some(execution), at)
}

pub fn snapshots_from_state(
    state: &RuntimeState,
    scope: Option<&RecordScope>,
    at: Timestamp,
) -> Vec<TraceabilitySnapshot> {
    let mut snapshots = derived_snapshots(state, scope, &at);
    snapshots.extend(snapshots_from_parameters(
        &state.run_parameters,
        state.executions.values(),
        scope,
        at,
    ));
    snapshots
}

pub fn snapshots_from_parameters<'a>(
    run_parameters: &'a BTreeMap<String, MaterializedParameter>,
    executions: impl IntoIterator<Item = &'a dokime_domain::model::execution::result::TestExecution>,
    scope: Option<&RecordScope>,
    created_at: Timestamp,
) -> Vec<TraceabilitySnapshot> {
    let mut groups = Vec::<ParameterTraceabilityGroup>::new();
    let executions = executions.into_iter().collect::<Vec<_>>();
    collect_parameter_traceability(
        &mut groups,
        RecordScope::default(),
        run_parameters.values(),
        scope,
    );
    for execution in &executions {
        collect_parameter_traceability(
            &mut groups,
            execution.scope.clone(),
            execution_parameter_values(execution),
            scope,
        );
    }

    let mut snapshots = finish_parameter_groups(groups, created_at.clone());
    snapshots.extend(stated_snapshots(executions, scope, &created_at));
    snapshots
}

pub(crate) fn bounded_snapshots_from_state(
    state: &RuntimeState,
    selected_scope: Option<&RecordScope>,
    after: &str,
    limit: usize,
    budget: TraceabilityPageBudget,
    additional: Option<&TraceabilityPageCandidate>,
    created_at: Timestamp,
) -> Option<(Vec<TraceabilitySnapshot>, bool, bool)> {
    let candidate_limit = limit.saturating_add(1);
    if candidate_limit == 0 {
        return Some((Vec::new(), false, false));
    }

    // Stated sections are cheap to project and few per attempt, so they are
    // built once and then compete for the page on the same terms as a
    // parameter group: by identity first, then against the same budget.
    let mut stated = stated_snapshots(state.executions.values(), selected_scope, &created_at);
    stated.extend(derived_snapshots(state, selected_scope, &created_at));
    let mut candidates = BTreeMap::<String, Candidate>::new();
    visit_parameter_scopes(state, |scope, parameters| {
        if selected_scope.is_some_and(|selected| !scope.is_visible_in(selected)) {
            return;
        }
        for parameter in parameters.filter(|parameter| {
            parameter_traceability_source(parameter).is_some()
                && resource_traceability_value_visible(&parameter.value)
        }) {
            let source = parameter_traceability_source(parameter)
                .expect("traceability source was checked before candidate creation");
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
        candidates.entry(id).or_insert(Candidate {
            parameter: None,
            additional: false,
            stated: Some(index),
        });
        trim_candidates(&mut candidates, candidate_limit);
    }

    if let Some(additional) =
        additional.filter(|candidate| candidate.traceability_id.as_str() > after)
    {
        candidates
            .entry(additional.traceability_id.clone())
            .and_modify(|candidate| candidate.additional = true)
            .or_insert(Candidate {
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
        .filter_map(|(id, candidate)| {
            candidate
                .parameter
                .as_ref()
                .map(|(scope, source)| (id.clone(), (scope.clone(), source.clone())))
        })
        .collect::<BTreeMap<_, _>>();
    let selected_stated = candidates
        .iter()
        .take(limit)
        .filter_map(|(_, candidate)| candidate.stated)
        .collect::<Vec<_>>();
    let additional_selected = additional.is_some_and(|additional| {
        additional.traceability_id.as_str() > after
            && candidates
                .iter()
                .take(limit)
                .any(|(id, candidate)| id == &additional.traceability_id && candidate.additional)
    });

    let mut entry_count = 0_usize;
    let mut estimated_bytes = 0_usize;
    visit_parameter_scopes(state, |scope, parameters| {
        if selected_scope.is_some_and(|selected| !scope.is_visible_in(selected)) {
            return;
        }
        for parameter in parameters.filter(|parameter| {
            parameter_traceability_source(parameter).is_some()
                && resource_traceability_value_visible(&parameter.value)
        }) {
            let source = parameter_traceability_source(parameter)
                .expect("traceability source was checked before budget calculation");
            let id = traceability_group_id(&scope, source);
            if selected_parameters.contains_key(&id) {
                entry_count = entry_count.saturating_add(1);
                estimated_bytes =
                    estimated_bytes.saturating_add(traceability_entry_bytes(parameter, source));
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
                    traceability_id: TraceabilityId::from_uuid(id)
                        .expect("candidate traceability id is a valid UUID"),
                    scope,
                    created_at: created_at.clone(),
                    metadata: None,
                    entries: Vec::new(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    visit_parameter_scopes(state, |scope, parameters| {
        if selected_scope.is_some_and(|selected| !scope.is_visible_in(selected)) {
            return;
        }
        for parameter in parameters.filter(|parameter| {
            parameter_traceability_source(parameter).is_some()
                && resource_traceability_value_visible(&parameter.value)
        }) {
            let source = parameter_traceability_source(parameter)
                .expect("traceability source was checked before entry creation");
            let id = traceability_group_id(&scope, source);
            if let Some(snapshot) = snapshots.get_mut(&id) {
                snapshot.entries.push(TraceabilityEntry {
                    metadata: parameter.metadata.clone(),
                    value: parameter.value.clone(),
                    source: Some(source.to_owned()),
                    unit: parameter_unit(parameter),
                });
            }
        }
    });

    // Into the same map, not appended after it: the page's cursor is the last
    // id it returned, so a stated section that sorted below a parameter group
    // would make the next page re-deliver rows the caller already has.
    for index in selected_stated {
        let snapshot = stated[index].clone();
        snapshots.insert(snapshot.traceability_id.to_string(), snapshot);
    }

    Some((
        snapshots.into_values().collect(),
        has_more,
        additional_selected,
    ))
}

pub fn snapshots_from_parameter_scopes<'a>(
    parameters: impl IntoIterator<Item = (RecordScope, &'a BTreeMap<String, MaterializedParameter>)>,
    scope: Option<&RecordScope>,
    created_at: Timestamp,
) -> Vec<TraceabilitySnapshot> {
    let mut groups = Vec::new();
    for (parameter_scope, values) in parameters {
        collect_parameter_traceability(&mut groups, parameter_scope, values.values(), scope);
    }
    finish_parameter_groups(groups, created_at)
}

pub(crate) fn snapshots_from_execution_parameters(
    run_parameters: &BTreeMap<String, MaterializedParameter>,
    execution: &dokime_domain::model::execution::result::TestExecution,
    scope: Option<&RecordScope>,
    created_at: Timestamp,
) -> Vec<TraceabilitySnapshot> {
    let mut groups = Vec::new();
    collect_parameter_traceability(
        &mut groups,
        RecordScope::default(),
        run_parameters.values(),
        scope,
    );
    collect_parameter_traceability(
        &mut groups,
        execution.scope.clone(),
        execution_parameter_values(execution),
        scope,
    );
    finish_parameter_groups(groups, created_at)
}

pub fn stated_snapshots<'a>(
    executions: impl IntoIterator<Item = &'a TestExecution>,
    scope: Option<&RecordScope>,
    created_at: &Timestamp,
) -> Vec<TraceabilitySnapshot> {
    executions
        .into_iter()
        .filter(|execution| scope.is_none_or(|selected| execution.scope.is_visible_in(selected)))
        .flat_map(|execution| {
            execution
                .traceability
                .iter()
                .map(|section| stated_snapshot(&execution.scope, section, created_at))
        })
        .collect()
}

pub(crate) fn stated_execution_memory(
    execution: &TestExecution,
    created_at: &Timestamp,
) -> crate::runtime::Result<(usize, usize)> {
    let mut retained = std::mem::size_of::<Vec<TraceabilitySnapshot>>();
    let mut seed_peak = 0;
    for section in &execution.traceability {
        let mut bytes = std::mem::size_of::<TraceabilitySnapshot>().saturating_add(36);
        for (owned, inline) in [
            (
                execution.scope.memory_size(),
                std::mem::size_of::<RecordScope>(),
            ),
            (created_at.memory_size(), std::mem::size_of::<Timestamp>()),
            (
                section.identifier.memory_size(),
                std::mem::size_of::<Identifier>(),
            ),
        ] {
            if owned == usize::MAX {
                return Err(crate::runtime::Error::Operation(
                    "authored traceability memory is unbounded".to_owned(),
                ));
            }
            bytes = bytes.saturating_add(owned.saturating_sub(inline));
        }
        let attributes = harmos::btree_map_memory::<String, Value>(0)
            .saturating_sub(std::mem::size_of::<BTreeMap<String, Value>>());
        bytes = bytes
            .saturating_add(section.label.len())
            .saturating_add(attributes);
        for row in section
            .rows
            .iter()
            .filter(|row| resource_traceability_value_visible(&row.value))
        {
            let mut entry = std::mem::size_of::<TraceabilityEntry>();
            for (owned, inline) in [
                (
                    row.identifier.memory_size(),
                    std::mem::size_of::<Identifier>(),
                ),
                (row.value.memory_size(), std::mem::size_of::<Value>()),
            ] {
                if owned == usize::MAX {
                    return Err(crate::runtime::Error::Operation(
                        "authored traceability row memory is unbounded".to_owned(),
                    ));
                }
                entry = entry.saturating_add(owned.saturating_sub(inline));
            }
            entry = entry
                .saturating_add(row.label.len())
                .saturating_add(row.unit.as_ref().map_or(0, String::len))
                .saturating_add(stated_source_bytes(row.source.as_deref()))
                .saturating_add(attributes);
            bytes = bytes.saturating_add(entry);
        }
        retained = retained.saturating_add(bytes);
        let seed = traceability_seed_bytes(
            &execution.scope,
            [TEMPLATE_TRACEABILITY_SOURCE, section.identifier.as_str()].into_iter(),
            '.',
        )
        .saturating_add(std::mem::size_of::<String>());
        seed_peak = seed_peak.max(seed);
    }
    if retained == usize::MAX || seed_peak == usize::MAX {
        return Err(crate::runtime::Error::Operation(
            "authored traceability construction memory overflows".to_owned(),
        ));
    }
    Ok((retained, seed_peak))
}

pub(crate) fn stated_execution_snapshots(
    execution: &TestExecution,
    created_at: &Timestamp,
    limit: usize,
) -> crate::runtime::Result<Vec<TraceabilitySnapshot>> {
    let (retained, seed) = stated_execution_memory(execution, created_at)?;
    if retained.checked_add(seed).is_none_or(|bytes| bytes > limit) {
        return Err(crate::runtime::Error::Operation(
            "authored traceability exceeds construction budget".to_owned(),
        ));
    }
    let count = execution.traceability.len();
    let mut snapshots = Vec::new();
    snapshots
        .try_reserve_exact(count)
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
    if snapshots.capacity() > count {
        return Err(crate::runtime::Error::Operation(
            "authored traceability slots exceed construction budget".to_owned(),
        ));
    }
    for section in &execution.traceability {
        snapshots.push(stated_snapshot(&execution.scope, section, created_at));
    }
    if snapshots.memory_size() > retained {
        return Err(crate::runtime::Error::Operation(
            "authored traceability exceeds original retained bound".to_owned(),
        ));
    }
    Ok(snapshots)
}

pub fn host_traceability_snapshot_from_samples<'a, I>(
    samples: I,
    selected_scope: Option<&RecordScope>,
    created_at: Timestamp,
) -> Option<TraceabilitySnapshot>
where
    I: IntoIterator<Item = &'a TelemetrySample>,
    I::IntoIter: DoubleEndedIterator,
{
    let sample = samples.into_iter().rev().find(|sample| {
        sample.stream.as_str() == "host.system"
            && selected_scope.is_none_or(|scope| scope_visible_in(sample.scope.as_ref(), scope))
    })?;
    let entries = host_traceability_entries_from_values(&sample.values);

    (!entries.is_empty()).then(|| {
        traceability_group(
            sample.scope.clone().unwrap_or_default(),
            created_at,
            entries,
        )
    })
}

pub fn bounded_host_traceability_snapshot_from_sample(
    sample: &TelemetrySample,
    selected_scope: Option<&RecordScope>,
    max_entries: usize,
    max_bytes: usize,
    created_at: Timestamp,
) -> Result<Option<TraceabilitySnapshot>, String> {
    let Some(candidate) = host_traceability_page_candidate(sample, selected_scope) else {
        return Ok(None);
    };
    if candidate.entry_count > max_entries || candidate.estimated_bytes > max_bytes {
        return Err("host traceability sample exceeds the bounded page budget".to_owned());
    }

    Ok(Some(traceability_group(
        sample.scope.clone().unwrap_or_default(),
        created_at,
        host_traceability_entries_from_values(&sample.values),
    )))
}

pub(crate) fn host_traceability_page_candidate(
    sample: &TelemetrySample,
    selected_scope: Option<&RecordScope>,
) -> Option<TraceabilityPageCandidate> {
    if sample.stream.as_str() != "host.system"
        || selected_scope.is_some_and(|scope| !scope_visible_in(sample.scope.as_ref(), scope))
    {
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
        traceability_id: TraceabilityId::from_seed(format!(
            "{}|{discriminator}",
            scope_discriminator(&scope)
        ))
        .to_string(),
        entry_count,
        estimated_bytes,
    })
}

pub fn replace_snapshots_by_source(
    snapshots: &mut Vec<TraceabilitySnapshot>,
    source: &str,
    replacement: TraceabilitySnapshot,
) {
    for snapshot in snapshots.iter_mut() {
        snapshot.entries.retain(|entry| {
            entry.source.as_deref().is_none_or(|entry_source| {
                entry_source != source
                    && !entry_source
                        .strip_prefix(source)
                        .is_some_and(|suffix| suffix.starts_with('.'))
            })
        });
    }
    snapshots.retain(|snapshot| !snapshot.entries.is_empty());
    snapshots.push(replacement);
}

pub fn lease_traceability_snapshot(
    leases: &[crate::telemetry_runtime::LeaseFact],
    scope: &RecordScope,
    created_at: Timestamp,
) -> Option<TraceabilitySnapshot> {
    let mut entries = Vec::new();
    for lease in leases {
        let stream = lease.stream.as_str();
        entries.push(entry(
            &format!("{stream}.owner"),
            &format!("{stream} owner"),
            Value::from(lease.owner.clone()),
            LEASES_TRACEABILITY.source,
        ));
        entries.push(entry(
            &format!("{stream}.leased_from"),
            &format!("{stream} leased from"),
            Value::from(lease.opened_at.to_string()),
            LEASES_TRACEABILITY.source,
        ));
        // An open hold belongs to a run that is still going; saying so beats
        // writing an instant that has not happened yet.
        entries.push(entry(
            &format!("{stream}.leased_until"),
            &format!("{stream} leased until"),
            Value::from(
                lease
                    .closed_at
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "open".to_owned()),
            ),
            LEASES_TRACEABILITY.source,
        ));
        if !lease.shared_with.is_empty() {
            entries.push(entry(
                &format!("{stream}.shared_with"),
                &format!("{stream} shared with"),
                Value::from(lease.shared_with.join(", ")),
                LEASES_TRACEABILITY.source,
            ));
        }
        if lease.gap {
            entries.push(entry(
                &format!("{stream}.gap"),
                &format!("{stream} gap"),
                Value::from(
                    "the stream's owner restarted while this run held it, so rows are missing for that interval".to_owned(),
                ),
                LEASES_TRACEABILITY.source,
            ));
        }
    }

    (!entries.is_empty()).then(|| traceability_group(scope.clone(), created_at, entries))
}

pub(crate) fn scope_discriminator(scope: &RecordScope) -> String {
    let values = scope_values(scope);
    let length = values
        .iter()
        .map(|value| value.map_or(0, str::len))
        .sum::<usize>()
        + values.len()
        - 1;
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
fn scope_values(scope: &RecordScope) -> [Option<&str>; 8] {
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

fn traceability_seed_bytes<'a>(
    scope: &RecordScope,
    parts: impl Iterator<Item = &'a str> + Clone,
    separator: char,
) -> usize {
    let values = scope_values(scope);
    let scope_bytes = values.iter().fold(values.len(), |bytes, value| {
        bytes.saturating_add(value.map_or(0, str::len))
    });
    let separators = parts
        .clone()
        .count()
        .saturating_sub(1)
        .saturating_mul(separator.len_utf8());
    parts.fold(scope_bytes.saturating_add(separators), |bytes, part| {
        bytes.saturating_add(part.len())
    })
}

fn traceability_seed<'a>(
    scope: &RecordScope,
    parts: impl Iterator<Item = &'a str> + Clone,
    separator: char,
) -> String {
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
    state: &RuntimeState,
    cycle: &CycleView,
    scope: RecordScope,
    created_at: Timestamp,
) -> Vec<TraceabilitySnapshot> {
    let mut snapshots = test_snapshots(cycle, scope.clone(), None, created_at.clone());
    snapshots.extend(resource_traceability(state, cycle, scope, created_at));
    snapshots
}

fn test_snapshots(
    cycle: &CycleView,
    scope: RecordScope,
    admitted: Option<&TestExecution>,
    created_at: Timestamp,
) -> Vec<TraceabilitySnapshot> {
    let mut groups = BTreeMap::<String, Vec<TraceabilityEntry>>::new();
    let mut entries = cycle_traceability_entries(cycle);
    if let Some(execution_entries) = case_execution_traceability_entries(cycle, &scope, admitted) {
        // Once an attempt exists its admitted suite is authoritative.
        entries.retain(|entry| !entry.metadata.identifier.as_str().starts_with("suite."));
        entries.extend(execution_entries);
    }
    for entry in entries {
        groups
            .entry(entry.source.clone().unwrap_or_default())
            .or_default()
            .push(entry);
    }
    groups
        .into_values()
        .map(|entries| traceability_group(scope.clone(), created_at.clone(), entries))
        .collect::<Vec<_>>()
}

fn derived_snapshots(
    state: &RuntimeState,
    selected: Option<&RecordScope>,
    at: &Timestamp,
) -> Vec<TraceabilitySnapshot> {
    let mut snapshots = Vec::new();
    for cycle in state.cycle_views() {
        let scopes = if cycle.test_executions.is_empty() {
            vec![RecordScope::default().with_cycle(cycle.test_cycle_id.clone())]
        } else {
            cycle
                .test_executions
                .iter()
                .map(|execution| execution.scope.clone())
                .collect()
        };
        for scope in scopes {
            if selected.is_some_and(|selected| !scope.is_visible_in(selected)) {
                continue;
            }
            snapshots.extend(autogenerated_traceability(state, &cycle, scope, at.clone()));
        }
    }
    snapshots
}

fn stated_snapshot(
    scope: &RecordScope,
    section: &TraceabilitySection,
    created_at: &Timestamp,
) -> TraceabilitySnapshot {
    let rows = section
        .rows
        .iter()
        .filter(|row| resource_traceability_value_visible(&row.value));
    let mut entries = Vec::with_capacity(rows.clone().count());
    for row in rows {
        entries.push(TraceabilityEntry {
            metadata: Metadata::new(row.identifier.clone(), row.label.clone(), None),
            value: row.value.clone(),
            source: Some(stated_source(row.source.as_deref())),
            unit: row.unit.clone(),
        });
    }
    let seed = traceability_seed(
        scope,
        [TEMPLATE_TRACEABILITY_SOURCE, section.identifier.as_str()].into_iter(),
        '.',
    );
    TraceabilitySnapshot {
        traceability_id: TraceabilityId::from_seed(seed),
        scope: scope.clone(),
        created_at: created_at.clone(),
        metadata: Some(Metadata::new(
            section.identifier.clone(),
            section.label.clone(),
            None,
        )),
        entries,
    }
}

fn stated_source_bytes(declared: Option<&str>) -> usize {
    match declared.filter(|declared| !declared.is_empty()) {
        Some(declared) => TEMPLATE_TRACEABILITY_SOURCE
            .len()
            .saturating_add(1)
            .saturating_add(declared.chars().count()),
        None => TEMPLATE_TRACEABILITY_SOURCE.len(),
    }
}

fn stated_source(declared: Option<&str>) -> String {
    let Some(declared) = declared.filter(|declared| !declared.is_empty()) else {
        return TEMPLATE_TRACEABILITY_SOURCE.to_owned();
    };
    let length = stated_source_bytes(Some(declared));
    let mut source = String::with_capacity(length);
    source.push_str(TEMPLATE_TRACEABILITY_SOURCE);
    source.push('.');
    for character in declared.chars() {
        source.push(
            if character.is_ascii_alphanumeric() || character == '.' || character == '_' {
                character
            } else {
                '_'
            },
        );
    }
    source
}

fn finish_parameter_groups(
    groups: Vec<ParameterTraceabilityGroup>,
    created_at: Timestamp,
) -> Vec<TraceabilitySnapshot> {
    groups
        .into_iter()
        .filter(|group| !group.entries.is_empty())
        .map(|group| traceability_group(group.scope, created_at.clone(), group.entries))
        .collect()
}

fn visit_parameter_scopes(
    state: &RuntimeState,
    mut visit: impl for<'a> FnMut(RecordScope, Box<dyn Iterator<Item = &'a MaterializedParameter> + 'a>),
) {
    visit(
        RecordScope::default(),
        Box::new(state.run_parameters.values()),
    );
    for execution in state.executions.values() {
        let snapshot_parameters = execution.snapshot.parameters();
        let output_parameters = &execution.output_parameters;
        visit(
            execution.scope.clone(),
            Box::new(
                snapshot_parameters
                    .iter()
                    .filter_map(move |(key, parameter)| {
                        (!output_parameters.contains_key(key)).then_some(parameter)
                    })
                    .chain(output_parameters.values()),
            ),
        );
    }
}

fn trim_candidates(candidates: &mut BTreeMap<String, Candidate>, limit: usize) {
    while candidates.len() > limit {
        let last = candidates
            .keys()
            .next_back()
            .cloned()
            .expect("candidate map is nonempty after insertion");
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
    entry_cost(
        entry.metadata.identifier.as_str(),
        &entry.metadata.label,
        [entry.source.as_deref(), entry.unit.as_deref()],
        &entry.value,
    )
}

fn entry_cost(identifier: &str, label: &str, texts: [Option<&str>; 2], value: &Value) -> usize {
    texts
        .into_iter()
        .fold(
            identifier.len().saturating_add(label.len()),
            |bytes, text| bytes.saturating_add(text.map_or(0, str::len)),
        )
        .saturating_add(traceability_value_bytes(value))
        .saturating_add(32)
}

fn traceability_value_bytes(value: &Value) -> usize {
    match value {
        Value::Null => 1,
        Value::Bool(_) | Value::Integer(_) | Value::Unsigned(_) | Value::Number(_) => 16,
        Value::Text(value) => value.len().saturating_add(16),
        Value::Datetime(timestamp) => dokime_domain::model::time::datetime_text(*timestamp)
            .len()
            .saturating_add(16),
        Value::Array(values) => values.iter().fold(16, |bytes, value| {
            bytes.saturating_add(traceability_value_bytes(value))
        }),
        Value::Object(values) => values.iter().fold(16, |bytes, (key, value)| {
            bytes
                .saturating_add(key.len())
                .saturating_add(traceability_value_bytes(value))
        }),
    }
}

fn traceability_group(
    scope: RecordScope,
    created_at: Timestamp,
    entries: Vec<TraceabilityEntry>,
) -> TraceabilitySnapshot {
    let mut discriminator = entries
        .iter()
        .filter_map(|entry| entry.source.as_deref())
        .collect::<BTreeSet<_>>();
    if discriminator.is_empty() || (discriminator.len() == 1 && discriminator.contains("")) {
        discriminator = entries
            .iter()
            .map(|entry| entry.metadata.identifier.as_str())
            .collect();
    }
    let seed = traceability_seed(&scope, discriminator.iter().copied(), '|');
    drop(discriminator);
    TraceabilitySnapshot {
        traceability_id: TraceabilityId::from_seed(seed),
        scope,
        created_at,
        metadata: None,
        entries,
    }
}

fn execution_parameter_values(
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
    groups: &mut Vec<ParameterTraceabilityGroup>,
    parameter_scope: RecordScope,
    parameters: impl IntoIterator<Item = &'a MaterializedParameter>,
    selected_scope: Option<&RecordScope>,
) {
    if selected_scope.is_some_and(|selected| !parameter_scope.is_visible_in(selected)) {
        return;
    }

    for parameter in parameters.into_iter().filter(|parameter| {
        parameter_traceability_source(parameter).is_some()
            && resource_traceability_value_visible(&parameter.value)
    }) {
        let source = parameter_traceability_source(parameter)
            .expect("traceability source was checked before entry creation");
        let entry = TraceabilityEntry {
            metadata: parameter.metadata.clone(),
            value: parameter.value.clone(),
            source: Some(source.to_owned()),
            unit: parameter_unit(parameter),
        };
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.scope == parameter_scope && group.source == source)
        {
            group.entries.push(entry);
        } else {
            groups.push(ParameterTraceabilityGroup {
                scope: parameter_scope.clone(),
                source: source.to_owned(),
                entries: vec![entry],
            });
        }
    }
}

fn parameter_traceability_source(parameter: &MaterializedParameter) -> Option<&str> {
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
        ParameterSource::Traceability
        | ParameterSource::Environment
        | ParameterSource::Resource => Some(parameter.source.as_str()),
        _ => None,
    }
}

fn entry(identifier_value: &str, label: &str, value: Value, source: &str) -> TraceabilityEntry {
    TraceabilityEntry {
        metadata: Metadata::new(identifier(identifier_value), label, None),
        value,
        source: Some(source.to_owned()),
        unit: None,
    }
}

fn parameter_unit(parameter: &MaterializedParameter) -> Option<String> {
    parameter
        .unit
        .as_ref()
        .filter(|unit| !unit.symbol.is_empty())
        .map(|unit| unit.symbol.clone())
}

fn cycle_traceability_entries(cycle: &CycleView) -> Vec<TraceabilityEntry> {
    vec![
        entry(
            "cycle.created_at",
            "Cycle Created At",
            Value::from(cycle.created_at.to_string()),
            TEST_EXECUTION_TRACEABILITY.source,
        ),
        entry(
            "cycle.updated_at",
            "Cycle Updated At",
            Value::from(cycle.updated_at.to_string()),
            TEST_EXECUTION_TRACEABILITY.source,
        ),
        entry(
            "cycle.label",
            "Cycle Label",
            Value::from(cycle.metadata.label.clone()),
            TEST_EXECUTION_TRACEABILITY.source,
        ),
        entry(
            "cycle.case_count",
            "Case Count",
            Value::Integer(cycle.cases().count() as i64),
            TEST_EXECUTION_TRACEABILITY.source,
        ),
        entry(
            "cycle.continue_on_failure",
            "Continue On Failure",
            Value::from(cycle.continue_on_failure),
            TEST_EXECUTION_TRACEABILITY.source,
        ),
    ]
}

fn host_traceability_entries_from_values(
    values: &BTreeMap<String, Value>,
) -> Vec<TraceabilityEntry> {
    let mut entries = Vec::new();

    for (key, identifier_value, label, source) in HOST_TRACEABILITY_FIELDS {
        push_host_traceability_entry(&mut entries, values, key, identifier_value, label, source);
    }

    entries
}

fn host_traceability_value_bytes(key: &str, value: &Value) -> usize {
    host_display_override(key, value)
        .map_or_else(|| traceability_value_bytes(value), |text| text.len() + 16)
}

fn host_display_override(key: &str, value: &Value) -> Option<String> {
    match (key, value) {
        ("memory_total_bytes" | "memory_used_bytes", Value::Integer(bytes)) => {
            Some(format!("{:.2} GB", *bytes as f64 / 1_000_000_000.0))
        }
        ("system_uptime_s", Value::Number(seconds)) => Some(format_duration(*seconds)),
        ("system_uptime_s", Value::Integer(seconds)) => Some(format_duration(*seconds as f64)),
        _ => None,
    }
}

fn push_host_traceability_entry(
    entries: &mut Vec<TraceabilityEntry>,
    values: &BTreeMap<String, Value>,
    key: &str,
    identifier_value: &str,
    label: &str,
    source: &str,
) {
    let Some(value) = values.get(key) else {
        return;
    };
    if !resource_traceability_value_visible(value) {
        return;
    }
    entries.push(entry(
        identifier_value,
        label,
        host_traceability_value(key, value),
        source,
    ));
}

fn host_traceability_value(key: &str, value: &Value) -> Value {
    host_display_override(key, value).map_or_else(|| value.clone(), Value::from)
}

fn format_duration(seconds: f64) -> String {
    let total_minutes = (seconds.max(0.0) / 60.0).round() as u64;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    match (hours, minutes) {
        (0, minutes) => format!("{minutes} min"),
        (hours, 0) => format!("{hours} h"),
        (hours, minutes) => format!("{hours} h {minutes} min"),
    }
}

fn case_execution_traceability_entries(
    cycle: &CycleView,
    scope: &RecordScope,
    admitted: Option<&TestExecution>,
) -> Option<Vec<TraceabilityEntry>> {
    let execution = if let Some(id) = &scope.test_execution_id {
        admitted
            .filter(|execution| &execution.test_execution_id == id)
            .or_else(|| {
                cycle
                    .test_executions
                    .iter()
                    .find(|execution| &execution.test_execution_id == id)
            })
    } else {
        admitted
            .filter(|execution| {
                scope
                    .test_case_id
                    .as_ref()
                    .is_none_or(|id| &execution.test_case_id == id)
            })
            .or_else(|| {
                cycle.test_executions.iter().rev().find(|execution| {
                    scope
                        .test_case_id
                        .as_ref()
                        .is_none_or(|id| &execution.test_case_id == id)
                })
            })
    }?;
    let case_id = &execution.test_case_id;
    if scope.test_case_id.as_ref().is_some_and(|id| id != case_id) {
        return None;
    }
    let case_index = cycle
        .cases()
        .position(|candidate| &candidate.test_case_id == case_id);
    let snapshot = &execution.snapshot;
    let case = snapshot.case();
    let mut entries = Vec::new();

    push_execution_entry(
        &mut entries,
        "case.label",
        "Case",
        Value::from(case.metadata.label.clone()),
        TEST_DEFINITION_TRACEABILITY.source,
    );
    if let Some(index) = case_index {
        push_execution_entry(
            &mut entries,
            "case.ordinal",
            "Case Position",
            Value::Integer(index as i64 + 1),
            TEST_DEFINITION_TRACEABILITY.source,
        );
    }
    push_execution_entry(
        &mut entries,
        "execution.case_id",
        "Case ID",
        Value::from(case_id.to_string()),
        TEST_DEFINITION_TRACEABILITY.source,
    );

    // Execution provenance comes from its frozen case, even after plan edits.
    // Template version is not declared by TestCaseTemplate. A reader names the
    // method by the identifier its author gave it; the UUID stands in only
    // when the frozen snapshot carries no template.
    push_execution_entry(
        &mut entries,
        "execution.template",
        "Test Method",
        Value::from(snapshot.template().map_or_else(
            || case.test_case_template_id.to_string(),
            |template| template.metadata.identifier.as_str().to_owned(),
        )),
        TEST_DEFINITION_TRACEABILITY.source,
    );
    for (identifier, label, timestamp) in [
        (
            "execution.started_at",
            "Started At",
            execution.started_at.as_ref(),
        ),
        (
            "execution.finished_at",
            "Finished At",
            execution.finished_at.as_ref(),
        ),
    ] {
        if let Some(timestamp) = timestamp {
            push_execution_entry(
                &mut entries,
                identifier,
                label,
                Value::from(timestamp.to_string()),
                TEST_EXECUTION_TRACEABILITY.source,
            );
        }
    }
    if let Some(suite) = snapshot.suite() {
        push_execution_entry(
            &mut entries,
            "execution.suite",
            "Suite",
            Value::from(suite.identifier.as_str()),
            TEST_DEFINITION_TRACEABILITY.source,
        );
        push_execution_entry(
            &mut entries,
            "execution.suite_label",
            "Suite Label",
            Value::from(suite.label.clone()),
            TEST_DEFINITION_TRACEABILITY.source,
        );
    }
    push_execution_entry(
        &mut entries,
        "execution.status",
        "Execution Status",
        Value::from(format!("{:?}", execution.status)),
        TEST_EXECUTION_TRACEABILITY.source,
    );

    if let Some(plugin) = snapshot.plugin() {
        push_execution_entry(
            &mut entries,
            "execution.plugin",
            "Plugin",
            Value::from(plugin.source.clone()),
            TEST_DEFINITION_TRACEABILITY.source,
        );
        if let Some(version) = &plugin.version {
            push_execution_entry(
                &mut entries,
                "execution.plugin_version",
                "Plugin Version",
                Value::from(version.clone()),
                TEST_DEFINITION_TRACEABILITY.source,
            );
        }
        let (key, label, value) = match &plugin.implementation {
            dokime_domain::model::execution::snapshot::ExecutionImplementation::Bundle { content_digest } => {
                ("execution.plugin_digest", "Plugin Digest", content_digest)
            }
            dokime_domain::model::execution::snapshot::ExecutionImplementation::Embedded { generation } => (
                "execution.provider_generation",
                "Provider Generation",
                generation,
            ),
        };
        push_execution_entry(
            &mut entries,
            key,
            label,
            Value::from(value.clone()),
            TEST_DEFINITION_TRACEABILITY.source,
        );
    }

    if let Some(version) = snapshot.host_version() {
        push_execution_entry(
            &mut entries,
            "execution.dokime_version",
            "Dokime Version",
            Value::from(version),
            TEST_DEFINITION_TRACEABILITY.source,
        );
    }

    Some(entries)
}

fn push_object_entries(
    entries: &mut Vec<TraceabilityEntry>,
    object: Option<&Value>,
    prefix: &str,
    source: &str,
) {
    let Some(Value::Object(members)) = object else {
        return;
    };

    for (key, value) in members {
        if !resource_traceability_value_visible(value) {
            continue;
        }
        entries.push(entry(
            &format!("{prefix}.{}", sanitize_identifier(key)),
            &parameter_label(key),
            value.clone(),
            source,
        ));
    }
}

fn push_execution_entry(
    entries: &mut Vec<TraceabilityEntry>,
    identifier_value: &str,
    label: &str,
    value: Value,
    source: &str,
) {
    if !resource_traceability_value_visible(&value) {
        return;
    }
    entries.push(entry(identifier_value, label, value, source));
}

fn resource_traceability_groups(
    state: &RuntimeState,
    cycle: &CycleView,
    scope: &RecordScope,
) -> Vec<Vec<TraceabilityEntry>> {
    let fallback_execution = cycle.test_executions.last();
    let test_execution_id = scope
        .test_execution_id
        .as_ref()
        .or_else(|| fallback_execution.map(|execution| &execution.test_execution_id));
    let Some(test_execution_id) = test_execution_id else {
        return Vec::new();
    };
    let mut resources = BTreeMap::<&str, (&str, &BTreeMap<String, Value>)>::new();
    for record in state.resources.records_for_scope(scope) {
        if record
            .scope
            .as_ref()
            .and_then(|record_scope| record_scope.test_execution_id.as_ref())
            != Some(test_execution_id)
        {
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
                record
                    .details
                    .get("resource_name")
                    .and_then(|value| match value {
                        Value::Text(text) => Some(text.as_str()),
                        _ => None,
                    })
            })
            .unwrap_or(instance_id);
        resources.insert(instance_id, (label, &record.details));
    }

    if resources.is_empty() {
        return Vec::new();
    }

    let mut groups = Vec::new();
    for (index, (instance_id, (label, details))) in resources.into_iter().enumerate() {
        groups.extend(resource_traceability_group(
            instance_id,
            label,
            details,
            index,
        ));
    }

    groups
}

fn resource_traceability_group(
    instance_id: &str,
    label: &str,
    details: &BTreeMap<String, Value>,
    index: usize,
) -> Vec<Vec<TraceabilityEntry>> {
    let instance_id = sanitize_identifier(instance_id);
    let prefix = format!("resource.{instance_id}");
    let resource_source = format!("traceability.resources.{instance_id}");
    let device_source = format!("traceability.devices.{instance_id}");
    let python_source = format!("traceability.python_interfaces.{instance_id}");
    let mut groups = Vec::new();

    let mut entries = Vec::new();
    entries.push(entry(
        &format!("{prefix}.name"),
        "Resource",
        Value::from(label),
        &resource_source,
    ));
    if let Some(name) = details.get("resource_name").and_then(|value| match value {
        Value::Text(text) => Some(text.clone()),
        _ => None,
    }) && name != label
    {
        entries.push(entry(
            &format!("{prefix}.resource_name"),
            "Resource Name",
            Value::from(name),
            &resource_source,
        ));
    }
    if let Some(kind) = details.get("resource_kind").cloned() {
        entries.push(entry(
            &format!("{prefix}.kind"),
            "Resource Kind",
            kind,
            &resource_source,
        ));
    }
    if let Some(path) = details.get("path").cloned() {
        entries.push(entry(
            &format!("{prefix}.path"),
            "Resource Path",
            path,
            &resource_source,
        ));
    }
    push_resource_detail(
        &mut entries,
        details,
        &prefix,
        "connection",
        "Connection",
        &resource_source,
    );
    push_resource_detail(
        &mut entries,
        details,
        &prefix,
        "endpoint",
        "Endpoint",
        &resource_source,
    );
    push_resource_detail(
        &mut entries,
        details,
        &prefix,
        "process_isolation",
        "Process Isolation",
        &resource_source,
    );
    if let Some(error) = details.get("traceability_error").cloned() {
        entries.push(entry(
            &format!("{prefix}.traceability_error"),
            "Traceability Error",
            error,
            &resource_source,
        ));
    }
    push_object_entries(
        &mut entries,
        details.get("traceability"),
        &format!("{prefix}.traceability"),
        &resource_source,
    );
    entries.push(entry(
        &format!("{prefix}.order"),
        "Resource Order",
        Value::Integer(index as i64 + 1),
        &resource_source,
    ));
    groups.push(entries);

    for (field, entry_prefix, source) in [
        (
            "device_info",
            format!("{prefix}.device_info"),
            &device_source,
        ),
        (
            "python_interfaces",
            format!("{prefix}.python_interface"),
            &python_source,
        ),
    ] {
        let mut entries = Vec::new();
        push_object_entries(&mut entries, details.get(field), &entry_prefix, source);
        if !entries.is_empty() {
            groups.push(entries);
        }
    }

    groups
}

fn resource_traceability_value_visible(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Text(text) => !text.trim().is_empty(),
        Value::Array(values) => !values.is_empty(),
        Value::Object(values) => !values.is_empty(),
        Value::Bool(_)
        | Value::Integer(_)
        | Value::Unsigned(_)
        | Value::Number(_)
        | Value::Datetime(_) => true,
    }
}

fn push_resource_detail(
    entries: &mut Vec<TraceabilityEntry>,
    details: &BTreeMap<String, Value>,
    prefix: &str,
    key: &str,
    label: &str,
    source: &str,
) {
    if let Some(value) = details.get(key).cloned() {
        entries.push(entry(&format!("{prefix}.{key}"), label, value, source));
    }
}

fn parameter_label(key: &str) -> String {
    let parts = key.split(['.', '_', '-']).filter(|part| !part.is_empty());
    let count = parts.clone().count();
    let length = parts
        .clone()
        .map(|part| {
            let mut chars = part.chars();
            let first = chars.next().expect("empty parts were filtered");
            first.to_uppercase().map(char::len_utf8).sum::<usize>() + chars.as_str().len()
        })
        .sum::<usize>()
        + count.saturating_sub(1);
    let mut label = String::with_capacity(length);
    for (index, part) in parts.enumerate() {
        if index != 0 {
            label.push(' ');
        }
        let mut chars = part.chars();
        let first = chars.next().expect("empty parts were filtered");
        for character in first.to_uppercase() {
            label.push(character);
        }
        label.push_str(chars.as_str());
    }
    label
}

fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '.' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn identifier(value: &str) -> Identifier {
    Identifier::new(value).expect("built-in runtime traceability identifiers are valid")
}

/* === Tests === */
