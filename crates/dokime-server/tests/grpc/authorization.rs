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
use std::future::Future;
use std::pin::Pin;
// Third Party Imports
use tonic::{Code, Status, transport::Channel};
// Local Imports
use super::*;
use dokime_protocol::v1;
use dokime_server::grpc::access::Role;

/* === Constants === */
const STREAMING_CASES: &[Case] = &[
    Case {
        path: "/dokime.v1.Content/UploadArtifact",
        call: |stream, token| {
            Box::pin(async move {
                v1::content_client::ContentClient::new(stream)
                    .upload_artifact(credentialed(
                        tokio_stream::iter(Vec::<v1::UploadArtifactFrame>::new()),
                        token,
                    ))
                    .await
                    .err()
            })
        },
    },
    Case {
        path: "/dokime.v1.Plugins/Stage",
        call: |stream, token| {
            Box::pin(async move {
                v1::plugins_client::PluginsClient::new(stream)
                    .stage(credentialed(
                        tokio_stream::iter(Vec::<v1::StagePluginFrame>::new()),
                        token,
                    ))
                    .await
                    .err()
            })
        },
    },
];

/* === Macros === */
macro_rules! matrix {
    ($($path:literal, $module:ident :: $client:ident, $method:ident, $request:ident;)*) => {
        const CASES: &[Case] = &[$(Case {
            path: $path,
            call: |stream, token| {
                Box::pin(async move {
                    v1::$module::$client::new(stream)
                        .$method(credentialed(v1::$request::default(), token))
                        .await
                        .err()
                })
            },
        },)*];
    };
}

