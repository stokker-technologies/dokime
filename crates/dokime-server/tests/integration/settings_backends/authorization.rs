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
use dokime::domain::model;

/* === Constants === */

const NATIVE_ONLY_METHODS: [&str; 6] = [
    "/dokime.v1.Media/Screenshot",
    "/dokime.v1.Media/Photo",
    "/dokime.v1.Media/AudioRecording",
    "/dokime.v1.Media/ScreenRecording",
    "/dokime.v1.Media/CameraRecording",
    "/dokime.v1.Files/FileWatch",
];

const STREAMING_CASES: &[Case] = &[
    Case {
        path: "/dokime.v1.Content/UploadArtifact",
        call: |stream, token| {
            Box::pin(async move {
                let entries = Vec::<model::content::wire::UploadArtifactFrame>::new();
                let frames = tokio_stream::iter(entries);
                let request = secreted(frames, token);
                let client = WireClient::new(stream);
                let response = client.upload::<contracts::content::UploadArtifact>(request).await;
                response.err()
            })
        },
    },
    Case {
        path: "/dokime.v1.Plugins/Install",
        call: |stream, token| {
            Box::pin(async move {
                let entries = Vec::<model::plugins::wire::InstallPluginFrame>::new();
                let frames = tokio_stream::iter(entries);
                let request = secreted(frames, token);
                let client = WireClient::new(stream);
                let response = client.upload::<contracts::plugins::Install>(request).await;
                response.err()
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
                        .$method::<$operation>(secreted(request, token))
                        .await
                        .err()
                })
            },
        },)*];
    };
}

