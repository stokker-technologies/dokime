/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod catalog;
mod executions;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Local Imports
use super::shared::Map;
use super::{
    DokimeIdentity, EvidenceRecords, InstalledPluginErrorRecord, MaterializedParameter, PluginOwner, PromptDefinition, PromptId,
    PromptInstance, PromptResponse, ResourceRegistry, RuntimeState, SessionId, TestCycleId, record_history,
};
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::NoteId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// The managed snapshot's state encoding. Catalogs and execution plans are
// stored once and referenced by position (see `catalog` and `executions`), so
// the snapshot's byte cap bounds distinct content rather than its copies. The
// encoder and decoder list every field: adding one to RuntimeState fails to
// compile until both carry it.
#[derive(Serialize)]
struct Encoded<'a> {
    // The Shared Catalog Precedes Every Owner That References It
    catalog: &'a catalog::Pool<'a>,
    session_id: &'a SessionId,
    record_namespace: &'a SessionId,
    identity: &'a DokimeIdentity,
    run_parameters: &'a BTreeMap<String, MaterializedParameter>,
    shutting_down: bool,
    pending_effects: &'a Vec<dokime_domain::model::execution::effects::PendingEffect>,
    installed_plugins: BTreeMap<&'a String, catalog::Plugin>,
    plugin_owners: &'a BTreeMap<String, PluginOwner>,
    suites: catalog::Registry<'a>,
    record_definitions: BTreeMap<&'a Identifier, u32>,
    record_history: BTreeMap<&'a Identifier, BTreeMap<u32, u32>>,
    telemetry_definitions: BTreeMap<&'a Identifier, u32>,
    cycles: &'a Map<TestCycleId, dokime_domain::model::execution::result::TestCycle>,

    #[serde(serialize_with = "executions::serialize")]
    executions: &'a Map<TestExecutionId, dokime_domain::model::execution::result::TestExecution>,

    retired_executions: &'a BTreeSet<TestExecutionId>,
    run_requests: &'a Map<CommandId, crate::transactions::provider::request::RunRequest>,
    run_queue: &'a std::collections::VecDeque<CommandId>,
    cycle_report_finalizations: &'a Map<TestCycleId, crate::transactions::CycleReportFinalization>,
    report_finalizations: &'a Map<TestExecutionId, crate::transactions::ReportFinalization>,
    note_admissions: &'a Map<NoteId, dokime_domain::model::execution::notes::NoteAdmission>,
    records: &'a EvidenceRecords,
    resources: &'a ResourceRegistry,
    installed_plugin_errors: &'a [InstalledPluginErrorRecord],
    prompt_definitions: &'a BTreeMap<String, PromptDefinition>,
    record_count: u64,
    prompt_requests: &'a Map<PromptId, PromptInstance>,
    prompt_responses: &'a Map<PromptId, PromptResponse>,
    closed_prompts: &'a Map<PromptId, Timestamp>,
    archived_cycles: &'a BTreeSet<TestCycleId>,
    archived_executions: &'a BTreeSet<TestExecutionId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Decoded {
    catalog: catalog::DecodedPool,
    session_id: SessionId,
    record_namespace: SessionId,
    identity: DokimeIdentity,
    run_parameters: BTreeMap<String, MaterializedParameter>,
    shutting_down: bool,
    pending_effects: Vec<dokime_domain::model::execution::effects::PendingEffect>,
    installed_plugins: BTreeMap<String, catalog::Plugin>,
    plugin_owners: BTreeMap<String, PluginOwner>,
    suites: catalog::DecodedRegistry,
    record_definitions: BTreeMap<Identifier, u32>,
    record_history: BTreeMap<Identifier, BTreeMap<u32, u32>>,
    telemetry_definitions: BTreeMap<Identifier, u32>,
    cycles: Map<TestCycleId, dokime_domain::model::execution::result::TestCycle>,

    #[serde(deserialize_with = "executions::deserialize")]
    executions: Map<TestExecutionId, dokime_domain::model::execution::result::TestExecution>,

    retired_executions: BTreeSet<TestExecutionId>,

    #[serde(default)]
    run_requests: Map<CommandId, crate::transactions::provider::request::RunRequest>,

    run_queue: std::collections::VecDeque<CommandId>,
    cycle_report_finalizations: Map<TestCycleId, crate::transactions::CycleReportFinalization>,
    report_finalizations: Map<TestExecutionId, crate::transactions::ReportFinalization>,
    note_admissions: Map<NoteId, dokime_domain::model::execution::notes::NoteAdmission>,
    records: EvidenceRecords,
    resources: ResourceRegistry,
    installed_plugin_errors: Vec<InstalledPluginErrorRecord>,
    prompt_definitions: BTreeMap<String, PromptDefinition>,
    record_count: u64,
    prompt_requests: Map<PromptId, PromptInstance>,
    prompt_responses: Map<PromptId, PromptResponse>,
    closed_prompts: Map<PromptId, Timestamp>,
    archived_cycles: BTreeSet<TestCycleId>,
    archived_executions: BTreeSet<TestExecutionId>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serialize for RuntimeState {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let RuntimeState {
            session_id,
            record_namespace,
            identity,
            run_parameters,
            shutting_down,
            pending_effects,
            installed_plugins,
            plugin_owners,
            suites,
            record_definitions,
            record_history,
            telemetry_definitions,
            cycles,
            executions,
            retired_executions,
            run_requests,
            run_queue,
            cycle_report_finalizations,
            report_finalizations,
            note_admissions,
            records,
            resources,
            installed_plugin_errors,
            prompt_definitions,
            record_count,
            prompt_requests,
            prompt_responses,
            closed_prompts,
            archived_cycles,
            archived_executions,
        } = self;

        // Collect The Shared Catalog Before Encoding Its Owners
        let mut pool = catalog::Pool::default();
        let installed_plugins = pool.plugins(installed_plugins);
        let suites = pool.registry(suites);
        let record_definitions = pool.records(record_definitions);
        let record_history = pool.history(record_history.contracts());
        let telemetry_definitions = pool.telemetry(telemetry_definitions);

        Encoded {
            catalog: &pool,
            session_id,
            record_namespace,
            identity,
            run_parameters,
            shutting_down: *shutting_down,
            pending_effects,
            installed_plugins,
            plugin_owners,
            suites,
            record_definitions,
            record_history,
            telemetry_definitions,
            cycles,
            executions,
            retired_executions,
            run_requests,
            run_queue,
            cycle_report_finalizations,
            report_finalizations,
            note_admissions,
            records,
            resources,
            installed_plugin_errors,
            prompt_definitions,
            record_count: *record_count,
            prompt_requests,
            prompt_responses,
            closed_prompts,
            archived_cycles,
            archived_executions,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RuntimeState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let decoded = Decoded::deserialize(deserializer)?;

        let state = decoded.restore().map_err(serde::de::Error::custom)?;
        state.validate_execution_ownership().map_err(serde::de::Error::custom)?;
        Ok(state)
    }
}