matrix! {
    "/dokime.v1.Appearance/GetAppearance", appearance_client::AppearanceClient, get_appearance, GetAppearanceRequest;
    "/dokime.v1.Backend/CollectDiagnostics", backend_client::BackendClient, collect_diagnostics, CollectDiagnosticsRequest;
    "/dokime.v1.Backend/GetCapabilities", backend_client::BackendClient, get_capabilities, GetCapabilitiesRequest;
    "/dokime.v1.Backend/GetDiagnostics", backend_client::BackendClient, get_diagnostics, GetDiagnosticsRequest;
    "/dokime.v1.Backend/GetHealth", backend_client::BackendClient, get_health, GetHealthRequest;
    "/dokime.v1.Backend/GetIdentity", backend_client::BackendClient, get_identity, GetIdentityRequest;
    "/dokime.v1.Backend/WatchLogs", backend_client::BackendClient, watch_logs, WatchLogsRequest;
    "/dokime.v1.Catalog/GetRecordSchema", catalog_client::CatalogClient, get_record_schema, GetRecordSchemaRequest;
    "/dokime.v1.Catalog/GetSuite", catalog_client::CatalogClient, get_suite, GetSuiteRequest;
    "/dokime.v1.Catalog/GetTelemetrySchema", catalog_client::CatalogClient, get_telemetry_schema, GetTelemetrySchemaRequest;
    "/dokime.v1.Catalog/ListRecordSchemas", catalog_client::CatalogClient, list_record_schemas, ListRecordSchemasRequest;
    "/dokime.v1.Catalog/ListSuites", catalog_client::CatalogClient, list_suites, ListSuitesRequest;
    "/dokime.v1.Catalog/ListTelemetrySchemas", catalog_client::CatalogClient, list_telemetry_schemas, ListTelemetrySchemasRequest;
    "/dokime.v1.Content/DownloadAppearanceAsset", content_client::ContentClient, download_appearance_asset, DownloadAppearanceAssetRequest;
    "/dokime.v1.Content/DownloadArtifact", content_client::ContentClient, download_artifact, DownloadArtifactRequest;
    "/dokime.v1.Content/DownloadCycleBundle", content_client::ContentClient, download_cycle_bundle, DownloadCycleBundleRequest;
    "/dokime.v1.Content/DownloadExecutionFile", content_client::ContentClient, download_execution_file, DownloadExecutionFileRequest;
    "/dokime.v1.Content/GetCycleFolderPath", content_client::ContentClient, get_cycle_folder_path, GetCycleFolderPathRequest;
    "/dokime.v1.Content/ListArtifacts", content_client::ContentClient, list_artifacts, ListArtifactsRequest;
    "/dokime.v1.Content/ListExecutionFiles", content_client::ContentClient, list_execution_files, ListExecutionFilesRequest;
    "/dokime.v1.Content/ReadExecutionFile", content_client::ContentClient, read_execution_file, ReadExecutionFileRequest;
    "/dokime.v1.Documents/CaptureCurrentDocument", documents_client::DocumentsClient, capture_current_document, CaptureCurrentDocumentRequest;
    "/dokime.v1.Evidence/GetCapturedRecordSchema", evidence_client::EvidenceClient, get_captured_record_schema, GetCapturedRecordSchemaRequest;
    "/dokime.v1.Evidence/ListRecords", evidence_client::EvidenceClient, list_records, ListRecordsRequest;
    "/dokime.v1.Evidence/WatchRecords", evidence_client::EvidenceClient, watch_records, WatchRecordsRequest;
    "/dokime.v1.Execution/AppendPlannedCases", execution_client::ExecutionClient, append_planned_cases, AppendPlannedCasesRequest;
    "/dokime.v1.Execution/ClearCycle", execution_client::ExecutionClient, clear_cycle, CycleControlRequest;
    "/dokime.v1.Execution/CopyCycle", execution_client::ExecutionClient, copy_cycle, CopyCycleRequest;
    "/dokime.v1.Execution/CreateCycle", execution_client::ExecutionClient, create_cycle, CreateCycleRequest;
    "/dokime.v1.Execution/DeleteCase", execution_client::ExecutionClient, delete_case, CaseControlRequest;
    "/dokime.v1.Execution/GetCase", execution_client::ExecutionClient, get_case, GetCaseRequest;
    "/dokime.v1.Execution/GetCycle", execution_client::ExecutionClient, get_cycle, GetCycleRequest;
    "/dokime.v1.Execution/GetExecution", execution_client::ExecutionClient, get_execution, GetExecutionRequest;
    "/dokime.v1.Execution/GetExecutionCommand", execution_client::ExecutionClient, get_execution_command, GetExecutionCommandRequest;
    "/dokime.v1.Execution/GetRunRequest", execution_client::ExecutionClient, get_run_request, GetRunRequestRequest;
    "/dokime.v1.Execution/ListCaseParameters", execution_client::ExecutionClient, list_case_parameters, ListCaseParametersRequest;
    "/dokime.v1.Execution/ListCases", execution_client::ExecutionClient, list_cases, ListCasesRequest;
    "/dokime.v1.Execution/ListCycles", execution_client::ExecutionClient, list_cycles, ListCyclesRequest;
    "/dokime.v1.Execution/ListExecutionParameters", execution_client::ExecutionClient, list_execution_parameters, ListExecutionParametersRequest;
    "/dokime.v1.Execution/ListExecutionSteps", execution_client::ExecutionClient, list_execution_steps, ListExecutionStepsRequest;
    "/dokime.v1.Execution/ListExecutions", execution_client::ExecutionClient, list_executions, ListExecutionsRequest;
    "/dokime.v1.Execution/ListStepCriteria", execution_client::ExecutionClient, list_step_criteria, ListStepEvidenceRequest;
    "/dokime.v1.Execution/PauseCycle", execution_client::ExecutionClient, pause_cycle, CycleControlRequest;
    "/dokime.v1.Execution/ResumeCycle", execution_client::ExecutionClient, resume_cycle, CycleControlRequest;
    "/dokime.v1.Execution/RunCase", execution_client::ExecutionClient, run_case, RunCaseRequest;
    "/dokime.v1.Execution/RunExecution", execution_client::ExecutionClient, run_execution, RunExecutionRequest;
    "/dokime.v1.Execution/PauseExecution", execution_client::ExecutionClient, pause_execution, ExecutionControlRequest;
    "/dokime.v1.Execution/ResumeExecution", execution_client::ExecutionClient, resume_execution, ExecutionControlRequest;
    "/dokime.v1.Execution/StopExecution", execution_client::ExecutionClient, stop_execution, ExecutionControlRequest;
    "/dokime.v1.Execution/SubmitExecutionCommand", execution_client::ExecutionClient, submit_execution_command, SubmitExecutionCommandRequest;
    "/dokime.v1.Execution/RunCycle", execution_client::ExecutionClient, run_cycle, RunCycleRequest;
    "/dokime.v1.Execution/ScheduleCaseRerun", execution_client::ExecutionClient, schedule_case_rerun, CaseControlRequest;
    "/dokime.v1.Execution/StopCycle", execution_client::ExecutionClient, stop_cycle, CycleControlRequest;
    "/dokime.v1.Execution/UpdateCyclePolicy", execution_client::ExecutionClient, update_cycle_policy, UpdateCyclePolicyRequest;
    "/dokime.v1.Health/ListHealth", health_client::HealthClient, list_health, ListAnnotationsRequest;
    "/dokime.v1.History/BrowseExecutions", history_client::HistoryClient, browse_executions, BrowseExecutionsRequest;
    "/dokime.v1.History/DeleteExecution", history_client::HistoryClient, delete_execution, DeleteExecutionRequest;
    "/dokime.v1.History/DownloadCheckpointArtifact", history_client::HistoryClient, download_checkpoint_artifact, DownloadCheckpointArtifactRequest;
    "/dokime.v1.History/GetCheckpointContext", history_client::HistoryClient, get_checkpoint_context, GetCheckpointContextRequest;
    "/dokime.v1.History/GetCheckpointDocument", history_client::HistoryClient, get_checkpoint_document, GetCheckpointDocumentRequest;
    "/dokime.v1.History/GetCheckpointEvidence", history_client::HistoryClient, get_checkpoint_evidence, GetCheckpointEvidenceRequest;
    "/dokime.v1.History/GetExecutionCheckpoint", history_client::HistoryClient, get_execution_checkpoint, GetExecutionCheckpointRequest;
    "/dokime.v1.History/ListCheckpointRecords", history_client::HistoryClient, list_checkpoint_records, ListCheckpointRecordsRequest;
    "/dokime.v1.History/ListCheckpointTelemetry", history_client::HistoryClient, list_checkpoint_telemetry, ListCheckpointTelemetryRequest;
    "/dokime.v1.History/ReadCheckpointOutput", history_client::HistoryClient, read_checkpoint_output, ReadCheckpointOutputRequest;
    "/dokime.v1.Notes/CreateNote", notes_client::NotesClient, create_note, CreateNoteRequest;
    "/dokime.v1.Notes/ListNotes", notes_client::NotesClient, list_notes, ListAnnotationsRequest;
    "/dokime.v1.Observation/GetRuntimeShell", observation_client::ObservationClient, get_runtime_shell, GetRuntimeShellRequest;
    "/dokime.v1.Observation/GetTraceability", observation_client::ObservationClient, get_traceability, GetTraceabilityRequest;
    "/dokime.v1.Observation/WatchExecutionProgress", observation_client::ObservationClient, watch_execution_progress, WatchExecutionProgressRequest;
    "/dokime.v1.Observation/WatchRuntime", observation_client::ObservationClient, watch_runtime, WatchRuntimeRequest;
    "/dokime.v1.Planning/PreviewSelection", planning_client::PlanningClient, preview_selection, PreviewSelectionRequest;
    "/dokime.v1.Planning/RegisterSelection", planning_client::PlanningClient, register_selection, RegisterSelectionRequest;
    "/dokime.v1.Plots/ListAnalyses", plots_client::PlotsClient, list_analyses, ListAnalysesRequest;
    "/dokime.v1.Plots/QuerySnapshots", plots_client::PlotsClient, query_snapshots, PlotSnapshotsRequest;
    "/dokime.v1.Plots/Watch", plots_client::PlotsClient, watch, PlotWatchRequest;
    "/dokime.v1.Plugins/Consent", plugins_client::PluginsClient, consent, ConsentPluginRequest;
    "/dokime.v1.Plugins/GetInstallation", plugins_client::PluginsClient, get_installation, GetInstallationRequest;
    "/dokime.v1.Plugins/ListAdmittedPlugins", plugins_client::PluginsClient, list_admitted_plugins, ListAdmittedPluginsRequest;
    "/dokime.v1.Plugins/ListInstallations", plugins_client::PluginsClient, list_installations, ListInstallationsRequest;
    "/dokime.v1.Plugins/Remove", plugins_client::PluginsClient, remove, RemovePluginRequest;
    "/dokime.v1.Plugins/SetEnabled", plugins_client::PluginsClient, set_enabled, SetPluginEnabledRequest;
    "/dokime.v1.Prompts/GetDefinition", prompts_client::PromptsClient, get_definition, GetPromptDefinitionRequest;
    "/dokime.v1.Prompts/GetPrompt", prompts_client::PromptsClient, get_prompt, GetPromptRequest;
    "/dokime.v1.Prompts/ListDefinitions", prompts_client::PromptsClient, list_definitions, ListPromptDefinitionsRequest;
    "/dokime.v1.Prompts/ListPending", prompts_client::PromptsClient, list_pending, ListPendingPromptsRequest;
    "/dokime.v1.Prompts/Respond", prompts_client::PromptsClient, respond, RespondToPromptRequest;
    "/dokime.v1.Query/Aggregate", query_client::QueryClient, aggregate, AggregateRequest;
    "/dokime.v1.Reports/DownloadCycleReport", reports_client::ReportsClient, download_cycle_report, DownloadCycleReportRequest;
    "/dokime.v1.Reports/RenderCheckpointReport", reports_client::ReportsClient, render_checkpoint_report, RenderCheckpointReportRequest;
    "/dokime.v1.Settings/GetPluginSettings", settings_client::SettingsClient, get_plugin_settings, GetPluginSettingsRequest;
    "/dokime.v1.Settings/GetSettings", settings_client::SettingsClient, get_settings, GetSettingsRequest;
    "/dokime.v1.Settings/ReadApplicationSettings", settings_client::SettingsClient, read_application_settings, ReadApplicationSettingsRequest;
    "/dokime.v1.Settings/ReadStoragePolicy", settings_client::SettingsClient, read_storage_policy, ReadStoragePolicyRequest;
    "/dokime.v1.Settings/UpdateStoragePolicy", settings_client::SettingsClient, update_storage_policy, UpdateStoragePolicyRequest;
    "/dokime.v1.Settings/ListStorageSources", settings_client::SettingsClient, list_storage_sources, ListStorageSourcesRequest;
    "/dokime.v1.Settings/ReorderStorageSources", settings_client::SettingsClient, reorder_storage_sources, ReorderStorageSourcesRequest;
    "/dokime.v1.Settings/SetStorageSourceEnabled", settings_client::SettingsClient, set_storage_source_enabled, SetStorageSourceEnabledRequest;
    "/dokime.v1.Settings/UpdateAppearance", settings_client::SettingsClient, update_appearance, UpdateAppearanceRequest;
    "/dokime.v1.Settings/UpdateDataRoot", settings_client::SettingsClient, update_data_root, UpdateDataRootRequest;
    "/dokime.v1.Settings/UpdatePluginCredential", settings_client::SettingsClient, update_plugin_credential, UpdatePluginCredentialRequest;
    "/dokime.v1.Settings/UpdatePluginSettings", settings_client::SettingsClient, update_plugin_settings, UpdatePluginSettingsRequest;
    "/dokime.v1.Telemetry/ListSamples", telemetry_client::TelemetryClient, list_samples, ListTelemetrySamplesRequest;
}

