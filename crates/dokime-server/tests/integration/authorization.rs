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
use dokime_domain::model;
use dokime_server::grpc::access::Role;

/* === Constants === */
const STREAMING_CASES: &[Case] = &[
    Case {
        path: "/dokime.v1.Content/UploadArtifact",
        call: |stream, token| {
            Box::pin(async move {
                WireClient::new(stream)
                    .upload::<contracts::content::UploadArtifact>(credentialed(
                        tokio_stream::iter(Vec::<model::content::wire::UploadArtifactFrame>::new()),
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
                WireClient::new(stream)
                    .upload::<contracts::plugins::Stage>(credentialed(
                        tokio_stream::iter(Vec::<model::plugins::wire::StagePluginFrame>::new()),
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
    (@request) => {
        Default::default()
    };
    (@request $request:expr) => {
        $request
    };
    ($($path:literal, $method:ident, $operation:path $(= $request:expr)?;)*) => {
        const CASES: &[Case] = &[$(Case {
            path: $path,
            call: |stream, token| {
                Box::pin(async move {
                    let request: $operation =
                        matrix!(@request $($request)?);
                    WireClient::new(stream)
                        .$method::<$operation>(credentialed(request, token))
                        .await
                        .err()
                })
            },
        },)*];
    };
}

matrix! {
    "/dokime.v1.Reports/MaterializeReport", unary, contracts::reports::MaterializeReport = contracts::reports::MaterializeReport {
        subject: model::reports::wire::ReportSubject::Execution(String::new()),
        at: harmos::storage::StorageCursor {
            history: harmos::storage::StoreHistory {
                store: harmos::Identifier::new("authorization").unwrap(),
                epoch: harmos::Identifier::new("test").unwrap(),
            },
            sequence: 0,
        },
    };
    "/harmos.host.v1.Host/Catalog", unary, harmos::operations::host::Catalog = harmos::operations::host::Catalog {
        kind: harmos::operations::host::CatalogKind::Operations,
        schema: None,
        after: None,
        rows: None,
        bytes: None,
    };
    "/dokime.v1.Content/FollowExecutionFile", download, contracts::content::FollowExecutionFile;
    "/dokime.v1.Appearance/GetAppearance", unary, contracts::appearance::GetAppearance;
    "/dokime.v1.Backend/CollectDiagnostics", download, contracts::backend::CollectDiagnostics;
    "/dokime.v1.Backend/GetAvailability", unary, contracts::backend::GetAvailability;
    "/dokime.v1.Backend/GetDiagnostics", unary, contracts::backend::GetDiagnostics;
    "/dokime.v1.Backend/GetHealth", unary, contracts::backend::GetHealth;
    "/dokime.v1.Backend/GetIdentity", unary, contracts::backend::GetIdentity;
    "/dokime.v1.Backend/WatchLogs", download, contracts::backend::WatchLogs = contracts::backend::WatchLogs {
        min_level: model::backend::wire::LogLevel::Unspecified,
        replay_lines: 0,
    };
    "/dokime.v1.Backend/WatchTrace", download, contracts::backend::WatchTrace = contracts::backend::WatchTrace {
        epoch: None,
        scope: None,
        min_level: model::backend::wire::LogLevel::Unspecified,
        after_sequence: 0,
        max_events: 0,
        max_bytes: 0,
        follow: false,
    };
    "/dokime.v1.Catalog/GetRecordSchema", unary, contracts::catalog::GetRecordSchema;
    "/dokime.v1.Catalog/GetSuite", unary, contracts::catalog::GetSuite;
    "/dokime.v1.Catalog/GetTelemetrySchema", unary, contracts::catalog::GetTelemetrySchema;
    "/dokime.v1.Catalog/ListRecordSchemas", unary, contracts::catalog::ListRecordSchemas;
    "/dokime.v1.Catalog/ListSuites", unary, contracts::catalog::ListSuites;
    "/dokime.v1.Catalog/ListTelemetrySchemas", unary, contracts::catalog::ListTelemetrySchemas;
    "/dokime.v1.Content/DownloadAppearanceAsset", download, contracts::content::DownloadAppearanceAsset;
    "/dokime.v1.Content/DownloadArtifact", download, contracts::content::DownloadArtifact;
    "/dokime.v1.Content/DownloadExecutionFile", download, contracts::content::DownloadExecutionFile;
    "/dokime.v1.Content/ListArtifacts", unary, contracts::content::ListArtifacts;
    "/dokime.v1.Content/ListExecutionFiles", unary, contracts::content::ListExecutionFiles;
    "/dokime.v1.Content/ReadExecutionFile", unary, contracts::content::ReadExecutionFile;
    "/dokime.v1.Documents/CaptureCurrentDocument", download, contracts::documents::CaptureCurrentDocument;
    "/dokime.v1.Documents/UpdatePlotSettings", unary, contracts::documents::UpdatePlotSettings = contracts::documents::UpdatePlotSettings {
        scope: None,
        document_id: String::new(),
        plot_id: String::new(),
        display_points: None,
        downsample: Default::default(),
    };
    "/dokime.v1.Evidence/GetCapturedRecordSchema", unary, contracts::evidence::GetCapturedRecordSchema;
    "/dokime.v1.Evidence/ListRecords", unary, contracts::evidence::ListRecords;
    "/dokime.v1.Evidence/WatchRecords", download, contracts::evidence::WatchRecords;
    "/dokime.v1.Execution/AppendPlannedCases", unary, contracts::execution::AppendPlannedCases;
    "/dokime.v1.Execution/CopyCycle", unary, contracts::execution::CopyCycle;
    "/dokime.v1.Execution/CreateCycle", unary, contracts::execution::CreateCycle;
    "/dokime.v1.Execution/DeleteCase", unary, contracts::execution::DeleteCase;
    "/dokime.v1.Execution/GetCase", unary, contracts::execution::GetCase;
    "/dokime.v1.Execution/GetCycle", unary, contracts::execution::GetCycle;
    "/dokime.v1.Execution/GetExecution", unary, contracts::execution::GetExecution;
    "/dokime.v1.Execution/GetExecutionCommand", unary, contracts::execution::GetExecutionCommand;
    "/dokime.v1.Execution/GetRunRequest", unary, contracts::execution::GetRunRequest;
    "/dokime.v1.Execution/ListCaseParameters", unary, contracts::execution::ListCaseParameters;
    "/dokime.v1.Execution/ListCases", unary, contracts::execution::ListCases;
    "/dokime.v1.Execution/ListCycles", unary, contracts::execution::ListCycles;
    "/dokime.v1.Execution/ListExecutionParameters", unary, contracts::execution::ListExecutionParameters = contracts::execution::ListExecutionParameters {
        execution_id: String::new(),
        set: model::execution::wire::ExecutionParameterSet::Unspecified,
        limit: 0,
        after_name: String::new(),
        cursor: None,
    };
    "/dokime.v1.Execution/ListExecutionSteps", unary, contracts::execution::ListExecutionSteps;
    "/dokime.v1.Execution/ListExecutions", unary, contracts::execution::ListExecutions;
    "/dokime.v1.Execution/ListStepCriteria", unary, contracts::execution::ListStepCriteria;
    "/dokime.v1.Execution/PauseCycle", unary, contracts::execution::PauseCycle;
    "/dokime.v1.Execution/ResumeCycle", unary, contracts::execution::ResumeCycle;
    "/dokime.v1.Execution/RunCase", unary, contracts::execution::RunCase;
    "/dokime.v1.Execution/RunExecution", unary, contracts::execution::RunExecution = contracts::execution::RunExecution {
        execution_id: String::new(),
        context: None,
        idempotency_key: None,
        mode: model::execution::wire::ExecutionMode::Automatic,
    };
    "/dokime.v1.Execution/PauseExecution", unary, contracts::execution::PauseExecution;
    "/dokime.v1.Execution/ResumeExecution", unary, contracts::execution::ResumeExecution;
    "/dokime.v1.Execution/StopExecution", unary, contracts::execution::StopExecution;
    "/dokime.v1.Execution/SubmitExecutionCommand", unary, contracts::execution::SubmitExecutionCommand = contracts::execution::SubmitExecutionCommand {
        request_id: String::new(),
        command_id: String::new(),
        execution_id: String::new(),
        action: model::execution::wire::ExecutionCommandAction::Unspecified,
        step_id: None,
    };
    "/dokime.v1.Execution/RunCycle", unary, contracts::execution::RunCycle;
    "/dokime.v1.Execution/ScheduleCaseRerun", unary, contracts::execution::ScheduleCaseRerun;
    "/dokime.v1.Execution/RenameCycle", unary, contracts::execution::RenameCycle;
    "/dokime.v1.Execution/RenameExecution", unary, contracts::execution::RenameExecution;
    "/dokime.v1.Execution/SetCycleArchived", unary, contracts::execution::SetCycleArchived;
    "/dokime.v1.Execution/SetExecutionArchived", unary, contracts::execution::SetExecutionArchived;
    "/dokime.v1.Execution/StopCycle", unary, contracts::execution::StopCycle;
    "/dokime.v1.Execution/UpdateCyclePolicy", unary, contracts::execution::UpdateCyclePolicy;
    "/dokime.v1.Files/FileWatch", download, contracts::files::FileWatch = contracts::files::FileWatch {
        execution_id: None,
        source: None,
        rules: Vec::new(),
        output: String::new(),
        policy: model::files::wire::FileWatchPolicy::Unspecified,
        poll_interval_ms: None,
        maximum_seconds: None,
        maximum_files: None,
        maximum_bytes: None,
    };
    "/dokime.v1.Health/ListHealth", unary, contracts::notes::ListHealth;
    "/dokime.v1.History/BrowseExecutions", unary, contracts::history::BrowseExecutions;
    "/dokime.v1.History/DeleteExecution", unary, contracts::history::DeleteExecution;
    "/dokime.v1.History/DownloadCheckpointArtifact", download, contracts::history::DownloadCheckpointArtifact;
    "/dokime.v1.History/GetCheckpointContext", unary, contracts::history::GetCheckpointContext;
    "/dokime.v1.History/GetCheckpointDocument", unary, contracts::history::GetCheckpointDocument;
    "/dokime.v1.History/GetCheckpointEvidence", unary, contracts::history::GetCheckpointEvidence;
    "/dokime.v1.History/GetExecutionCheckpoint", unary, contracts::history::GetExecutionCheckpoint;
    "/dokime.v1.History/ListCheckpointRecords", unary, contracts::history::ListCheckpointRecords;
    "/dokime.v1.History/ListCheckpointTelemetry", unary, contracts::history::ListCheckpointTelemetry;
    "/dokime.v1.History/ReadCheckpointOutput", unary, contracts::history::ReadCheckpointOutput;
    "/dokime.v1.Media/AudioRecording", download, contracts::media::AudioRecording;
    "/dokime.v1.Media/CameraRecording", download, contracts::media::CameraRecording;
    "/dokime.v1.Media/Photo", unary, contracts::media::Photo;
    "/dokime.v1.Media/ScreenRecording", download, contracts::media::ScreenRecording;
    "/dokime.v1.Media/Screenshot", unary, contracts::media::Screenshot;
    "/dokime.v1.Notes/CreateNote", unary, contracts::notes::CreateNote;
    "/dokime.v1.Notes/EditNote", unary, contracts::notes::EditNote;
    "/dokime.v1.Notes/ListNotes", unary, contracts::notes::ListNotes;
    "/dokime.v1.Observation/GetRuntimeShell", unary, contracts::observation::GetRuntimeShell;
    "/dokime.v1.Observation/GetTraceability", unary, contracts::observation::GetTraceability;
    "/dokime.v1.Observation/WatchExecutionProgress", download, contracts::observation::WatchExecutionProgress;
    "/dokime.v1.Observation/WatchRuntime", download, contracts::observation::WatchRuntime;
    "/dokime.v1.Planning/PreviewSelection", unary, contracts::planning::PreviewSelection;
    "/dokime.v1.Planning/RegisterSelection", unary, contracts::planning::RegisterSelection;
    "/dokime.v1.Plots/ListAnalyses", unary, contracts::plots::ListAnalyses;
    "/dokime.v1.Plots/QuerySnapshots", unary, contracts::plots::QuerySnapshots;
    "/dokime.v1.Plots/Watch", download, contracts::plots::Watch;
    "/dokime.v1.Plugins/Consent", unary, contracts::plugins::Consent;
    "/dokime.v1.Plugins/GetInstallation", unary, contracts::plugins::GetInstallation;
    "/dokime.v1.Plugins/ListAdmittedPlugins", unary, contracts::plugins::ListAdmittedPlugins;
    "/dokime.v1.Plugins/ListInstallations", unary, contracts::plugins::ListInstallations;
    "/dokime.v1.Plugins/Remove", unary, contracts::plugins::Remove;
    "/dokime.v1.Plugins/SetEnabled", unary, contracts::plugins::SetEnabled;
    "/dokime.v1.Prompts/GetDefinition", unary, contracts::prompts::GetDefinition;
    "/dokime.v1.Prompts/GetPrompt", unary, contracts::prompts::GetPrompt;
    "/dokime.v1.Prompts/ListDefinitions", unary, contracts::prompts::ListDefinitions;
    "/dokime.v1.Prompts/ListPending", unary, contracts::prompts::ListPending;
    "/dokime.v1.Prompts/Respond", unary, contracts::prompts::Respond;
    "/dokime.v1.Query/Aggregate", unary, contracts::query::Aggregate = contracts::query::Aggregate {
        source: model::query::wire::QuerySource::Unspecified,
        filters: vec![],
        group_by: vec![],
        aggregates: vec![],
        limit: 0,
    };
    "/dokime.v1.Reports/DownloadCycleReport", download, contracts::reports::DownloadCycleReport;
    "/dokime.v1.Reports/DownloadExecutionReport", download, contracts::reports::DownloadExecutionReport;
    "/dokime.v1.Settings/GetPluginSettings", unary, contracts::settings::GetPluginSettings;
    "/dokime.v1.Settings/GetSettings", unary, contracts::settings::GetSettings;
    "/dokime.v1.Settings/ReadApplicationSettings", unary, contracts::settings::ReadApplicationSettings;
    "/dokime.v1.Settings/ReadStoragePolicy", unary, contracts::settings::ReadStoragePolicy;
    "/dokime.v1.Settings/UpdateStoragePolicy", unary, contracts::settings::UpdateStoragePolicy;
    "/dokime.v1.Settings/ListStorageSources", unary, contracts::settings::ListStorageSources;
    "/dokime.v1.Settings/ReorderStorageSources", unary, contracts::settings::ReorderStorageSources;
    "/dokime.v1.Settings/SetStorageSourceEnabled", unary, contracts::settings::SetStorageSourceEnabled;
    "/dokime.v1.Settings/UpdateAppearance", unary, contracts::settings::UpdateAppearance;
    "/dokime.v1.Settings/UpdateDataRoot", unary, contracts::settings::UpdateDataRoot;
    "/dokime.v1.Settings/UpdateReportDefaults", unary, contracts::settings::UpdateReportDefaults;
    "/dokime.v1.Settings/UpdatePluginCredential", unary, contracts::settings::UpdatePluginCredential;
    "/dokime.v1.Settings/UpdatePluginSettings", unary, contracts::settings::UpdatePluginSettings;
    "/dokime.v1.Telemetry/ListSamples", unary, contracts::telemetry::ListSamples;
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
    use dokime_runtime::operations::runtime::{GetHealth, GetIdentity};
    static OPERATIONS: &[Case] = &[
        Case { path: GetHealth::PATH, call: |channel, token| operation::<GetHealth>(channel, token, GetHealth::PATH) },
        Case { path: GetIdentity::PATH, call: |channel, token| operation::<GetIdentity>(channel, token, GetIdentity::PATH) },
    ];
    CASES.iter().chain(STREAMING_CASES).chain(OPERATIONS)
}

fn operation<O>(channel: Channel, token: Option<&'static str>, path: &'static str) -> Refusal
where
    O: harmos::operations::OperationDeclaration + harmos::domain::Portable + Default,
    O::Response: harmos::domain::Portable + harmos::MemorySize,
{
    Box::pin(async move {
        use harmos::grpc::{Codec, DecodeSettings, EncodeSettings};
        let bytes = std::num::NonZeroUsize::new(4096).unwrap();
        let codec = Codec::<harmos::grpc::Transcoded, O::Response, harmos::operations::Response<O>>::new(
            DecodeSettings { message_bytes: bytes, retained_bytes: bytes },
            EncodeSettings { message_bytes: bytes, buffer_bytes: bytes },
        )
        .unwrap();
        let mut client = tonic::client::Grpc::new(channel);
        client.ready().await.unwrap();
        client
            .unary(
                credentialed(harmos::operations::Response::new(O::default()), token),
                tonic::codegen::http::uri::PathAndQuery::from_static(path),
                codec,
            )
            .await
            .err()
    })
}

#[test]
fn table_covers_serves() {
    let served: std::collections::BTreeSet<_> = dokime_server::grpc::methods().map(|(path, _)| path).collect();
    let covered: std::collections::BTreeSet<_> = cases().map(|case| case.path).collect();
    assert_eq!(served, covered, "every served method needs a row here, and a row needs a served method");
    assert_eq!(covered.len(), cases().count(), "no duplicate rows");
}

#[tokio::test]
async fn permission_matrix_enforcement() {
    const PRINCIPALS: &[(Role, &str)] = &[
        (Role::Read, "read-token-012345678901234567890123456789"),
        (Role::Control, "control-token-012345678901234567890123456789"),
        (Role::Admin, "admin-token-012345678901234567890123456789"),
    ];
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let grants = PRINCIPALS.iter().map(|(role, token)| BearerGrant::new(format!("{role:?}"), *role, token).unwrap()).collect();
    let server = Server::start(owner.handle(), Config::local().with_bearer_auth(grants).unwrap()).await.unwrap();
    let channel = Endpoint::from_shared(server.local_endpoint()).unwrap().connect().await.unwrap();
    let roles: std::collections::BTreeMap<_, _> = dokime_server::grpc::methods().collect();

    // Every Method Requires Authentication Before Payload Decoding.
    for case in cases() {
        let refusal =
            (case.call)(channel.clone(), None).await.unwrap_or_else(|| panic!("{} admitted a call with no credential", case.path));
        assert_eq!(refusal.code(), Code::Unauthenticated, "{}", case.path);
    }

    // Distinct Credentials Exercise Every Insufficient Role On One Listener.
    for &(role, token) in PRINCIPALS {
        let advertised: std::collections::BTreeSet<_> = WireClient::new(channel.clone())
            .unary::<contracts::backend::GetAvailability>(credentialed(contracts::backend::GetAvailability {}, Some(token)))
            .await
            .unwrap()
            .into_inner()
            .methods
            .into_iter()
            .collect();
        let expected: std::collections::BTreeSet<_> =
            roles.iter().filter(|(_, required)| role >= **required).map(|(path, _)| (*path).to_owned()).collect();
        assert_eq!(advertised, expected, "{role:?} advertised permissions");

        for case in cases() {
            let required = roles[case.path];
            if role >= required {
                continue;
            }
            let refusal = (case.call)(channel.clone(), Some(token))
                .await
                .unwrap_or_else(|| panic!("{} admitted {role:?}, needs {required:?}", case.path));
            assert_eq!(refusal.code(), Code::PermissionDenied, "{} for {role:?}", case.path);
        }
    }

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

// A remote listener refuses every local-only operation even to Control.
#[tokio::test]
async fn remote_refuses_local() {
    // A throwaway localhost identity generated for this run; nothing is committed.
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate = generated.cert.pem().into_bytes();
    let key = generated.signing_key.serialize_pem().into_bytes();
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let config =
        Config::remote("127.0.0.1:0".parse().unwrap(), TlsIdentity::from_pem(certificate.clone(), key), vec![grant(Role::Control)])
            .unwrap();
    let server = Server::start(owner.handle(), config).await.unwrap();
    let channel = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .tls_config(ClientTlsConfig::new().domain_name("localhost").ca_certificate(Certificate::from_pem(&certificate)))
        .unwrap()
        .connect()
        .await
        .unwrap();

    // Media Capture, File Watches And Host Paths Stay On The Loopback Listener
    let local_only = [
        "/dokime.v1.Media/Screenshot",
        "/dokime.v1.Media/Photo",
        "/dokime.v1.Media/AudioRecording",
        "/dokime.v1.Media/ScreenRecording",
        "/dokime.v1.Media/CameraRecording",
        "/dokime.v1.Files/FileWatch",
    ];
    for case in cases().filter(|case| local_only.contains(&case.path)) {
        let refusal =
            (case.call)(channel.clone(), Some(TOKEN)).await.unwrap_or_else(|| panic!("{} admitted a remote Control caller", case.path));
        assert_eq!(refusal.code(), Code::PermissionDenied, "{}", case.path);
    }
    assert_eq!(cases().filter(|case| local_only.contains(&case.path)).count(), local_only.len());

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