impl Decoded {
    fn restore(self) -> Result<RuntimeState, String> {
        let Decoded {
            catalog,
            session_id,
            record_namespace,
            identity,
            run_parameters,
            shutting_down,
            pending_effects,
            installed_plugins,
            plugin_owners,
            suites,
            record_definitions,
            record_history,
            telemetry_definitions,
            cycles,
            executions,
            retired_executions,
            run_requests,
            run_queue,
            cycle_report_finalizations,
            report_finalizations,
            note_admissions,
            records,
            resources,
            installed_plugin_errors,
            prompt_definitions,
            record_count,
            prompt_requests,
            prompt_responses,
            closed_prompts,
            archived_cycles,
            archived_executions,
        } = self;
        let record_history = record_history::RecordHistory::from_contracts(catalog.history(record_history)?);

        Ok(RuntimeState {
            session_id,
            record_namespace,
            identity,
            run_parameters: run_parameters.into(),
            shutting_down,
            pending_effects: pending_effects.into(),
            installed_plugins: catalog.plugins(installed_plugins)?.into(),
            plugin_owners: plugin_owners.into(),
            suites: catalog.registry(suites)?.into(),
            record_definitions: catalog.records(record_definitions)?.into(),
            record_history: record_history.into(),
            telemetry_definitions: catalog.telemetry(telemetry_definitions)?.into(),
            cycles,
            executions,
            retired_executions: retired_executions.into(),
            run_requests,
            run_queue: run_queue.into(),
            cycle_report_finalizations,
            report_finalizations,
            note_admissions,
            records: records.into(),
            resources: resources.into(),
            installed_plugin_errors: installed_plugin_errors.into(),
            prompt_definitions: prompt_definitions.into(),
            record_count,
            prompt_requests,
            prompt_responses,
            closed_prompts,
            archived_cycles: archived_cycles.into(),
            archived_executions: archived_executions.into(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

// Returns The Position Of A Shared Value, Storing It On First Sight
fn intern<'a, T: PartialEq + ?Sized>(values: &mut Vec<&'a T>, value: &'a T) -> u32 {
    let index = match values.iter().position(|candidate| *candidate == value) {
        Some(index) => index,
        None => {
            values.push(value);
            values.len() - 1
        }
    };
    u32::try_from(index).expect("shared value count fits u32")
}

fn position<T: PartialEq + ?Sized>(values: &[&T], value: &T) -> u32 {
    let index = values.iter().position(|candidate| *candidate == value).expect("shared values were collected from this state");
    u32::try_from(index).expect("shared value count fits u32")
}

fn resolve<T: Clone>(values: &[T], index: u32, kind: &str) -> Result<T, String> {
    values.get(index as usize).cloned().ok_or_else(|| format!("state references a missing shared {kind}"))
}

/* === Tests === */