/* === Type Aliases === */
type Refusal = Pin<Box<dyn Future<Output = Option<Status>> + Send>>;

/* === Structs === */
struct Case {
    path: &'static str,
    call: fn(Channel, Option<&'static str>) -> Refusal,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn credentialed<T>(message: T, token: Option<&'static str>) -> Request<T> {
    match token {
        Some(token) => auth(message, token),
        None => Request::new(message),
    }
}

fn cases() -> impl Iterator<Item = &'static Case> {
    use dokime::operations::runtime::{GetHealth, GetIdentity};
    static OPERATIONS: &[Case] = &[
        Case {
            path: GetHealth::PATH,
            call: |channel, token| operation::<GetHealth>(channel, token, GetHealth::PATH),
        },
        Case {
            path: GetIdentity::PATH,
            call: |channel, token| operation::<GetIdentity>(channel, token, GetIdentity::PATH),
        },
    ];
    CASES.iter().chain(STREAMING_CASES).chain(OPERATIONS)
}

fn operation<O>(channel: Channel, token: Option<&'static str>, path: &'static str) -> Refusal
where
    O: harmos::operations::OperationDeclaration<Request = dokime::operations::runtime::Request>,
    O::Response: harmos::Protobuf + harmos::MemorySize,
{
    Box::pin(async move {
        use harmos::grpc::{Codec, DecodeSettings, EncodeSettings};
        let bytes = std::num::NonZeroUsize::new(4096).unwrap();
        let codec =
            Codec::<O::Response, std::sync::Arc<dokime::operations::runtime::Request>>::new(
                DecodeSettings {
                    message_bytes: bytes,
                    retained_bytes: bytes,
                },
                EncodeSettings {
                    message_bytes: bytes,
                    buffer_bytes: bytes,
                },
            )
            .unwrap();
        let mut client = tonic::client::Grpc::new(channel);
        client.ready().await.unwrap();
        client
            .unary(
                credentialed(
                    std::sync::Arc::new(dokime::operations::runtime::Request {}),
                    token,
                ),
                tonic::codegen::http::uri::PathAndQuery::from_static(path),
                codec,
            )
            .await
            .err()
    })
}

