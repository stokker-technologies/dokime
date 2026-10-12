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
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Deserializer};

// Local Imports
use super::{
    DokimeIdentity, EvidenceRecords, InstalledPlugin, InstalledPluginErrorRecord,
    MaterializedParameter, PluginOwner, PromptDefinition, PromptId, PromptInstance, PromptResponse,
    RecordContract, ResourceRegistry, RuntimeState, SessionId, SuiteRegistry, TelemetryContract,
    TestCycleId, Timestamp, record_history,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Remote derive constructs RuntimeState directly, with no second state owner or
// JSON value conversion. Adding a field to RuntimeState also requires updating
// this decoder: the generated struct construction is checked by the compiler.
#[derive(Deserialize)]
#[serde(remote = "RuntimeState", deny_unknown_fields)]
struct DecodedState {
    session_id: SessionId,
    record_namespace: SessionId,
    identity: DokimeIdentity,
    run_parameters: BTreeMap<String, MaterializedParameter>,
    shutting_down: bool,
    installed_plugins: BTreeMap<String, InstalledPlugin>,
    plugin_owners: BTreeMap<String, PluginOwner>,
    suites: SuiteRegistry,
    record_definitions: BTreeMap<dokime::core::Identifier, RecordContract>,
    record_history: record_history::RecordHistory,
    telemetry_definitions: BTreeMap<dokime::core::Identifier, TelemetryContract>,
    cycles: BTreeMap<TestCycleId, crate::execution::TestCycle>,
    executions: BTreeMap<dokime::core::TestExecutionId, crate::execution::TestExecution>,
    #[serde(default)]
    run_requests:
        BTreeMap<dokime::core::CommandId, crate::transactions::provider::request::RunRequest>,
    note_admissions: BTreeMap<dokime::core::NoteId, crate::execution::notes::NoteAdmission>,
    records: EvidenceRecords,
    resources: ResourceRegistry,
    installed_plugin_errors: Vec<InstalledPluginErrorRecord>,
    prompt_definitions: BTreeMap<String, PromptDefinition>,
    record_count: u64,
    prompt_requests: BTreeMap<PromptId, PromptInstance>,
    prompt_responses: BTreeMap<PromptId, PromptResponse>,
    closed_prompts: BTreeMap<PromptId, Timestamp>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<'de> Deserialize<'de> for RuntimeState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let state = DecodedState::deserialize(deserializer)?;

        state
            .validate_execution_ownership()
            .map_err(serde::de::Error::custom)?;

        Ok(state)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