matrix! {
    "/dokime.v1.Remote/ReadListener", unary, contracts::remote::ReadListener = contracts::remote::ReadListener {};
    "/dokime.v1.Remote/UpdateListener", unary, contracts::remote::UpdateListener = contracts::remote::UpdateListener { settings: None };
    "/dokime.v1.Remote/CreatePairing", unary, contracts::remote::CreatePairing = contracts::remote::CreatePairing {
        label: "Fixture".into(),
    };
    "/dokime.v1.Remote/ListPairedClients", unary, contracts::remote::ListPairedClients = contracts::remote::ListPairedClients {
        cursor: None, limit: 100
    };
    "/dokime.v1.Remote/RevokePairedClient", unary, contracts::remote::RevokePairedClient = contracts::remote::RevokePairedClient {
        identifier: "fixture".into()
    };
    "/dokime.v1.Remote/RenamePairedClient", unary, contracts::remote::RenamePairedClient = contracts::remote::RenamePairedClient { identifier: "fixture".into(), label: "Renamed".into() };
    "/dokime.v1.Remote/ListSessions", unary, contracts::remote::ListSessions = contracts::remote::ListSessions { cursor: None, limit: 20 };
    "/dokime.v1.Remote/RegenerateCertificate", unary, contracts::remote::RegenerateCertificate = contracts::remote::RegenerateCertificate { certificate_names: vec![] };
    "/dokime.v1.Reports/MaterializeReport", unary, contracts::reports::MaterializeReport = contracts::reports::MaterializeReport {
        subject: model::reports::wire::ReportSubject::Execution(String::new()),
        at: dokime::harmos::storage::StorageCursor {
            history: dokime::harmos::storage::StoreHistory {
                store: dokime::harmos::Identifier::new("authorization").unwrap(),
                epoch: dokime::harmos::Identifier::new("test").unwrap(),
            },
            sequence: 0,
        },
    };
    "/harmos.host.v1.Host/Catalog", unary, dokime::harmos::builtins::host::Catalog = dokime::harmos::builtins::host::Catalog {
        kind: dokime::harmos::builtins::host::CatalogKind::Operations,
        schema: None,
        after: None,
        rows: None,
        bytes: None,
    };
    "/dokime.v1.Content/FollowExecutionFile", download, contracts::content::FollowExecutionFile;
    "/dokime.v1.Appearance/GetAppearance", unary, contracts::appearance::GetAppearance;
    "/dokime.v1.Backend/CollectDiagnostics", download, contracts::backend::CollectDiagnostics;
    "/dokime.v1.Backend/GetAvailability", unary, contracts::backend::GetAvailability;
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
    "/dokime.v1.Content/DownloadEvidence", download, contracts::content::DownloadEvidence = contracts::content::DownloadEvidence { subject: None };
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
    "/dokime.v1.Execution/AppendExecutions", unary, contracts::execution::AppendExecutions;
    "/dokime.v1.Execution/CopyCycle", unary, contracts::execution::CopyCycle;
    "/dokime.v1.Execution/CreateCycle", unary, contracts::execution::CreateCycle;
    "/dokime.v1.Execution/DeleteCase", unary, contracts::execution::DeleteCase;
    "/dokime.v1.Execution/GetCase", unary, contracts::execution::GetCase;
    "/dokime.v1.Execution/GetCycle", unary, contracts::execution::GetCycle;
    "/dokime.v1.Execution/GetExecution", unary, contracts::execution::GetExecution;
    "/dokime.v1.Execution/GetExecutionCommand", unary, contracts::execution::GetExecutionCommand;
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
    "/dokime.v1.Execution/PauseCycle", unary, contracts::execution::PauseCycle;
    "/dokime.v1.Execution/ResumeCycle", unary, contracts::execution::ResumeCycle;
    "/dokime.v1.Execution/StartExecution", unary, contracts::execution::StartExecution = contracts::execution::StartExecution {
        execution_id: String::new(),
        mode: model::execution::wire::ExecutionMode::Automatic,
    };
    "/dokime.v1.Execution/PauseExecution", unary, contracts::execution::PauseExecution;
    "/dokime.v1.Execution/ResumeExecution", unary, contracts::execution::ResumeExecution;
    "/dokime.v1.Execution/StopExecution", unary, contracts::execution::StopExecution;
    "/dokime.v1.Execution/CancelExecution", unary, contracts::execution::CancelExecution;
    "/dokime.v1.Execution/SubmitExecutionCommand", unary, contracts::execution::SubmitExecutionCommand = contracts::execution::SubmitExecutionCommand {
        request_id: String::new(),
        command_id: String::new(),
        execution_id: String::new(),
        action: model::execution::wire::ExecutionCommandAction::Unspecified,
        step_id: None,
    };
    "/dokime.v1.Execution/RenameCycle", unary, contracts::execution::RenameCycle;
    "/dokime.v1.Execution/RenameExecution", unary, contracts::execution::RenameExecution;
    "/dokime.v1.Execution/SetCycleArchived", unary, contracts::execution::SetCycleArchived;
    "/dokime.v1.Execution/SetExecutionArchived", unary, contracts::execution::SetExecutionArchived;
    "/dokime.v1.Execution/SealCycle", unary, contracts::execution::SealCycle = contracts::execution::SealCycle { cycle_id: String::new(), context: None };
    "/dokime.v1.Execution/UpdateCyclePolicy", unary, contracts::execution::UpdateCyclePolicy;
    "/dokime.v1.Files/FileWatch", download, contracts::files::FileWatch = contracts::files::FileWatch {
        execution_id: None,
        source: None,
        rules: Vec::new(),
        output: String::new(),
        policy: model::files::wire::FileWatchPolicy::Unspecified,
        selection: model::files::wire::FileWatchSelection::All,
        wait_for_source_seconds: None,
        poll_interval_ms: None,
        maximum_seconds: None,
        maximum_files: None,
        maximum_bytes: None,
    };
    "/dokime.v1.Health/ListHealth", unary, contracts::notes::ListHealth;
    "/dokime.v1.History/BrowseCycles", unary, contracts::history::BrowseCycles;
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
    "/dokime.v1.Planning/MaterializeTemplate", unary, contracts::planning::MaterializeTemplate;
    "/dokime.v1.Plots/ListAnalyses", unary, contracts::plots::ListAnalyses;
    "/dokime.v1.Plots/QuerySnapshots", unary, contracts::plots::QuerySnapshots;
    "/dokime.v1.Plots/Watch", download, contracts::plots::Watch;
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
    "/dokime.v1.Query/ListCycles", unary, contracts::query::ListCycles = super::super::finite_queries::cycles();
    "/dokime.v1.Query/ListExecutions", unary, contracts::query::ListExecutions = contracts::query::ListExecutions {
        selection: model::query::selection::ExecutionSelection { template_ids: vec![], cycle_ids: vec![], statuses: vec![], started_at: None, archive: None },
        page: model::query::page::PageRequest { rows: None, bytes: None, after: None }, at: None,
    };
    "/dokime.v1.Query/ListRecords", unary, contracts::query::ListRecords = contracts::query::ListRecords {
        scope: model::query::selection::RecordScope::Execution(String::new()), record_type: String::new(), versions: vec![], r#where: vec![], between: None,
        page: model::query::page::PageRequest { rows: None, bytes: None, after: None }, at: None,
    };
    "/dokime.v1.Query/ListSamples", unary, contracts::query::ListSamples = contracts::query::ListSamples {
        execution_id: String::new(), stream: String::new(), versions: vec![], fields: vec![], r#where: vec![], between: None, bucket: None,
        page: model::query::page::PageRequest { rows: None, bytes: None, after: None }, at: None,
    };
    "/dokime.v1.Query/AggregateSamples", unary, contracts::query::AggregateSamples = contracts::query::AggregateSamples {
        execution_id: String::new(), stream: String::new(), versions: vec![], r#where: vec![], between: None, outputs: vec![], at: None,
    };
    "/dokime.v1.Query/Aggregate", unary, contracts::query::Aggregate = contracts::query::Aggregate {
        source: model::query::wire::QuerySource::Unspecified,
        filters: vec![],
        group_by: vec![],
        aggregates: vec![],
        limit: 0,
    };
    "/dokime.v1.Reports/DownloadCycleReport", download, contracts::reports::DownloadCycleReport;
    "/dokime.v1.Reports/DownloadExecutionReport", download, contracts::reports::DownloadExecutionReport;
    "/dokime.v1.Secrets/Lookup", unary, contracts::secrets::Lookup = contracts::secrets::Lookup { identifier: String::new() };
    "/dokime.v1.Secrets/List", unary, contracts::secrets::List = contracts::secrets::List {};
    "/dokime.v1.Secrets/Put", unary, contracts::secrets::Put = contracts::secrets::Put {
        metadata: model::secrets::catalog::Metadata { identifier: String::new(), label: String::new(), description: None }, value: None, generation: 0,
    };
    "/dokime.v1.Secrets/Remove", unary, contracts::secrets::Remove = contracts::secrets::Remove { identifiers: vec![], generation: 0 };
    "/dokime.v1.Secrets/Export", unary, contracts::secrets::Export = contracts::secrets::Export { identifiers: vec![], generation: 0 };
    "/dokime.v1.Secrets/Preview", unary, contracts::secrets::Preview = contracts::secrets::Preview { data: vec![] };
    "/dokime.v1.Secrets/Import", unary, contracts::secrets::Import = contracts::secrets::Import { data: vec![], choices: vec![], generation: 0 };
    "/dokime.v1.Settings/GetPluginSettings", unary, contracts::settings::GetPluginSettings;
    "/dokime.v1.Settings/GetSettings", unary, contracts::settings::GetSettings;
    "/dokime.v1.Settings/ReadApplicationSettings", unary, contracts::settings::ReadApplicationSettings;
    "/dokime.v1.Settings/ListStorageSources", unary, contracts::settings::ListStorageSources;
    "/dokime.v1.Settings/ReorderStorageSources", unary, contracts::settings::ReorderStorageSources;
    "/dokime.v1.Settings/SetStorageSourceEnabled", unary, contracts::settings::SetStorageSourceEnabled;
    "/dokime.v1.Settings/UpdateApplicationSettings", unary, contracts::settings::UpdateApplicationSettings = contracts::settings::UpdateApplicationSettings {
        update: model::settings::application::UpdateSettings { expected_revision: 0, settings: Default::default() },
    };
    "/dokime.v1.Settings/UpdatePluginSettings", unary, contracts::settings::UpdatePluginSettings;
    "/dokime.v1.Telemetry/ListSamples", unary, contracts::telemetry::ListSamples;
    "/dokime.v1.Telemetry/WatchSamples", download, contracts::telemetry::WatchSamples;
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

fn secreted<T>(message: T, token: Option<&'static str>) -> Request<T> {
    match token {
        Some(token) => auth(message, token),
        None => Request::new(message),
    }
}

fn cases() -> impl Iterator<Item = &'static Case> {
    use dokime::operations::runtime::{GetHealth, GetIdentity};
    static OPERATIONS: &[Case] = &[
        Case {
            path: "/dokime.v1.Runtime/GetHealth",
            call: |channel, token| operation::<GetHealth>(channel, token),
        },
        Case {
            path: "/dokime.v1.Runtime/GetIdentity",
            call: |channel, token| operation::<GetIdentity>(channel, token),
        },
    ];
    CASES.iter().chain(STREAMING_CASES).chain(OPERATIONS)
}

