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
use std::sync::LazyLock;

// Third Party Imports
use harmos::catalog::{Catalog, Projection};

// Local Imports
use super::access::Role;
use dokime::operations::{Endpoints, Operations};

/* === Constants === */
const LEGACY_METHODS: &[Method] = &[
    method("/dokime.v1.Appearance/GetAppearance", Role::Read),
    method("/dokime.v1.Backend/CollectDiagnostics", Role::Read).transfer(),
    method("/dokime.v1.Backend/GetCapabilities", Role::Read),
    method("/dokime.v1.Backend/GetDiagnostics", Role::Read),
    method("/dokime.v1.Backend/GetHealth", Role::Read),
    method("/dokime.v1.Backend/GetIdentity", Role::Read),
    // A follow stream, not a transfer: the handler returns before the first
    // record is read, so the ordinary budget bounds only that handshake.
    method("/dokime.v1.Backend/WatchLogs", Role::Read),
    method("/dokime.v1.Catalog/GetRecordSchema", Role::Read),
    method("/dokime.v1.Catalog/GetSuite", Role::Read),
    method("/dokime.v1.Catalog/GetTelemetrySchema", Role::Read),
    method("/dokime.v1.Catalog/ListRecordSchemas", Role::Read),
    method("/dokime.v1.Catalog/ListSuites", Role::Read),
    method("/dokime.v1.Catalog/ListTelemetrySchemas", Role::Read),
    method("/dokime.v1.Content/DownloadAppearanceAsset", Role::Read),
    method("/dokime.v1.Content/DownloadArtifact", Role::Read),
    method("/dokime.v1.Content/DownloadCycleBundle", Role::Read).transfer(),
    method("/dokime.v1.Content/DownloadExecutionFile", Role::Read).transfer(),
    method("/dokime.v1.Content/GetCycleFolderPath", Role::Read).local_only(),
    method("/dokime.v1.Content/ListArtifacts", Role::Read),
    method("/dokime.v1.Content/ListExecutionFiles", Role::Read),
    method("/dokime.v1.Content/ReadExecutionFile", Role::Read),
    method("/dokime.v1.Content/UploadArtifact", Role::Control).transfer(),
    method("/dokime.v1.Documents/CaptureCurrentDocument", Role::Read),
    method("/dokime.v1.Evidence/GetCapturedRecordSchema", Role::Read),
    method("/dokime.v1.Evidence/ListRecords", Role::Read),
    method("/dokime.v1.Evidence/WatchRecords", Role::Read),
    method("/dokime.v1.Execution/AppendPlannedCases", Role::Control),
    method("/dokime.v1.Execution/ClearCycle", Role::Control),
    method("/dokime.v1.Execution/CopyCycle", Role::Control),
    method("/dokime.v1.Execution/CreateCycle", Role::Control),
    method("/dokime.v1.Execution/DeleteCase", Role::Control),
    method("/dokime.v1.Execution/GetCase", Role::Read),
    method("/dokime.v1.Execution/GetCycle", Role::Read),
    method("/dokime.v1.Execution/GetExecution", Role::Read),
    method("/dokime.v1.Execution/GetExecutionCommand", Role::Read),
    method("/dokime.v1.Execution/GetRunRequest", Role::Read),
    method("/dokime.v1.Execution/ListCaseParameters", Role::Read),
    method("/dokime.v1.Execution/ListCases", Role::Read),
    method("/dokime.v1.Execution/ListCycles", Role::Read),
    method("/dokime.v1.Execution/ListExecutionParameters", Role::Read),
    method("/dokime.v1.Execution/ListExecutionSteps", Role::Read),
    method("/dokime.v1.Execution/ListExecutions", Role::Read),
    method("/dokime.v1.Execution/ListStepCriteria", Role::Read),
    method("/dokime.v1.Execution/PauseCycle", Role::Control),
    method("/dokime.v1.Execution/PauseExecution", Role::Control),
    method("/dokime.v1.Execution/ResumeCycle", Role::Control),
    method("/dokime.v1.Execution/ResumeExecution", Role::Control),
    method("/dokime.v1.Execution/RunCase", Role::Control),
    method("/dokime.v1.Execution/RunCycle", Role::Control),
    method("/dokime.v1.Execution/RunExecution", Role::Control),
    method("/dokime.v1.Execution/ScheduleCaseRerun", Role::Control),
    method("/dokime.v1.Execution/StopCycle", Role::Control),
    method("/dokime.v1.Execution/StopExecution", Role::Control),
    method("/dokime.v1.Execution/SubmitExecutionCommand", Role::Control),
    method("/dokime.v1.Execution/UpdateCyclePolicy", Role::Control),
    method("/dokime.v1.Health/ListHealth", Role::Read),
    method("/dokime.v1.History/BrowseExecutions", Role::Read),
    method("/dokime.v1.History/DeleteExecution", Role::Admin),
    method("/dokime.v1.History/DownloadCheckpointArtifact", Role::Read),
    method("/dokime.v1.History/GetCheckpointContext", Role::Read),
    method("/dokime.v1.History/GetCheckpointDocument", Role::Read),
    method("/dokime.v1.History/GetCheckpointEvidence", Role::Read),
    method("/dokime.v1.History/GetExecutionCheckpoint", Role::Read),
    method("/dokime.v1.History/ListCheckpointRecords", Role::Read),
    method("/dokime.v1.History/ListCheckpointTelemetry", Role::Read),
    method("/dokime.v1.History/ReadCheckpointOutput", Role::Read),
    method("/dokime.v1.Notes/CreateNote", Role::Control),
    method("/dokime.v1.Notes/ListNotes", Role::Read),
    method("/dokime.v1.Observation/GetRuntimeShell", Role::Read),
    method("/dokime.v1.Observation/GetTraceability", Role::Read),
    method("/dokime.v1.Observation/WatchExecutionProgress", Role::Read),
    method("/dokime.v1.Observation/WatchRuntime", Role::Read),
    method("/dokime.v1.Planning/PreviewSelection", Role::Read),
    method("/dokime.v1.Planning/RegisterSelection", Role::Control),
    method("/dokime.v1.Plots/ListAnalyses", Role::Read),
    method("/dokime.v1.Plots/QuerySnapshots", Role::Read),
    method("/dokime.v1.Plots/Watch", Role::Read),
    method("/dokime.v1.Plugins/Consent", Role::Admin),
    method("/dokime.v1.Plugins/GetInstallation", Role::Admin),
    method("/dokime.v1.Plugins/ListAdmittedPlugins", Role::Read),
    method("/dokime.v1.Plugins/ListInstallations", Role::Admin),
    method("/dokime.v1.Plugins/Remove", Role::Admin),
    method("/dokime.v1.Plugins/SetEnabled", Role::Admin),
    method("/dokime.v1.Plugins/Stage", Role::Admin).transfer(),
    method("/dokime.v1.Prompts/GetDefinition", Role::Read),
    method("/dokime.v1.Prompts/GetPrompt", Role::Read),
    method("/dokime.v1.Prompts/ListDefinitions", Role::Read),
    method("/dokime.v1.Prompts/ListPending", Role::Read),
    method("/dokime.v1.Prompts/Respond", Role::Control),
    method("/dokime.v1.Query/Aggregate", Role::Read),
    method("/dokime.v1.Reports/DownloadCycleReport", Role::Read).transfer(),
    method("/dokime.v1.Reports/RenderCheckpointReport", Role::Read).transfer(),
    method("/dokime.v1.Settings/GetPluginSettings", Role::Read),
    method("/dokime.v1.Settings/GetSettings", Role::Read),
    method("/dokime.v1.Settings/ListStorageSources", Role::Read),
    method("/dokime.v1.Settings/ReadApplicationSettings", Role::Read),
    method("/dokime.v1.Settings/ReadStoragePolicy", Role::Read),
    method("/dokime.v1.Settings/ReorderStorageSources", Role::Admin),
    method("/dokime.v1.Settings/SetStorageSourceEnabled", Role::Admin),
    method("/dokime.v1.Settings/UpdateAppearance", Role::Control),
    method("/dokime.v1.Settings/UpdateDataRoot", Role::Admin),
    method("/dokime.v1.Settings/UpdatePluginCredential", Role::Admin),
    method("/dokime.v1.Settings/UpdatePluginSettings", Role::Admin),
    method("/dokime.v1.Settings/UpdateStoragePolicy", Role::Admin),
    method("/dokime.v1.Telemetry/ListSamples", Role::Read),
];