pub(crate) fn table_covers_serves() {
    let served: std::collections::BTreeSet<_> = dokime_server::grpc::methods()
        .map(|(path, _)| path)
        .collect();
    let covered: std::collections::BTreeSet<_> = cases().map(|case| case.path).collect();
    assert_eq!(
        served, covered,
        "every served method needs a row here, and a row needs a served method"
    );
    assert_eq!(covered.len(), cases().count(), "no duplicate rows");
}

pub(crate) async fn permission_matrix_enforcement() {
    const PRINCIPALS: &[(Role, &str)] = &[
        (Role::Read, "read-token-012345678901234567890123456789"),
        (
            Role::Control,
            "control-token-012345678901234567890123456789",
        ),
        (Role::Admin, "admin-token-012345678901234567890123456789"),
    ];
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let grants = PRINCIPALS
        .iter()
        .map(|(role, token)| BearerGrant::new(format!("{role:?}"), *role, token).unwrap())
        .collect();
    let server = Server::start(
        owner.handle(),
        Config::local().with_bearer_auth(grants).unwrap(),
    )
    .await
    .unwrap();
    let channel = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let roles: std::collections::BTreeMap<_, _> = dokime_server::grpc::methods().collect();

    // Every Method Requires Authentication Before Payload Decoding.
    for case in cases() {
        let refusal = (case.call)(channel.clone(), None)
            .await
            .unwrap_or_else(|| panic!("{} admitted a call with no credential", case.path));
        assert_eq!(refusal.code(), Code::Unauthenticated, "{}", case.path);
    }

    // Distinct Credentials Exercise Every Insufficient Role On One Listener.
    for &(role, token) in PRINCIPALS {
        let advertised: std::collections::BTreeSet<_> = BackendClient::new(channel.clone())
            .get_capabilities(credentialed(v1::GetCapabilitiesRequest {}, Some(token)))
            .await
            .unwrap()
            .into_inner()
            .methods
            .into_iter()
            .collect();
        let expected: std::collections::BTreeSet<_> = roles
            .iter()
            .filter(|(_, required)| role >= **required)
            .map(|(path, _)| (*path).to_owned())
            .collect();
        assert_eq!(advertised, expected, "{role:?} advertised permissions");

        for case in cases() {
            let required = roles[case.path];
            if role >= required {
                continue;
            }
            let refusal = (case.call)(channel.clone(), Some(token))
                .await
                .unwrap_or_else(|| panic!("{} admitted {role:?}, needs {required:?}", case.path));
            assert_eq!(
                refusal.code(),
                Code::PermissionDenied,
                "{} for {role:?}",
                case.path
            );
        }
    }

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