fn operation<O>(channel: Channel, token: Option<&'static str>) -> Refusal
where
    O: dokime::harmos::operations::OperationDeclaration + dokime::harmos::domain::Portable + Default,
    O::Response: dokime::harmos::domain::Portable + dokime::harmos::MemorySize,
{
    Box::pin(async move {
        use dokime::harmos::grpc::{Codec, DecodeSettings, EncodeSettings};
        let bytes = std::num::NonZeroUsize::new(4096).unwrap();
        let codec = Codec::<dokime::harmos::grpc::Transcoded, O::Response, dokime::harmos::operations::Response<O>>::new(
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
        let declaration = O::default();
        let envelope = dokime::harmos::operations::Response::new(declaration);
        let request = secreted(envelope, token);
        let route = dokime::operations::path::<O>().parse::<tonic::codegen::http::uri::PathAndQuery>().unwrap();
        let response = client.unary(request, route, codec).await;
        response.err()
    })
}

fn assert_case_admission(case: &Case, local_only: bool, refusal: Option<Status>) {
    if local_only {
        let refusal = refusal.unwrap_or_else(|| panic!("{} admitted a native-only call", case.path));
        assert_eq!(refusal.code(), Code::PermissionDenied, "{}", case.path);
        return;
    }
    let Some(refusal) = refusal else {
        return;
    };
    assert_ne!(refusal.code(), Code::Unauthenticated, "{}", case.path);
    assert_ne!(refusal.code(), Code::PermissionDenied, "{}", case.path);
}

async fn assert_key_refusals(channel: &Channel, authenticated: bool) {
    if !authenticated {
        return;
    }
    let missing_key_calls = cases().map(|case| (case, None));
    let wrong_key = Some("wrong-key-012345678901234567890123456789");
    let wrong_key_calls = cases().map(|case| (case, wrong_key));
    for (case, rejected_key) in missing_key_calls.chain(wrong_key_calls) {
        let result = (case.call)(channel.clone(), rejected_key).await;
        let refusal = result.unwrap_or_else(|| panic!("{} admitted a call without its key", case.path));
        assert_eq!(refusal.code(), Code::Unauthenticated, "{}", case.path);
    }
}

pub(crate) fn table_covers_serves() {
    let served: std::collections::BTreeSet<_> = dokime_server::grpc::methods().map(|(path, _)| path).collect();
    let covered: std::collections::BTreeSet<_> = cases().map(|case| case.path.to_owned()).collect();
    assert_eq!(served, covered, "every served method needs a row here, and a row needs a served method");
    assert_eq!(covered.len(), cases().count(), "no duplicate rows");
}

pub(crate) async fn key_boundary_matrix() {
    assert_remote_key_boundary(true).await;
    assert_remote_key_boundary(false).await;
}

async fn assert_remote_key_boundary(authenticated: bool) {
    let root = tempfile::tempdir().unwrap();
    let token = authenticated.then_some(TOKEN);
    let config = Config::remote("127.0.0.1:0".parse().unwrap(), None, token).unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(config.clone()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;

    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let channel = Endpoint::from_shared(server.local_endpoint()).unwrap().connect().await.unwrap();
    let methods: std::collections::BTreeMap<_, _> = dokime_server::grpc::methods().collect();

    // Every Method Requires Authentication Before Payload Decoding.
    assert_key_refusals(&channel, authenticated).await;

    // One Valid Key Or Explicit Authentication-Off Grants All Remote Product Methods.
    let advertised: std::collections::BTreeSet<_> = WireClient::new(channel.clone())
        .unary::<contracts::backend::GetAvailability>(secreted(contracts::backend::GetAvailability {}, token))
        .await
        .unwrap()
        .into_inner()
        .methods
        .into_iter()
        .collect();
    let public_methods = methods.iter().filter(|(_, local_only)| !**local_only);
    let expected: std::collections::BTreeSet<_> = public_methods.map(|(path, _)| path.clone()).collect();
    assert_eq!(advertised, expected, "availability must exclude native-only and internal host providers");

    for case in cases() {
        let refusal = (case.call)(channel.clone(), token).await;
        assert_case_admission(case, methods[case.path], refusal);
    }

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

// TLS And A Valid Full-Control Key Do Not Confer Native Listener Trust.
pub(crate) async fn remote_refuses_local() {
    // A throwaway localhost identity generated for this run; nothing is committed.
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let certificate = generated.cert.pem().into_bytes();
    let key = generated.signing_key.serialize_pem().into_bytes();
    let root = tempfile::tempdir().unwrap();
    let address = "127.0.0.1:0".parse().unwrap();
    let identity = TlsIdentity::from_pem(certificate.clone(), key);
    let config = Config::remote(address, Some(identity), Some(TOKEN)).unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(config.clone()).unwrap();
    let mut owner = runtime(root.path(), &prepared_owner).await;

    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let uri = server.local_endpoint();
    let endpoint = Endpoint::from_shared(uri).unwrap();
    let trust = Certificate::from_pem(&certificate);
    let tls = ClientTlsConfig::new().domain_name("localhost");
    let tls = tls.ca_certificate(trust);
    let endpoint = endpoint.tls_config(tls).unwrap();
    let channel = endpoint.connect().await.unwrap();

    // Media Capture, File Watches And Host Paths Stay On The Loopback Listener
    let mut covered = 0;
    let served = cases();
    let native_cases = served.filter(|case| {
        let path = case.path;
        NATIVE_ONLY_METHODS.contains(&path)
    });
    for case in native_cases {
        covered += 1;
        let refusal =
            (case.call)(channel.clone(), Some(TOKEN)).await.unwrap_or_else(|| panic!("{} admitted a native-only remote call", case.path));
        assert_eq!(refusal.code(), Code::PermissionDenied, "{}", case.path);
    }
    assert_eq!(covered, NATIVE_ONLY_METHODS.len());

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn finite_query_authorization_matrix() {
    table_covers_serves();
    for authenticated in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let token = authenticated.then_some(TOKEN);
        let config = Config::remote("127.0.0.1:0".parse().unwrap(), None, token).unwrap();
        let prepared = dokime_server::grpc::Service::prepare(config).unwrap();
        let mut owner = runtime(root.path(), &prepared).await;
        let server = prepared.ready().await.unwrap().listen().await.unwrap();
        let channel = Endpoint::from_shared(server.local_endpoint()).unwrap().connect().await.unwrap();
        for case in cases().filter(|case| case.path.starts_with("/dokime.v1.Query/")) {
            if authenticated {
                for rejected in [None, Some("wrong-key-012345678901234567890123456789")] {
                    let status = (case.call)(channel.clone(), rejected).await.unwrap();
                    assert_eq!(status.code(), Code::Unauthenticated);
                }
            }
            assert_case_admission(case, false, (case.call)(channel.clone(), token).await);
        }
        server.shutdown().await.unwrap();
        owner.shutdown().await.unwrap();
    }
}