pub(super) static METHODS: LazyLock<Vec<Method>> = LazyLock::new(|| {
    let mut methods = LEGACY_METHODS.to_vec();
    let added = <Operations as Catalog<Endpoints>>::ENTRIES.try_for_each(|entry| {
        methods.push(method(entry.path, entry.access));
        Ok::<_, std::convert::Infallible>(())
    });
    match added {
        Ok(()) => {}
        Err(impossible) => match impossible {},
    }
    methods.sort_unstable_by_key(|entry| entry.path);
    assert!(
        methods.windows(2).all(|pair| pair[0].path != pair[1].path),
        "duplicate public method identity"
    );
    methods
});

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy)]
pub(super) struct Method {
    pub(super) path: &'static str,
    pub(super) role: Role,
    pub(super) local_only: bool,
    pub(super) work: Work,
}

/* === Enums === */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Work {
    Ordinary,
    Transfer,
}

/* === Traits === */

/* === Implementations === */
impl Method {
    const fn local_only(mut self) -> Self {
        self.local_only = true;
        self
    }

    const fn transfer(mut self) -> Self {
        self.work = Work::Transfer;
        self
    }
}

/* === Public Functions === */
pub(super) fn find(path: &str) -> Option<&'static Method> {
    METHODS
        .binary_search_by(|method| method.path.cmp(path))
        .ok()
        .map(|index| &METHODS[index])
}

/* === Private Functions === */
const fn method(path: &'static str, role: Role) -> Method {
    Method {
        path,
        role,
        local_only: false,
        work: Work::Ordinary,
    }
}

/* === Tests === */
