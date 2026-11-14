// @generated from protocol descriptors; do not edit.
use dokime_protocol::host::AggregatesGetRequest;
use dokime_protocol::host::AggregatesGetResponse;
use dokime_protocol::host::AnalysesListRequest;
use dokime_protocol::host::AnalysesListResponse;
use dokime_protocol::host::ApplicationSettings;
use dokime_protocol::host::BackendAppearanceRequest;
use dokime_protocol::host::BackendAppearanceResponse;
use dokime_protocol::host::CasesCreateRequest;
use dokime_protocol::host::CasesCreateResponse;
use dokime_protocol::host::CasesDeleteRequest;
use dokime_protocol::host::CasesDeleteResponse;
use dokime_protocol::host::CasesGetRequest;
use dokime_protocol::host::CasesGetResponse;
use dokime_protocol::host::CasesListRequest;
use dokime_protocol::host::CasesListResponse;
use dokime_protocol::host::CasesParametersRequest;
use dokime_protocol::host::CasesParametersResponse;
use dokime_protocol::host::CasesWatchRequest;
use dokime_protocol::host::CasesWatchResponse;
use dokime_protocol::host::CycleFilter;
use dokime_protocol::host::CycleOrder;
use dokime_protocol::host::CycleSelection;
use dokime_protocol::host::CycleWritable;
use dokime_protocol::host::CyclesCreateRequest;
use dokime_protocol::host::CyclesCreateResponse;
use dokime_protocol::host::CyclesGetRequest;
use dokime_protocol::host::CyclesGetResponse;
use dokime_protocol::host::CyclesListRequest;
use dokime_protocol::host::CyclesListResponse;
use dokime_protocol::host::CyclesSealRequest;
use dokime_protocol::host::CyclesSealResponse;
use dokime_protocol::host::CyclesSetRequest;
use dokime_protocol::host::CyclesSetResponse;
use dokime_protocol::host::CyclesWatchRequest;
use dokime_protocol::host::CyclesWatchResponse;
use dokime_protocol::host::DocumentScope;
use dokime_protocol::host::DocumentsGetRequest;
use dokime_protocol::host::DocumentsGetResponse;
use dokime_protocol::host::DocumentsWatchRequest;
use dokime_protocol::host::DocumentsWatchResponse;
use dokime_protocol::host::EvidenceArchiveRequest;
use dokime_protocol::host::EvidenceArchiveResponse;
use dokime_protocol::host::EvidenceStatisticsRequest;
use dokime_protocol::host::EvidenceStatisticsResponse;
use dokime_protocol::host::EvidenceSubject;
use dokime_protocol::host::ExecutionBinding;
use dokime_protocol::host::ExecutionFilter;
use dokime_protocol::host::ExecutionMode;
use dokime_protocol::host::ExecutionOrder;
use dokime_protocol::host::ExecutionParameterSet;
use dokime_protocol::host::ExecutionSelection;
use dokime_protocol::host::ExecutionWritable;
use dokime_protocol::host::ExecutionsCancelRequest;
use dokime_protocol::host::ExecutionsCancelResponse;
use dokime_protocol::host::ExecutionsCreateRequest;
use dokime_protocol::host::ExecutionsCreateResponse;
use dokime_protocol::host::ExecutionsDeleteRequest;
use dokime_protocol::host::ExecutionsDeleteResponse;
use dokime_protocol::host::ExecutionsGetRequest;
use dokime_protocol::host::ExecutionsGetResponse;
use dokime_protocol::host::ExecutionsListRequest;
use dokime_protocol::host::ExecutionsListResponse;
use dokime_protocol::host::ExecutionsParametersRequest;
use dokime_protocol::host::ExecutionsParametersResponse;
use dokime_protocol::host::ExecutionsSetRequest;
use dokime_protocol::host::ExecutionsSetResponse;
use dokime_protocol::host::ExecutionsStartRequest;
use dokime_protocol::host::ExecutionsStartResponse;
use dokime_protocol::host::ExecutionsStepsRequest;
use dokime_protocol::host::ExecutionsStepsResponse;
use dokime_protocol::host::ExecutionsStopRequest;
use dokime_protocol::host::ExecutionsStopResponse;
use dokime_protocol::host::ExecutionsWatchRequest;
use dokime_protocol::host::ExecutionsWatchResponse;
use dokime_protocol::host::FilesDeleteRequest;
use dokime_protocol::host::FilesDeleteResponse;
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::FilesGetResponse;
use dokime_protocol::host::FilesReadRequest;
use dokime_protocol::host::FilesReadResponse;
use dokime_protocol::host::FilesWatchRequest;
use dokime_protocol::host::FilesWatchResponse;
use dokime_protocol::host::Output;
use dokime_protocol::host::PromptDefinitionsGetRequest;
use dokime_protocol::host::PromptDefinitionsGetResponse;
use dokime_protocol::host::PromptDefinitionsListRequest;
use dokime_protocol::host::PromptDefinitionsListResponse;
use dokime_protocol::host::PromptsGetRequest;
use dokime_protocol::host::PromptsGetResponse;
use dokime_protocol::host::PromptsListRequest;
use dokime_protocol::host::PromptsListResponse;
use dokime_protocol::host::PromptsWatchRequest;
use dokime_protocol::host::PromptsWatchResponse;
use dokime_protocol::host::QueryAggregate;
use dokime_protocol::host::QueryFilter;
use dokime_protocol::host::QuerySource;
use dokime_protocol::host::RecordSchemasGetRequest;
use dokime_protocol::host::RecordSchemasGetResponse;
use dokime_protocol::host::RecordSchemasListRequest;
use dokime_protocol::host::RecordSchemasListResponse;
use dokime_protocol::host::ReportSubject;
use dokime_protocol::host::ReportsCreateRequest;
use dokime_protocol::host::ReportsCreateResponse;
use dokime_protocol::host::ReportsGetRequest;
use dokime_protocol::host::ReportsGetResponse;
use dokime_protocol::host::ReportsPublishRequest;
use dokime_protocol::host::ReportsPublishResponse;
use dokime_protocol::host::SecretImportChoice;
use dokime_protocol::host::SecretWritable;
use dokime_protocol::host::SecretsDeleteRequest;
use dokime_protocol::host::SecretsDeleteResponse;
use dokime_protocol::host::SecretsExportRequest;
use dokime_protocol::host::SecretsExportResponse;
use dokime_protocol::host::SecretsGetRequest;
use dokime_protocol::host::SecretsGetResponse;
use dokime_protocol::host::SecretsImportRequest;
use dokime_protocol::host::SecretsImportResponse;
use dokime_protocol::host::SecretsListRequest;
use dokime_protocol::host::SecretsListResponse;
use dokime_protocol::host::SecretsSetRequest;
use dokime_protocol::host::SecretsSetResponse;
use dokime_protocol::host::SecretsWatchRequest;
use dokime_protocol::host::SecretsWatchResponse;
use dokime_protocol::host::SelectionPlan;
use dokime_protocol::host::SettingsGetRequest;
use dokime_protocol::host::SettingsGetResponse;
use dokime_protocol::host::SettingsSchemaRequest;
use dokime_protocol::host::SettingsSchemaResponse;
use dokime_protocol::host::SettingsSetRequest;
use dokime_protocol::host::SettingsSetResponse;
use dokime_protocol::host::StreamSchemasGetRequest;
use dokime_protocol::host::StreamSchemasGetResponse;
use dokime_protocol::host::StreamSchemasListRequest;
use dokime_protocol::host::StreamSchemasListResponse;
use dokime_protocol::host::SuitesGetRequest;
use dokime_protocol::host::SuitesGetResponse;
use dokime_protocol::host::SuitesListRequest;
use dokime_protocol::host::SuitesListResponse;
use dokime_protocol::host::backend::OverviewGetRequest as BackendOverviewGetRequest;
use dokime_protocol::host::backend::OverviewGetResponse as BackendOverviewGetResponse;
use dokime_protocol::host::backend::OverviewWatchRequest as BackendOverviewWatchRequest;
use dokime_protocol::host::backend::OverviewWatchResponse as BackendOverviewWatchResponse;
use dokime_protocol::host::content::ArtifactsCreateRequest as ContentArtifactsCreateRequest;
use dokime_protocol::host::content::ArtifactsCreateResponse as ContentArtifactsCreateResponse;
use dokime_protocol::host::content::ArtifactsGetRequest as ContentArtifactsGetRequest;
use dokime_protocol::host::content::ArtifactsGetResponse as ContentArtifactsGetResponse;
use dokime_protocol::host::content::ArtifactsListRequest as ContentArtifactsListRequest;
use dokime_protocol::host::content::ArtifactsListResponse as ContentArtifactsListResponse;
use dokime_protocol::host::content::ArtifactsWatchRequest as ContentArtifactsWatchRequest;
use dokime_protocol::host::content::ArtifactsWatchResponse as ContentArtifactsWatchResponse;
use dokime_protocol::host::content::ExecutionFilesGetRequest as ContentExecutionFilesGetRequest;
use dokime_protocol::host::content::ExecutionFilesGetResponse as ContentExecutionFilesGetResponse;
use dokime_protocol::host::content::ExecutionFilesListRequest as ContentExecutionFilesListRequest;
use dokime_protocol::host::content::ExecutionFilesListResponse as ContentExecutionFilesListResponse;
use dokime_protocol::host::content::ExecutionFilesWatchRequest as ContentExecutionFilesWatchRequest;
use dokime_protocol::host::content::ExecutionFilesWatchResponse as ContentExecutionFilesWatchResponse;
use dokime_protocol::host::evidence::AnnotationOwner as EvidenceAnnotationOwner;
use dokime_protocol::host::evidence::AnnotationSelection as EvidenceAnnotationSelection;
use dokime_protocol::host::evidence::BucketRequest as EvidenceBucketRequest;
use dokime_protocol::host::evidence::CapturedRecordSelection as EvidenceCapturedRecordSelection;
use dokime_protocol::host::evidence::HealthEventsListRequest as EvidenceHealthEventsListRequest;
use dokime_protocol::host::evidence::HealthEventsListResponse as EvidenceHealthEventsListResponse;
use dokime_protocol::host::evidence::HealthEventsWatchRequest as EvidenceHealthEventsWatchRequest;
use dokime_protocol::host::evidence::HealthEventsWatchResponse as EvidenceHealthEventsWatchResponse;
use dokime_protocol::host::evidence::NoteWritable as EvidenceNoteWritable;
use dokime_protocol::host::evidence::NotesCreateRequest as EvidenceNotesCreateRequest;
use dokime_protocol::host::evidence::NotesCreateResponse as EvidenceNotesCreateResponse;
use dokime_protocol::host::evidence::NotesGetRequest as EvidenceNotesGetRequest;
use dokime_protocol::host::evidence::NotesGetResponse as EvidenceNotesGetResponse;
use dokime_protocol::host::evidence::NotesListRequest as EvidenceNotesListRequest;
use dokime_protocol::host::evidence::NotesListResponse as EvidenceNotesListResponse;
use dokime_protocol::host::evidence::NotesSetRequest as EvidenceNotesSetRequest;
use dokime_protocol::host::evidence::NotesSetResponse as EvidenceNotesSetResponse;
use dokime_protocol::host::evidence::NotesWatchRequest as EvidenceNotesWatchRequest;
use dokime_protocol::host::evidence::NotesWatchResponse as EvidenceNotesWatchResponse;
use dokime_protocol::host::evidence::RecordScope as EvidenceRecordScope;
use dokime_protocol::host::evidence::RecordsListRequest as EvidenceRecordsListRequest;
use dokime_protocol::host::evidence::RecordsListResponse as EvidenceRecordsListResponse;
use dokime_protocol::host::evidence::RecordsWatchRequest as EvidenceRecordsWatchRequest;
use dokime_protocol::host::evidence::RecordsWatchResponse as EvidenceRecordsWatchResponse;
use dokime_protocol::host::evidence::SamplesListRequest as EvidenceSamplesListRequest;
use dokime_protocol::host::evidence::SamplesListResponse as EvidenceSamplesListResponse;
use dokime_protocol::host::evidence::SamplesWatchRequest as EvidenceSamplesWatchRequest;
use dokime_protocol::host::evidence::SamplesWatchResponse as EvidenceSamplesWatchResponse;
use dokime_protocol::host::evidence::TraceabilityGetRequest as EvidenceTraceabilityGetRequest;
use dokime_protocol::host::evidence::TraceabilityGetResponse as EvidenceTraceabilityGetResponse;
use dokime_protocol::host::evidence::TraceabilityWatchRequest as EvidenceTraceabilityWatchRequest;
use dokime_protocol::host::evidence::TraceabilityWatchResponse as EvidenceTraceabilityWatchResponse;
use dokime_protocol::host::executions::CommandsCreateRequest as ExecutionsCommandsCreateRequest;
use dokime_protocol::host::executions::CommandsCreateResponse as ExecutionsCommandsCreateResponse;
use dokime_protocol::host::executions::CommandsGetRequest as ExecutionsCommandsGetRequest;
use dokime_protocol::host::executions::CommandsGetResponse as ExecutionsCommandsGetResponse;
use dokime_protocol::host::executions::ExecutionCommandAction as ExecutionsExecutionCommandAction;
use dokime_protocol::host::executions::ProgressGetRequest as ExecutionsProgressGetRequest;
use dokime_protocol::host::executions::ProgressGetResponse as ExecutionsProgressGetResponse;
use dokime_protocol::host::executions::ProgressWatchRequest as ExecutionsProgressWatchRequest;
use dokime_protocol::host::executions::ProgressWatchResponse as ExecutionsProgressWatchResponse;
use dokime_protocol::host::executions::SchemasGetRequest as ExecutionsSchemasGetRequest;
use dokime_protocol::host::executions::SchemasGetResponse as ExecutionsSchemasGetResponse;
use dokime_protocol::host::files::UploadsAppendRequest as FilesUploadsAppendRequest;
use dokime_protocol::host::files::UploadsAppendResponse as FilesUploadsAppendResponse;
use dokime_protocol::host::files::UploadsCreateRequest as FilesUploadsCreateRequest;
use dokime_protocol::host::files::UploadsCreateResponse as FilesUploadsCreateResponse;
use dokime_protocol::host::files::UploadsDeleteRequest as FilesUploadsDeleteRequest;
use dokime_protocol::host::files::UploadsDeleteResponse as FilesUploadsDeleteResponse;
use dokime_protocol::host::files::UploadsGetRequest as FilesUploadsGetRequest;
use dokime_protocol::host::files::UploadsGetResponse as FilesUploadsGetResponse;
use dokime_protocol::host::files::UploadsPublishRequest as FilesUploadsPublishRequest;
use dokime_protocol::host::files::UploadsPublishResponse as FilesUploadsPublishResponse;
use dokime_protocol::host::files::UploadsResumeRequest as FilesUploadsResumeRequest;
use dokime_protocol::host::files::UploadsResumeResponse as FilesUploadsResumeResponse;
use dokime_protocol::host::media::CaptureKind as MediaCaptureKind;
use dokime_protocol::host::media::CapturesCreateRequest as MediaCapturesCreateRequest;
use dokime_protocol::host::media::CapturesCreateResponse as MediaCapturesCreateResponse;
use dokime_protocol::host::media::CapturesGetRequest as MediaCapturesGetRequest;
use dokime_protocol::host::media::CapturesGetResponse as MediaCapturesGetResponse;
use dokime_protocol::host::media::CapturesStopRequest as MediaCapturesStopRequest;
use dokime_protocol::host::media::CapturesStopResponse as MediaCapturesStopResponse;
use dokime_protocol::host::media::CapturesWatchRequest as MediaCapturesWatchRequest;
use dokime_protocol::host::media::CapturesWatchResponse as MediaCapturesWatchResponse;
use dokime_protocol::host::media::RecordingAudio as MediaRecordingAudio;
use dokime_protocol::host::plots::PlotRequest as PlotsPlotRequest;
use dokime_protocol::host::plots::PlotSettingsValue as PlotsPlotSettingsValue;
use dokime_protocol::host::plots::SettingsGetRequest as PlotsSettingsGetRequest;
use dokime_protocol::host::plots::SettingsGetResponse as PlotsSettingsGetResponse;
use dokime_protocol::host::plots::SettingsSetRequest as PlotsSettingsSetRequest;
use dokime_protocol::host::plots::SettingsSetResponse as PlotsSettingsSetResponse;
use dokime_protocol::host::plots::SnapshotsListRequest as PlotsSnapshotsListRequest;
use dokime_protocol::host::plots::SnapshotsListResponse as PlotsSnapshotsListResponse;
use dokime_protocol::host::plots::SnapshotsWatchRequest as PlotsSnapshotsWatchRequest;
use dokime_protocol::host::plots::SnapshotsWatchResponse as PlotsSnapshotsWatchResponse;
use dokime_protocol::host::plugins::SecretBindingValue as PluginsSecretBindingValue;
use dokime_protocol::host::plugins::SecretsDeleteRequest as PluginsSecretsDeleteRequest;
use dokime_protocol::host::plugins::SecretsDeleteResponse as PluginsSecretsDeleteResponse;
use dokime_protocol::host::plugins::SecretsListRequest as PluginsSecretsListRequest;
use dokime_protocol::host::plugins::SecretsListResponse as PluginsSecretsListResponse;
use dokime_protocol::host::plugins::SecretsSetRequest as PluginsSecretsSetRequest;
use dokime_protocol::host::plugins::SecretsSetResponse as PluginsSecretsSetResponse;
use dokime_protocol::host::prompts::AnswersCreateRequest as PromptsAnswersCreateRequest;
use dokime_protocol::host::prompts::AnswersCreateResponse as PromptsAnswersCreateResponse;
use dokime_protocol::host::storage::SourcesGetRequest as StorageSourcesGetRequest;
use dokime_protocol::host::storage::SourcesGetResponse as StorageSourcesGetResponse;
use dokime_protocol::host::storage::SourcesSetRequest as StorageSourcesSetRequest;
use dokime_protocol::host::storage::SourcesSetResponse as StorageSourcesSetResponse;
use dokime_protocol::host::storage::StorageSources as StorageStorageSources;
use dokime_protocol::model::ArtifactScope;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::MapValue;
use dokime_protocol::model::Metadata;
use dokime_protocol::model::Origin;
use dokime_protocol::model::Window;
use harmos::client::{CallOptions, ClientTransport, Watching, action_error, request_id};
use harmos_protocol::host::InvocationsCancelRequest;
use harmos_protocol::host::InvocationsCancelResponse;
use harmos_protocol::host::InvocationsCreateRequest;
use harmos_protocol::host::InvocationsCreateResponse;
use harmos_protocol::host::InvocationsGetRequest;
use harmos_protocol::host::InvocationsGetResponse;
use harmos_protocol::host::InvocationsWatchRequest;
use harmos_protocol::host::InvocationsWatchResponse;
use harmos_protocol::host::OperationsGetRequest;
use harmos_protocol::host::OperationsGetResponse;
use harmos_protocol::host::OperationsListRequest;
use harmos_protocol::host::OperationsListResponse;
use harmos_protocol::host::PluginEntry;
use harmos_protocol::host::PluginPlacement;
use harmos_protocol::host::PluginValue;
use harmos_protocol::host::PluginsCreateRequest;
use harmos_protocol::host::PluginsCreateResponse;
use harmos_protocol::host::PluginsDeleteRequest;
use harmos_protocol::host::PluginsDeleteResponse;
use harmos_protocol::host::PluginsGetRequest;
use harmos_protocol::host::PluginsGetResponse;
use harmos_protocol::host::PluginsListRequest;
use harmos_protocol::host::PluginsListResponse;
use harmos_protocol::host::PluginsReplaceRequest;
use harmos_protocol::host::PluginsReplaceResponse;
use harmos_protocol::host::PluginsRestartRequest;
use harmos_protocol::host::PluginsRestartResponse;
use harmos_protocol::host::PluginsSetRequest;
use harmos_protocol::host::PluginsSetResponse;
use harmos_protocol::host::PluginsWatchRequest;
use harmos_protocol::host::PluginsWatchResponse;
use harmos_protocol::host::SchemaDescription;
use harmos_protocol::host::SchemasListRequest;
use harmos_protocol::host::SchemasListResponse;
use harmos_protocol::host::StatusGetRequest;
use harmos_protocol::host::StatusGetResponse;
use harmos_protocol::host::StatusWatchRequest;
use harmos_protocol::host::StatusWatchResponse;
use harmos_protocol::host::StreamIdentity;
use harmos_protocol::host::StreamsListRequest;
use harmos_protocol::host::StreamsListResponse;
use harmos_protocol::host::TimeGetRequest;
use harmos_protocol::host::TimeGetResponse;
use harmos_protocol::host::TimeSleepRequest;
use harmos_protocol::host::TimeSleepResponse;
use harmos_protocol::host::access::CertificatesCreateRequest as AccessCertificatesCreateRequest;
use harmos_protocol::host::access::CertificatesCreateResponse as AccessCertificatesCreateResponse;
use harmos_protocol::host::access::ListenerGetRequest as AccessListenerGetRequest;
use harmos_protocol::host::access::ListenerGetResponse as AccessListenerGetResponse;
use harmos_protocol::host::access::ListenerSetRequest as AccessListenerSetRequest;
use harmos_protocol::host::access::ListenerSetResponse as AccessListenerSetResponse;
use harmos_protocol::host::access::ListenerValue as AccessListenerValue;
use harmos_protocol::host::access::PairingsCreateRequest as AccessPairingsCreateRequest;
use harmos_protocol::host::access::PairingsCreateResponse as AccessPairingsCreateResponse;
use harmos_protocol::host::access::PairingsDeleteRequest as AccessPairingsDeleteRequest;
use harmos_protocol::host::access::PairingsDeleteResponse as AccessPairingsDeleteResponse;
use harmos_protocol::host::access::PairingsListRequest as AccessPairingsListRequest;
use harmos_protocol::host::access::PairingsListResponse as AccessPairingsListResponse;
use harmos_protocol::host::plugins::SettingsListRequest as PluginsSettingsListRequest;
use harmos_protocol::host::plugins::SettingsListResponse as PluginsSettingsListResponse;
use harmos_protocol::host::plugins::SettingsSetRequest as PluginsSettingsSetRequest;
use harmos_protocol::host::plugins::SettingsSetResponse as PluginsSettingsSetResponse;
use harmos_protocol::host::status::DiagnosticsGetRequest as StatusDiagnosticsGetRequest;
use harmos_protocol::host::status::DiagnosticsGetResponse as StatusDiagnosticsGetResponse;
use harmos_protocol::host::status::LogsListRequest as StatusLogsListRequest;
use harmos_protocol::host::status::LogsListResponse as StatusLogsListResponse;
use harmos_protocol::host::status::LogsWatchRequest as StatusLogsWatchRequest;
use harmos_protocol::host::status::LogsWatchResponse as StatusLogsWatchResponse;
use harmos_protocol::host::status::TraceEventsListRequest as StatusTraceEventsListRequest;
use harmos_protocol::host::status::TraceEventsListResponse as StatusTraceEventsListResponse;
use harmos_protocol::host::status::TraceEventsWatchRequest as StatusTraceEventsWatchRequest;
use harmos_protocol::host::status::TraceEventsWatchResponse as StatusTraceEventsWatchResponse;
use harmos_protocol::model::DiscoveredOperation;
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::FileReference;
use harmos_protocol::model::InstanceState;
use harmos_protocol::model::PageRequest;
use harmos_protocol::model::Position;
use harmos_protocol::model::ReadCut;
use harmos_protocol::model::Receipt;
use harmos_protocol::model::SettingValues;
use harmos_protocol::model::Upload;
use prost_reflect::ReflectMessage;
use prost_types::Duration;
use prost_types::Timestamp;
use tonic::Status as RpcStatus;
use tonic::codegen::http::uri::PathAndQuery;
#[derive(Default)]
pub struct AggregatesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct AnalysesListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CyclesGetOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CyclesSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CyclesListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CyclesWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CyclesCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CyclesSealOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct BackendOverviewGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct BackendOverviewWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct BackendAppearanceOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct RecordSchemasGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct RecordSchemasListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsGetOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsStartOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsStopOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsCancelOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsParametersOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsStepsOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CasesCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CasesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CasesListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CasesWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CasesDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct CasesParametersOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ContentArtifactsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ContentArtifactsCreateOptions {
    pub scope: Option<ArtifactScope>,
    pub request_id: Option<String>,
    pub metadata: Option<Metadata>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ContentArtifactsListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ContentArtifactsWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ContentExecutionFilesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ContentExecutionFilesListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ContentExecutionFilesWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PlotsSnapshotsListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PlotsSnapshotsWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct DocumentsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct DocumentsWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceNotesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceNotesSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceNotesListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceNotesWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceNotesCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceHealthEventsListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceHealthEventsWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceRecordsListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceRecordsWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceSamplesListOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceSamplesWatchOptions {
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceTraceabilityGetOptions {
    pub page: Option<PageRequest>,
    pub at: Option<ReadCut>,
    pub entry: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceTraceabilityWatchOptions {
    pub page: Option<PageRequest>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceStatisticsOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct EvidenceArchiveOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsCommandsGetOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsCommandsCreateOptions {
    pub request_id: Option<String>,
    pub step_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsProgressGetOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsProgressWatchOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ExecutionsSchemasGetOptions {
    pub at: Option<Position>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesUploadsCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesUploadsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesUploadsResumeOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesUploadsAppendOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesUploadsPublishOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesUploadsDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesReadOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct FilesDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct MediaCapturesCreateOptions {
    pub input: Option<u32>,
    pub camera: Option<u32>,
    pub display: Option<u32>,
    pub maximum_seconds: Option<u32>,
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct MediaCapturesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct MediaCapturesWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct MediaCapturesStopOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PlotsSettingsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PlotsSettingsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsSecretsListOptions {
    pub name: Option<String>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsSecretsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsSecretsDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PromptDefinitionsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PromptDefinitionsListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PromptsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PromptsListOptions {
    pub cycle_id: Option<String>,
    pub execution_id: Option<String>,
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PromptsWatchOptions {
    pub cycle_id: Option<String>,
    pub execution_id: Option<String>,
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PromptsAnswersCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ReportsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ReportsCreateOptions {
    pub at: Option<ReadCut>,
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct ReportsPublishOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SecretsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SecretsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SecretsListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SecretsWatchOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SecretsDeleteOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SecretsExportOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SecretsImportOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SettingsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SettingsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SettingsSchemaOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StorageSourcesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StorageSourcesSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StreamSchemasGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StreamSchemasListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SuitesGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SuitesListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct AccessCertificatesCreateOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct AccessListenerGetOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct AccessListenerSetOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct AccessPairingsCreateOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct AccessPairingsListOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct AccessPairingsDeleteOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct InvocationsCreateOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct InvocationsGetOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct InvocationsCancelOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct InvocationsWatchOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct OperationsListOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct OperationsGetOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsSettingsListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsSettingsSetOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsGetOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsListOptions {
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsWatchOptions {
    pub name: Option<String>,
    pub at: Option<ReadCut>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsCreateOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsSetOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsDeleteOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsRestartOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct PluginsReplaceOptions {
    pub request_id: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct SchemasListOptions {
    pub schema: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StatusDiagnosticsGetOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StatusLogsListOptions {
    pub at: Option<Position>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StatusLogsWatchOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StatusTraceEventsListOptions {
    pub at: Option<Position>,
    pub page: Option<PageRequest>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StatusTraceEventsWatchOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StatusGetOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StatusWatchOptions {
    pub at: Option<Position>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct StreamsListOptions {
    pub schema: Option<String>,
    pub context: CallOptions,
}
#[derive(Default)]
pub struct TimeGetOptions {
    pub context: CallOptions,
}
#[derive(Default)]
pub struct TimeSleepOptions {
    pub context: CallOptions,
}
#[derive(Clone)]
pub struct Root {
    pub access: Access,
    pub aggregates: Aggregates,
    pub analyses: Analyses,
    pub backend: Backend,
    pub cases: Cases,
    pub content: Content,
    pub cycles: Cycles,
    pub documents: Documents,
    pub evidence: Evidence,
    pub executions: Executions,
    pub files: Files,
    pub invocations: Invocations,
    pub media: Media,
    pub operations: Operations,
    pub plots: Plots,
    pub plugins: Plugins,
    pub prompt_definitions: PromptDefinitions,
    pub prompts: Prompts,
    pub record_schemas: RecordSchemas,
    pub reports: Reports,
    pub schemas: Schemas,
    pub secrets: Secrets,
    pub settings: Settings,
    pub status: Status,
    pub storage: Storage,
    pub stream_schemas: StreamSchemas,
    pub streams: Streams,
    pub suites: Suites,
    pub time: Time,
}
impl Root {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            access: Access::new(client.clone()),
            aggregates: Aggregates::new(client.clone()),
            analyses: Analyses::new(client.clone()),
            backend: Backend::new(client.clone()),
            cases: Cases::new(client.clone()),
            content: Content::new(client.clone()),
            cycles: Cycles::new(client.clone()),
            documents: Documents::new(client.clone()),
            evidence: Evidence::new(client.clone()),
            executions: Executions::new(client.clone()),
            files: Files::new(client.clone()),
            invocations: Invocations::new(client.clone()),
            media: Media::new(client.clone()),
            operations: Operations::new(client.clone()),
            plots: Plots::new(client.clone()),
            plugins: Plugins::new(client.clone()),
            prompt_definitions: PromptDefinitions::new(client.clone()),
            prompts: Prompts::new(client.clone()),
            record_schemas: RecordSchemas::new(client.clone()),
            reports: Reports::new(client.clone()),
            schemas: Schemas::new(client.clone()),
            secrets: Secrets::new(client.clone()),
            settings: Settings::new(client.clone()),
            status: Status::new(client.clone()),
            storage: Storage::new(client.clone()),
            stream_schemas: StreamSchemas::new(client.clone()),
            streams: Streams::new(client.clone()),
            suites: Suites::new(client.clone()),
            time: Time::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct Access {
    pub certificates: AccessCertificates,
    pub listener: AccessListener,
    pub pairings: AccessPairings,
}
impl Access {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            certificates: AccessCertificates::new(client.clone()),
            listener: AccessListener::new(client.clone()),
            pairings: AccessPairings::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct AccessCertificates {
    client: ClientTransport,
}
impl AccessCertificates {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self) -> Result<String, RpcStatus> {
        self.create_with(AccessCertificatesCreateOptions::default()).await
    }
    pub async fn create_with(&self, options: AccessCertificatesCreateOptions) -> Result<String, RpcStatus> {
        let value = AccessCertificatesCreateRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AccessCertificatesCreateRequest, AccessCertificatesCreateResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Certificates/Create"),
                request,
                AccessCertificatesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.certificates.create", error))?;
        let response = response.into_inner();
        Ok(response.certificate_pem)
    }
}
#[derive(Clone)]
pub struct AccessListener {
    client: ClientTransport,
}
impl AccessListener {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<AccessListenerGetResponse, RpcStatus> {
        self.get_with(AccessListenerGetOptions::default()).await
    }
    pub async fn get_with(&self, options: AccessListenerGetOptions) -> Result<AccessListenerGetResponse, RpcStatus> {
        let value = AccessListenerGetRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AccessListenerGetRequest, AccessListenerGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Listener/Get"),
                request,
                AccessListenerGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.listener.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(&self, value: AccessListenerValue, expected_revision: Vec<u8>) -> Result<AccessListenerSetResponse, RpcStatus> {
        self.set_with(value, expected_revision, AccessListenerSetOptions::default()).await
    }
    pub async fn set_with(
        &self, value: AccessListenerValue, expected_revision: Vec<u8>, options: AccessListenerSetOptions,
    ) -> Result<AccessListenerSetResponse, RpcStatus> {
        let value = AccessListenerSetRequest { value: Some(value), expected_revision };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AccessListenerSetRequest, AccessListenerSetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Listener/Set"),
                request,
                AccessListenerSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.listener.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct AccessPairings {
    client: ClientTransport,
}
impl AccessPairings {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self, label: impl Into<String>) -> Result<AccessPairingsCreateResponse, RpcStatus> {
        self.create_with(label, AccessPairingsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, label: impl Into<String>, options: AccessPairingsCreateOptions,
    ) -> Result<AccessPairingsCreateResponse, RpcStatus> {
        let value = AccessPairingsCreateRequest { label: label.into() };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AccessPairingsCreateRequest, AccessPairingsCreateResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Pairings/Create"),
                request,
                AccessPairingsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.pairings.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self) -> Result<AccessPairingsListResponse, RpcStatus> {
        self.list_with(AccessPairingsListOptions::default()).await
    }
    pub async fn list_with(&self, options: AccessPairingsListOptions) -> Result<AccessPairingsListResponse, RpcStatus> {
        let value = AccessPairingsListRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AccessPairingsListRequest, AccessPairingsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Pairings/List"),
                request,
                AccessPairingsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.pairings.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn delete(&self, name: impl Into<String>) -> Result<AccessPairingsDeleteResponse, RpcStatus> {
        self.delete_with(name, AccessPairingsDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, name: impl Into<String>, options: AccessPairingsDeleteOptions,
    ) -> Result<AccessPairingsDeleteResponse, RpcStatus> {
        let value = AccessPairingsDeleteRequest { name: name.into() };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AccessPairingsDeleteRequest, AccessPairingsDeleteResponse>(
                PathAndQuery::from_static("/harmos.host.v1.access.Pairings/Delete"),
                request,
                AccessPairingsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("access.pairings.delete", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Aggregates {
    client: ClientTransport,
}
impl Aggregates {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, source: QuerySource, filters: Vec<QueryFilter>, group_by: Vec<String>, aggregates: Vec<QueryAggregate>, limit: u32,
    ) -> Result<AggregatesGetResponse, RpcStatus> {
        self.get_with(source, filters, group_by, aggregates, limit, AggregatesGetOptions::default()).await
    }
    pub async fn get_with(
        &self, source: QuerySource, filters: Vec<QueryFilter>, group_by: Vec<String>, aggregates: Vec<QueryAggregate>, limit: u32,
        options: AggregatesGetOptions,
    ) -> Result<AggregatesGetResponse, RpcStatus> {
        let value = AggregatesGetRequest {
            source: source as i32,
            filters,
            group_by: group_by.into(),
            aggregates,
            limit,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AggregatesGetRequest, AggregatesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Aggregates/Get"),
                request,
                AggregatesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("aggregates.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Analyses {
    client: ClientTransport,
}
impl Analyses {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<AnalysesListResponse, RpcStatus> {
        self.list_with(AnalysesListOptions::default()).await
    }
    pub async fn list_with(&self, options: AnalysesListOptions) -> Result<AnalysesListResponse, RpcStatus> {
        let value = AnalysesListRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<AnalysesListRequest, AnalysesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Analyses/List"),
                request,
                AnalysesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("analyses.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Backend {
    client: ClientTransport,
    pub overview: BackendOverview,
}
impl Backend {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            overview: BackendOverview::new(client.clone()),
            client,
        }
    }
    pub async fn appearance(&self) -> Result<BackendAppearanceResponse, RpcStatus> {
        self.appearance_with(BackendAppearanceOptions::default()).await
    }
    pub async fn appearance_with(&self, options: BackendAppearanceOptions) -> Result<BackendAppearanceResponse, RpcStatus> {
        let value = BackendAppearanceRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<BackendAppearanceRequest, BackendAppearanceResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Backend/Appearance"),
                request,
                BackendAppearanceResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("backend.appearance", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct BackendOverview {
    client: ClientTransport,
}
impl BackendOverview {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<BackendOverviewGetResponse, RpcStatus> {
        self.get_with(BackendOverviewGetOptions::default()).await
    }
    pub async fn get_with(&self, options: BackendOverviewGetOptions) -> Result<BackendOverviewGetResponse, RpcStatus> {
        let value = BackendOverviewGetRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<BackendOverviewGetRequest, BackendOverviewGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.backend.Overview/Get"),
                request,
                BackendOverviewGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("backend.overview.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self) -> Result<Watching<BackendOverviewWatchResponse>, RpcStatus> {
        self.watch_with(BackendOverviewWatchOptions::default()).await
    }
    pub async fn watch_with(&self, options: BackendOverviewWatchOptions) -> Result<Watching<BackendOverviewWatchResponse>, RpcStatus> {
        let value = BackendOverviewWatchRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<BackendOverviewWatchRequest, BackendOverviewWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.backend.Overview/Watch"),
                request,
                BackendOverviewWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("backend.overview.watch", error))?;
        let response = response.with_action("backend.overview.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Cases {
    client: ClientTransport,
}
impl Cases {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self, plan: SelectionPlan, validate_only: bool) -> Result<CasesCreateResponse, RpcStatus> {
        self.create_with(plan, validate_only, CasesCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, plan: SelectionPlan, validate_only: bool, options: CasesCreateOptions,
    ) -> Result<CasesCreateResponse, RpcStatus> {
        let value = CasesCreateRequest {
            plan: Some(plan),
            validate_only,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesCreateRequest, CasesCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Create"),
                request,
                CasesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn get(&self, name: impl Into<String>, parent: impl Into<String>) -> Result<CasesGetResponse, RpcStatus> {
        self.get_with(name, parent, CasesGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, options: CasesGetOptions,
    ) -> Result<CasesGetResponse, RpcStatus> {
        let value = CasesGetRequest {
            name: name.into(),
            parent: parent.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesGetRequest, CasesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Get"),
                request,
                CasesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<CasesListResponse, RpcStatus> {
        self.list_with(parent, CasesListOptions::default()).await
    }
    pub async fn list_with(&self, parent: impl Into<String>, options: CasesListOptions) -> Result<CasesListResponse, RpcStatus> {
        let value = CasesListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesListRequest, CasesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/List"),
                request,
                CasesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, parent: impl Into<String>) -> Result<Watching<CasesWatchResponse>, RpcStatus> {
        self.watch_with(parent, CasesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, parent: impl Into<String>, options: CasesWatchOptions,
    ) -> Result<Watching<CasesWatchResponse>, RpcStatus> {
        let value = CasesWatchRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<CasesWatchRequest, CasesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Watch"),
                request,
                CasesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.watch", error))?;
        let response = response.with_action("cases.watch");
        Ok(response)
    }
    pub async fn delete(&self, name: impl Into<String>, parent: impl Into<String>) -> Result<Receipt, RpcStatus> {
        self.delete_with(name, parent, CasesDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, name: impl Into<String>, parent: impl Into<String>, options: CasesDeleteOptions,
    ) -> Result<Receipt, RpcStatus> {
        let value = CasesDeleteRequest {
            name: name.into(),
            parent: parent.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesDeleteRequest, CasesDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Delete"),
                request,
                CasesDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.delete", error))?;
        let response = response.into_inner();
        response.receipt.ok_or_else(|| RpcStatus::data_loss("Read Cases.Delete: Missing Payload"))
    }
    pub async fn parameters(&self, name: impl Into<String>, parent: impl Into<String>) -> Result<CasesParametersResponse, RpcStatus> {
        self.parameters_with(name, parent, CasesParametersOptions::default()).await
    }
    pub async fn parameters_with(
        &self, name: impl Into<String>, parent: impl Into<String>, options: CasesParametersOptions,
    ) -> Result<CasesParametersResponse, RpcStatus> {
        let value = CasesParametersRequest {
            name: name.into(),
            parent: parent.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CasesParametersRequest, CasesParametersResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cases/Parameters"),
                request,
                CasesParametersResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cases.parameters", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Content {
    pub artifacts: ContentArtifacts,
    pub execution_files: ContentExecutionFiles,
}
impl Content {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            artifacts: ContentArtifacts::new(client.clone()),
            execution_files: ContentExecutionFiles::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct ContentArtifacts {
    client: ClientTransport,
}
impl ContentArtifacts {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<ContentArtifactsGetResponse, RpcStatus> {
        self.get_with(name, ContentArtifactsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, options: ContentArtifactsGetOptions,
    ) -> Result<ContentArtifactsGetResponse, RpcStatus> {
        let value = ContentArtifactsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ContentArtifactsGetRequest, ContentArtifactsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.Artifacts/Get"),
                request,
                ContentArtifactsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.artifacts.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn create(
        &self, file: FileReference, filename: impl Into<String>, content_type: impl Into<String>,
    ) -> Result<ContentArtifactsCreateResponse, RpcStatus> {
        self.create_with(file, filename, content_type, ContentArtifactsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, file: FileReference, filename: impl Into<String>, content_type: impl Into<String>, options: ContentArtifactsCreateOptions,
    ) -> Result<ContentArtifactsCreateResponse, RpcStatus> {
        let value = ContentArtifactsCreateRequest {
            file: Some(file),
            scope: options.scope,
            filename: filename.into(),
            content_type: content_type.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
            metadata: options.metadata,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ContentArtifactsCreateRequest, ContentArtifactsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.Artifacts/Create"),
                request,
                ContentArtifactsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.artifacts.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, scope: ArtifactScope) -> Result<ContentArtifactsListResponse, RpcStatus> {
        self.list_with(scope, ContentArtifactsListOptions::default()).await
    }
    pub async fn list_with(
        &self, scope: ArtifactScope, options: ContentArtifactsListOptions,
    ) -> Result<ContentArtifactsListResponse, RpcStatus> {
        let value = ContentArtifactsListRequest {
            scope: Some(scope),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ContentArtifactsListRequest, ContentArtifactsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.Artifacts/List"),
                request,
                ContentArtifactsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.artifacts.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, scope: ArtifactScope) -> Result<Watching<ContentArtifactsWatchResponse>, RpcStatus> {
        self.watch_with(scope, ContentArtifactsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, scope: ArtifactScope, options: ContentArtifactsWatchOptions,
    ) -> Result<Watching<ContentArtifactsWatchResponse>, RpcStatus> {
        let value = ContentArtifactsWatchRequest {
            scope: Some(scope),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<ContentArtifactsWatchRequest, ContentArtifactsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.Artifacts/Watch"),
                request,
                ContentArtifactsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.artifacts.watch", error))?;
        let response = response.with_action("content.artifacts.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct ContentExecutionFiles {
    client: ClientTransport,
}
impl ContentExecutionFiles {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, parent: impl Into<String>, name: impl Into<String>) -> Result<ContentExecutionFilesGetResponse, RpcStatus> {
        self.get_with(parent, name, ContentExecutionFilesGetOptions::default()).await
    }
    pub async fn get_with(
        &self, parent: impl Into<String>, name: impl Into<String>, options: ContentExecutionFilesGetOptions,
    ) -> Result<ContentExecutionFilesGetResponse, RpcStatus> {
        let value = ContentExecutionFilesGetRequest {
            parent: parent.into(),
            name: name.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ContentExecutionFilesGetRequest, ContentExecutionFilesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.ExecutionFiles/Get"),
                request,
                ContentExecutionFilesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.execution_files.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<ContentExecutionFilesListResponse, RpcStatus> {
        self.list_with(parent, ContentExecutionFilesListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, options: ContentExecutionFilesListOptions,
    ) -> Result<ContentExecutionFilesListResponse, RpcStatus> {
        let value = ContentExecutionFilesListRequest {
            parent: parent.into(),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ContentExecutionFilesListRequest, ContentExecutionFilesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.ExecutionFiles/List"),
                request,
                ContentExecutionFilesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.execution_files.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, parent: impl Into<String>) -> Result<Watching<ContentExecutionFilesWatchResponse>, RpcStatus> {
        self.watch_with(parent, ContentExecutionFilesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, parent: impl Into<String>, options: ContentExecutionFilesWatchOptions,
    ) -> Result<Watching<ContentExecutionFilesWatchResponse>, RpcStatus> {
        let value = ContentExecutionFilesWatchRequest {
            parent: parent.into(),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<ContentExecutionFilesWatchRequest, ContentExecutionFilesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.content.ExecutionFiles/Watch"),
                request,
                ContentExecutionFilesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("content.execution_files.watch", error))?;
        let response = response.with_action("content.execution_files.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Cycles {
    client: ClientTransport,
}
impl Cycles {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<CyclesGetResponse, RpcStatus> {
        self.get_with(name, CyclesGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: CyclesGetOptions) -> Result<CyclesGetResponse, RpcStatus> {
        let value = CyclesGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesGetRequest, CyclesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Get"),
                request,
                CyclesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: CycleWritable, expected_revision: Vec<u8>,
    ) -> Result<CyclesSetResponse, RpcStatus> {
        self.set_with(name, value, expected_revision, CyclesSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: CycleWritable, expected_revision: Vec<u8>, options: CyclesSetOptions,
    ) -> Result<CyclesSetResponse, RpcStatus> {
        let value = CyclesSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesSetRequest, CyclesSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Set"),
                request,
                CyclesSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, selection: CycleSelection, filter: CycleFilter, order: CycleOrder) -> Result<CyclesListResponse, RpcStatus> {
        self.list_with(selection, filter, order, CyclesListOptions::default()).await
    }
    pub async fn list_with(
        &self, selection: CycleSelection, filter: CycleFilter, order: CycleOrder, options: CyclesListOptions,
    ) -> Result<CyclesListResponse, RpcStatus> {
        let value = CyclesListRequest {
            selection: Some(selection),
            filter: Some(filter),
            order: Some(order),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesListRequest, CyclesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/List"),
                request,
                CyclesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(
        &self, selection: CycleSelection, filter: CycleFilter, order: CycleOrder,
    ) -> Result<Watching<CyclesWatchResponse>, RpcStatus> {
        self.watch_with(selection, filter, order, CyclesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, selection: CycleSelection, filter: CycleFilter, order: CycleOrder, options: CyclesWatchOptions,
    ) -> Result<Watching<CyclesWatchResponse>, RpcStatus> {
        let value = CyclesWatchRequest {
            selection: Some(selection),
            filter: Some(filter),
            order: Some(order),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<CyclesWatchRequest, CyclesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Watch"),
                request,
                CyclesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.watch", error))?;
        let response = response.with_action("cycles.watch");
        Ok(response)
    }
    pub async fn create(
        &self, origins: Vec<Origin>, metadata: Metadata, continue_automatically: bool, source_cycle: impl Into<String>,
    ) -> Result<CyclesCreateResponse, RpcStatus> {
        self.create_with(origins, metadata, continue_automatically, source_cycle, CyclesCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, origins: Vec<Origin>, metadata: Metadata, continue_automatically: bool, source_cycle: impl Into<String>,
        options: CyclesCreateOptions,
    ) -> Result<CyclesCreateResponse, RpcStatus> {
        let value = CyclesCreateRequest {
            origins,
            metadata: Some(metadata),
            continue_automatically,
            source_cycle: source_cycle.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesCreateRequest, CyclesCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Create"),
                request,
                CyclesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn seal(&self, name: impl Into<String>) -> Result<CyclesSealResponse, RpcStatus> {
        self.seal_with(name, CyclesSealOptions::default()).await
    }
    pub async fn seal_with(&self, name: impl Into<String>, options: CyclesSealOptions) -> Result<CyclesSealResponse, RpcStatus> {
        let value = CyclesSealRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<CyclesSealRequest, CyclesSealResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Cycles/Seal"),
                request,
                CyclesSealResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("cycles.seal", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Documents {
    client: ClientTransport,
}
impl Documents {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, scope: DocumentScope, document_id: impl Into<String>, structure_only: bool, card_ids: Vec<String>,
        page_token: impl Into<String>, max_bytes: u64, max_frames: u32,
    ) -> Result<DocumentsGetResponse, RpcStatus> {
        self.get_with(scope, document_id, structure_only, card_ids, page_token, max_bytes, max_frames, DocumentsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, scope: DocumentScope, document_id: impl Into<String>, structure_only: bool, card_ids: Vec<String>,
        page_token: impl Into<String>, max_bytes: u64, max_frames: u32, options: DocumentsGetOptions,
    ) -> Result<DocumentsGetResponse, RpcStatus> {
        let value = DocumentsGetRequest {
            scope: Some(scope),
            document_id: document_id.into(),
            structure_only,
            card_ids: card_ids.into(),
            page_token: page_token.into(),
            max_bytes,
            max_frames,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<DocumentsGetRequest, DocumentsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Documents/Get"),
                request,
                DocumentsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("documents.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(
        &self, scope: DocumentScope, document_id: impl Into<String>, structure_only: bool, card_ids: Vec<String>,
        page_token: impl Into<String>, max_bytes: u64, max_frames: u32,
    ) -> Result<Watching<DocumentsWatchResponse>, RpcStatus> {
        self.watch_with(scope, document_id, structure_only, card_ids, page_token, max_bytes, max_frames, DocumentsWatchOptions::default())
            .await
    }
    pub async fn watch_with(
        &self, scope: DocumentScope, document_id: impl Into<String>, structure_only: bool, card_ids: Vec<String>,
        page_token: impl Into<String>, max_bytes: u64, max_frames: u32, options: DocumentsWatchOptions,
    ) -> Result<Watching<DocumentsWatchResponse>, RpcStatus> {
        let value = DocumentsWatchRequest {
            scope: Some(scope),
            document_id: document_id.into(),
            structure_only,
            card_ids: card_ids.into(),
            page_token: page_token.into(),
            max_bytes,
            max_frames,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<DocumentsWatchRequest, DocumentsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Documents/Watch"),
                request,
                DocumentsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("documents.watch", error))?;
        let response = response.with_action("documents.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Evidence {
    client: ClientTransport,
    pub health_events: EvidenceHealthEvents,
    pub notes: EvidenceNotes,
    pub records: EvidenceRecords,
    pub samples: EvidenceSamples,
    pub traceability: EvidenceTraceability,
}
impl Evidence {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            health_events: EvidenceHealthEvents::new(client.clone()),
            notes: EvidenceNotes::new(client.clone()),
            records: EvidenceRecords::new(client.clone()),
            samples: EvidenceSamples::new(client.clone()),
            traceability: EvidenceTraceability::new(client.clone()),
            client,
        }
    }
    pub async fn statistics(
        &self, scope: EvidenceScope, stream: impl Into<String>, versions: Vec<u32>, between: Window, outputs: Vec<Output>,
        field: impl Into<String>, step: impl Into<String>,
    ) -> Result<EvidenceStatisticsResponse, RpcStatus> {
        self.statistics_with(scope, stream, versions, between, outputs, field, step, EvidenceStatisticsOptions::default()).await
    }
    pub async fn statistics_with(
        &self, scope: EvidenceScope, stream: impl Into<String>, versions: Vec<u32>, between: Window, outputs: Vec<Output>,
        field: impl Into<String>, step: impl Into<String>, options: EvidenceStatisticsOptions,
    ) -> Result<EvidenceStatisticsResponse, RpcStatus> {
        let value = EvidenceStatisticsRequest {
            scope: Some(scope),
            stream: stream.into(),
            versions,
            between: Some(between),
            outputs,
            field: field.into(),
            step: step.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceStatisticsRequest, EvidenceStatisticsResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Evidence/Statistics"),
                request,
                EvidenceStatisticsResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.statistics", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn archive(&self, subject: EvidenceSubject) -> Result<EvidenceArchiveResponse, RpcStatus> {
        self.archive_with(subject, EvidenceArchiveOptions::default()).await
    }
    pub async fn archive_with(
        &self, subject: EvidenceSubject, options: EvidenceArchiveOptions,
    ) -> Result<EvidenceArchiveResponse, RpcStatus> {
        let value = EvidenceArchiveRequest { subject: Some(subject), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceArchiveRequest, EvidenceArchiveResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Evidence/Archive"),
                request,
                EvidenceArchiveResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.archive", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceHealthEvents {
    client: ClientTransport,
}
impl EvidenceHealthEvents {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self, selection: EvidenceAnnotationSelection) -> Result<EvidenceHealthEventsListResponse, RpcStatus> {
        self.list_with(selection, EvidenceHealthEventsListOptions::default()).await
    }
    pub async fn list_with(
        &self, selection: EvidenceAnnotationSelection, options: EvidenceHealthEventsListOptions,
    ) -> Result<EvidenceHealthEventsListResponse, RpcStatus> {
        let value = EvidenceHealthEventsListRequest {
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceHealthEventsListRequest, EvidenceHealthEventsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.HealthEvents/List"),
                request,
                EvidenceHealthEventsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.health_events.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, selection: EvidenceAnnotationSelection) -> Result<Watching<EvidenceHealthEventsWatchResponse>, RpcStatus> {
        self.watch_with(selection, EvidenceHealthEventsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, selection: EvidenceAnnotationSelection, options: EvidenceHealthEventsWatchOptions,
    ) -> Result<Watching<EvidenceHealthEventsWatchResponse>, RpcStatus> {
        let value = EvidenceHealthEventsWatchRequest {
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<EvidenceHealthEventsWatchRequest, EvidenceHealthEventsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.HealthEvents/Watch"),
                request,
                EvidenceHealthEventsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.health_events.watch", error))?;
        let response = response.with_action("evidence.health_events.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceNotes {
    client: ClientTransport,
}
impl EvidenceNotes {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<EvidenceNotesGetResponse, RpcStatus> {
        self.get_with(name, EvidenceNotesGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: EvidenceNotesGetOptions) -> Result<EvidenceNotesGetResponse, RpcStatus> {
        let value = EvidenceNotesGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceNotesGetRequest, EvidenceNotesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/Get"),
                request,
                EvidenceNotesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: EvidenceNoteWritable, expected_revision: Vec<u8>,
    ) -> Result<EvidenceNotesSetResponse, RpcStatus> {
        self.set_with(name, value, expected_revision, EvidenceNotesSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: EvidenceNoteWritable, expected_revision: Vec<u8>, options: EvidenceNotesSetOptions,
    ) -> Result<EvidenceNotesSetResponse, RpcStatus> {
        let value = EvidenceNotesSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceNotesSetRequest, EvidenceNotesSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/Set"),
                request,
                EvidenceNotesSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, selection: EvidenceAnnotationSelection) -> Result<EvidenceNotesListResponse, RpcStatus> {
        self.list_with(selection, EvidenceNotesListOptions::default()).await
    }
    pub async fn list_with(
        &self, selection: EvidenceAnnotationSelection, options: EvidenceNotesListOptions,
    ) -> Result<EvidenceNotesListResponse, RpcStatus> {
        let value = EvidenceNotesListRequest {
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceNotesListRequest, EvidenceNotesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/List"),
                request,
                EvidenceNotesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, selection: EvidenceAnnotationSelection) -> Result<Watching<EvidenceNotesWatchResponse>, RpcStatus> {
        self.watch_with(selection, EvidenceNotesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, selection: EvidenceAnnotationSelection, options: EvidenceNotesWatchOptions,
    ) -> Result<Watching<EvidenceNotesWatchResponse>, RpcStatus> {
        let value = EvidenceNotesWatchRequest {
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<EvidenceNotesWatchRequest, EvidenceNotesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/Watch"),
                request,
                EvidenceNotesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.watch", error))?;
        let response = response.with_action("evidence.notes.watch");
        Ok(response)
    }
    pub async fn create(
        &self, owner: EvidenceAnnotationOwner, value: EvidenceNoteWritable,
    ) -> Result<EvidenceNotesCreateResponse, RpcStatus> {
        self.create_with(owner, value, EvidenceNotesCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, owner: EvidenceAnnotationOwner, value: EvidenceNoteWritable, options: EvidenceNotesCreateOptions,
    ) -> Result<EvidenceNotesCreateResponse, RpcStatus> {
        let value = EvidenceNotesCreateRequest {
            owner: Some(owner),
            value: Some(value),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceNotesCreateRequest, EvidenceNotesCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Notes/Create"),
                request,
                EvidenceNotesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.notes.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceRecords {
    client: ClientTransport,
}
impl EvidenceRecords {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(
        &self, scope: EvidenceRecordScope, record_type: impl Into<String>, versions: Vec<u32>, between: Window, step: impl Into<String>,
        selection: EvidenceCapturedRecordSelection,
    ) -> Result<EvidenceRecordsListResponse, RpcStatus> {
        self.list_with(scope, record_type, versions, between, step, selection, EvidenceRecordsListOptions::default()).await
    }
    pub async fn list_with(
        &self, scope: EvidenceRecordScope, record_type: impl Into<String>, versions: Vec<u32>, between: Window, step: impl Into<String>,
        selection: EvidenceCapturedRecordSelection, options: EvidenceRecordsListOptions,
    ) -> Result<EvidenceRecordsListResponse, RpcStatus> {
        let value = EvidenceRecordsListRequest {
            scope: Some(scope),
            record_type: record_type.into(),
            versions,
            between: Some(between),
            step: step.into(),
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceRecordsListRequest, EvidenceRecordsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Records/List"),
                request,
                EvidenceRecordsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.records.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(
        &self, scope: EvidenceRecordScope, record_type: impl Into<String>, versions: Vec<u32>, between: Window, step: impl Into<String>,
        selection: EvidenceCapturedRecordSelection,
    ) -> Result<Watching<EvidenceRecordsWatchResponse>, RpcStatus> {
        self.watch_with(scope, record_type, versions, between, step, selection, EvidenceRecordsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, scope: EvidenceRecordScope, record_type: impl Into<String>, versions: Vec<u32>, between: Window, step: impl Into<String>,
        selection: EvidenceCapturedRecordSelection, options: EvidenceRecordsWatchOptions,
    ) -> Result<Watching<EvidenceRecordsWatchResponse>, RpcStatus> {
        let value = EvidenceRecordsWatchRequest {
            scope: Some(scope),
            record_type: record_type.into(),
            versions,
            between: Some(between),
            step: step.into(),
            selection: Some(selection),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<EvidenceRecordsWatchRequest, EvidenceRecordsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Records/Watch"),
                request,
                EvidenceRecordsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.records.watch", error))?;
        let response = response.with_action("evidence.records.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceSamples {
    client: ClientTransport,
}
impl EvidenceSamples {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(
        &self, scope: EvidenceScope, streams: Vec<String>, versions: Vec<u32>, fields: Vec<String>, sources: Vec<String>,
        signals: Vec<String>, between: Window, bucket: EvidenceBucketRequest, latest: bool,
    ) -> Result<EvidenceSamplesListResponse, RpcStatus> {
        self.list_with(scope, streams, versions, fields, sources, signals, between, bucket, latest, EvidenceSamplesListOptions::default())
            .await
    }
    pub async fn list_with(
        &self, scope: EvidenceScope, streams: Vec<String>, versions: Vec<u32>, fields: Vec<String>, sources: Vec<String>,
        signals: Vec<String>, between: Window, bucket: EvidenceBucketRequest, latest: bool, options: EvidenceSamplesListOptions,
    ) -> Result<EvidenceSamplesListResponse, RpcStatus> {
        let value = EvidenceSamplesListRequest {
            scope: Some(scope),
            streams: streams.into(),
            versions,
            fields: fields.into(),
            sources: sources.into(),
            signals: signals.into(),
            between: Some(between),
            bucket: Some(bucket),
            latest,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceSamplesListRequest, EvidenceSamplesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Samples/List"),
                request,
                EvidenceSamplesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.samples.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(
        &self, scope: EvidenceScope, streams: Vec<String>, versions: Vec<u32>, fields: Vec<String>, sources: Vec<String>,
        signals: Vec<String>, between: Window, bucket: EvidenceBucketRequest, latest: bool,
    ) -> Result<Watching<EvidenceSamplesWatchResponse>, RpcStatus> {
        self.watch_with(scope, streams, versions, fields, sources, signals, between, bucket, latest, EvidenceSamplesWatchOptions::default())
            .await
    }
    pub async fn watch_with(
        &self, scope: EvidenceScope, streams: Vec<String>, versions: Vec<u32>, fields: Vec<String>, sources: Vec<String>,
        signals: Vec<String>, between: Window, bucket: EvidenceBucketRequest, latest: bool, options: EvidenceSamplesWatchOptions,
    ) -> Result<Watching<EvidenceSamplesWatchResponse>, RpcStatus> {
        let value = EvidenceSamplesWatchRequest {
            scope: Some(scope),
            streams: streams.into(),
            versions,
            fields: fields.into(),
            sources: sources.into(),
            signals: signals.into(),
            between: Some(between),
            bucket: Some(bucket),
            latest,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<EvidenceSamplesWatchRequest, EvidenceSamplesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Samples/Watch"),
                request,
                EvidenceSamplesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.samples.watch", error))?;
        let response = response.with_action("evidence.samples.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct EvidenceTraceability {
    client: ClientTransport,
}
impl EvidenceTraceability {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, scope: EvidenceScope) -> Result<EvidenceTraceabilityGetResponse, RpcStatus> {
        self.get_with(scope, EvidenceTraceabilityGetOptions::default()).await
    }
    pub async fn get_with(
        &self, scope: EvidenceScope, options: EvidenceTraceabilityGetOptions,
    ) -> Result<EvidenceTraceabilityGetResponse, RpcStatus> {
        let value = EvidenceTraceabilityGetRequest {
            scope: Some(scope),
            page: options.page,
            at: options.at,
            entry: options.entry,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<EvidenceTraceabilityGetRequest, EvidenceTraceabilityGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Traceability/Get"),
                request,
                EvidenceTraceabilityGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.traceability.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, scope: EvidenceScope) -> Result<Watching<EvidenceTraceabilityWatchResponse>, RpcStatus> {
        self.watch_with(scope, EvidenceTraceabilityWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, scope: EvidenceScope, options: EvidenceTraceabilityWatchOptions,
    ) -> Result<Watching<EvidenceTraceabilityWatchResponse>, RpcStatus> {
        let value = EvidenceTraceabilityWatchRequest {
            scope: Some(scope),
            page: options.page,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<EvidenceTraceabilityWatchRequest, EvidenceTraceabilityWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.evidence.Traceability/Watch"),
                request,
                EvidenceTraceabilityWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("evidence.traceability.watch", error))?;
        let response = response.with_action("evidence.traceability.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Executions {
    client: ClientTransport,
    pub commands: ExecutionsCommands,
    pub progress: ExecutionsProgress,
    pub schemas: ExecutionsSchemas,
}
impl Executions {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            commands: ExecutionsCommands::new(client.clone()),
            progress: ExecutionsProgress::new(client.clone()),
            schemas: ExecutionsSchemas::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<ExecutionsGetResponse, RpcStatus> {
        self.get_with(name, ExecutionsGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: ExecutionsGetOptions) -> Result<ExecutionsGetResponse, RpcStatus> {
        let value = ExecutionsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsGetRequest, ExecutionsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Get"),
                request,
                ExecutionsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: ExecutionWritable, expected_revision: Vec<u8>,
    ) -> Result<ExecutionsSetResponse, RpcStatus> {
        self.set_with(name, value, expected_revision, ExecutionsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: ExecutionWritable, expected_revision: Vec<u8>, options: ExecutionsSetOptions,
    ) -> Result<ExecutionsSetResponse, RpcStatus> {
        let value = ExecutionsSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsSetRequest, ExecutionsSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Set"),
                request,
                ExecutionsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(
        &self, parent: impl Into<String>, selection: ExecutionSelection, filter: ExecutionFilter, order: ExecutionOrder,
    ) -> Result<ExecutionsListResponse, RpcStatus> {
        self.list_with(parent, selection, filter, order, ExecutionsListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, selection: ExecutionSelection, filter: ExecutionFilter, order: ExecutionOrder,
        options: ExecutionsListOptions,
    ) -> Result<ExecutionsListResponse, RpcStatus> {
        let value = ExecutionsListRequest {
            parent: parent.into(),
            selection: Some(selection),
            filter: Some(filter),
            order: Some(order),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsListRequest, ExecutionsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/List"),
                request,
                ExecutionsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(
        &self, parent: impl Into<String>, selection: ExecutionSelection, filter: ExecutionFilter, order: ExecutionOrder,
    ) -> Result<Watching<ExecutionsWatchResponse>, RpcStatus> {
        self.watch_with(parent, selection, filter, order, ExecutionsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, parent: impl Into<String>, selection: ExecutionSelection, filter: ExecutionFilter, order: ExecutionOrder,
        options: ExecutionsWatchOptions,
    ) -> Result<Watching<ExecutionsWatchResponse>, RpcStatus> {
        let value = ExecutionsWatchRequest {
            parent: parent.into(),
            selection: Some(selection),
            filter: Some(filter),
            order: Some(order),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<ExecutionsWatchRequest, ExecutionsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Watch"),
                request,
                ExecutionsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.watch", error))?;
        let response = response.with_action("executions.watch");
        Ok(response)
    }
    pub async fn create(
        &self, parent: impl Into<String>, executions: Vec<ExecutionBinding>,
    ) -> Result<ExecutionsCreateResponse, RpcStatus> {
        self.create_with(parent, executions, ExecutionsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, parent: impl Into<String>, executions: Vec<ExecutionBinding>, options: ExecutionsCreateOptions,
    ) -> Result<ExecutionsCreateResponse, RpcStatus> {
        let value = ExecutionsCreateRequest {
            parent: parent.into(),
            executions,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsCreateRequest, ExecutionsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Create"),
                request,
                ExecutionsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn start(&self, name: impl Into<String>, mode: ExecutionMode) -> Result<ExecutionsStartResponse, RpcStatus> {
        self.start_with(name, mode, ExecutionsStartOptions::default()).await
    }
    pub async fn start_with(
        &self, name: impl Into<String>, mode: ExecutionMode, options: ExecutionsStartOptions,
    ) -> Result<ExecutionsStartResponse, RpcStatus> {
        let value = ExecutionsStartRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
            mode: mode as i32,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsStartRequest, ExecutionsStartResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Start"),
                request,
                ExecutionsStartResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.start", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn stop(&self, name: impl Into<String>) -> Result<ExecutionsStopResponse, RpcStatus> {
        self.stop_with(name, ExecutionsStopOptions::default()).await
    }
    pub async fn stop_with(&self, name: impl Into<String>, options: ExecutionsStopOptions) -> Result<ExecutionsStopResponse, RpcStatus> {
        let value = ExecutionsStopRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsStopRequest, ExecutionsStopResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Stop"),
                request,
                ExecutionsStopResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.stop", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn cancel(&self, name: impl Into<String>) -> Result<ExecutionsCancelResponse, RpcStatus> {
        self.cancel_with(name, ExecutionsCancelOptions::default()).await
    }
    pub async fn cancel_with(
        &self, name: impl Into<String>, options: ExecutionsCancelOptions,
    ) -> Result<ExecutionsCancelResponse, RpcStatus> {
        let value = ExecutionsCancelRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsCancelRequest, ExecutionsCancelResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Cancel"),
                request,
                ExecutionsCancelResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.cancel", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn delete(&self, name: impl Into<String>) -> Result<ExecutionsDeleteResponse, RpcStatus> {
        self.delete_with(name, ExecutionsDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, name: impl Into<String>, options: ExecutionsDeleteOptions,
    ) -> Result<ExecutionsDeleteResponse, RpcStatus> {
        let value = ExecutionsDeleteRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsDeleteRequest, ExecutionsDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Delete"),
                request,
                ExecutionsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.delete", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn parameters(&self, name: impl Into<String>, set: ExecutionParameterSet) -> Result<ExecutionsParametersResponse, RpcStatus> {
        self.parameters_with(name, set, ExecutionsParametersOptions::default()).await
    }
    pub async fn parameters_with(
        &self, name: impl Into<String>, set: ExecutionParameterSet, options: ExecutionsParametersOptions,
    ) -> Result<ExecutionsParametersResponse, RpcStatus> {
        let value = ExecutionsParametersRequest {
            name: name.into(),
            set: set as i32,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsParametersRequest, ExecutionsParametersResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Parameters"),
                request,
                ExecutionsParametersResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.parameters", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn steps(&self, name: impl Into<String>) -> Result<ExecutionsStepsResponse, RpcStatus> {
        self.steps_with(name, ExecutionsStepsOptions::default()).await
    }
    pub async fn steps_with(&self, name: impl Into<String>, options: ExecutionsStepsOptions) -> Result<ExecutionsStepsResponse, RpcStatus> {
        let value = ExecutionsStepsRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsStepsRequest, ExecutionsStepsResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Executions/Steps"),
                request,
                ExecutionsStepsResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.steps", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct ExecutionsCommands {
    client: ClientTransport,
}
impl ExecutionsCommands {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>, parent: impl Into<String>) -> Result<ExecutionsCommandsGetResponse, RpcStatus> {
        self.get_with(name, parent, ExecutionsCommandsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, options: ExecutionsCommandsGetOptions,
    ) -> Result<ExecutionsCommandsGetResponse, RpcStatus> {
        let value = ExecutionsCommandsGetRequest {
            name: name.into(),
            parent: parent.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsCommandsGetRequest, ExecutionsCommandsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Commands/Get"),
                request,
                ExecutionsCommandsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.commands.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn create(
        &self, parent: impl Into<String>, command_id: impl Into<String>, action: ExecutionsExecutionCommandAction,
    ) -> Result<ExecutionsCommandsCreateResponse, RpcStatus> {
        self.create_with(parent, command_id, action, ExecutionsCommandsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, parent: impl Into<String>, command_id: impl Into<String>, action: ExecutionsExecutionCommandAction,
        options: ExecutionsCommandsCreateOptions,
    ) -> Result<ExecutionsCommandsCreateResponse, RpcStatus> {
        let value = ExecutionsCommandsCreateRequest {
            parent: parent.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
            command_id: command_id.into(),
            action: action as i32,
            step_id: options.step_id,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsCommandsCreateRequest, ExecutionsCommandsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Commands/Create"),
                request,
                ExecutionsCommandsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.commands.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct ExecutionsProgress {
    client: ClientTransport,
}
impl ExecutionsProgress {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<ExecutionsProgressGetResponse, RpcStatus> {
        self.get_with(name, ExecutionsProgressGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, options: ExecutionsProgressGetOptions,
    ) -> Result<ExecutionsProgressGetResponse, RpcStatus> {
        let value = ExecutionsProgressGetRequest { name: Some(name.into()), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsProgressGetRequest, ExecutionsProgressGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Progress/Get"),
                request,
                ExecutionsProgressGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.progress.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, name: impl Into<String>) -> Result<Watching<ExecutionsProgressWatchResponse>, RpcStatus> {
        self.watch_with(name, ExecutionsProgressWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, name: impl Into<String>, options: ExecutionsProgressWatchOptions,
    ) -> Result<Watching<ExecutionsProgressWatchResponse>, RpcStatus> {
        let value = ExecutionsProgressWatchRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<ExecutionsProgressWatchRequest, ExecutionsProgressWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Progress/Watch"),
                request,
                ExecutionsProgressWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.progress.watch", error))?;
        let response = response.with_action("executions.progress.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct ExecutionsSchemas {
    client: ClientTransport,
}
impl ExecutionsSchemas {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<ExecutionsSchemasGetResponse, RpcStatus> {
        self.get_with(name, ExecutionsSchemasGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, options: ExecutionsSchemasGetOptions,
    ) -> Result<ExecutionsSchemasGetResponse, RpcStatus> {
        let value = ExecutionsSchemasGetRequest {
            name: name.into(),
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ExecutionsSchemasGetRequest, ExecutionsSchemasGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.executions.Schemas/Get"),
                request,
                ExecutionsSchemasGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("executions.schemas.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Files {
    client: ClientTransport,
    pub uploads: FilesUploads,
}
impl Files {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            uploads: FilesUploads::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, file: FileReference) -> Result<FilesGetResponse, RpcStatus> {
        self.get_with(file, FilesGetOptions::default()).await
    }
    pub async fn get_with(&self, file: FileReference, options: FilesGetOptions) -> Result<FilesGetResponse, RpcStatus> {
        let value = FilesGetRequest { file: Some(file), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesGetRequest, FilesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Files/Get"),
                request,
                FilesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn read(&self, file: FileReference, offset: u64, max_bytes: u64) -> Result<FilesReadResponse, RpcStatus> {
        self.read_with(file, offset, max_bytes, FilesReadOptions::default()).await
    }
    pub async fn read_with(
        &self, file: FileReference, offset: u64, max_bytes: u64, options: FilesReadOptions,
    ) -> Result<FilesReadResponse, RpcStatus> {
        let value = FilesReadRequest {
            file: Some(file),
            offset,
            max_bytes,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesReadRequest, FilesReadResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Files/Read"),
                request,
                FilesReadResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.read", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, file: FileReference, offset: u64) -> Result<Watching<FilesWatchResponse>, RpcStatus> {
        self.watch_with(file, offset, FilesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, file: FileReference, offset: u64, options: FilesWatchOptions,
    ) -> Result<Watching<FilesWatchResponse>, RpcStatus> {
        let value = FilesWatchRequest { file: Some(file), offset, at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<FilesWatchRequest, FilesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Files/Watch"),
                request,
                FilesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.watch", error))?;
        let response = response.with_action("files.watch");
        Ok(response)
    }
    pub async fn delete(&self, file: FileReference) -> Result<Receipt, RpcStatus> {
        self.delete_with(file, FilesDeleteOptions::default()).await
    }
    pub async fn delete_with(&self, file: FileReference, options: FilesDeleteOptions) -> Result<Receipt, RpcStatus> {
        let value = FilesDeleteRequest {
            file: Some(file),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesDeleteRequest, FilesDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Files/Delete"),
                request,
                FilesDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.delete", error))?;
        let response = response.into_inner();
        response.receipt.ok_or_else(|| RpcStatus::data_loss("Read Files.Delete: Missing Payload"))
    }
}
#[derive(Clone)]
pub struct FilesUploads {
    client: ClientTransport,
}
impl FilesUploads {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self, metadata: FileMetadata) -> Result<FilesUploadsCreateResponse, RpcStatus> {
        self.create_with(metadata, FilesUploadsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, metadata: FileMetadata, options: FilesUploadsCreateOptions,
    ) -> Result<FilesUploadsCreateResponse, RpcStatus> {
        let value = FilesUploadsCreateRequest {
            metadata: Some(metadata),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesUploadsCreateRequest, FilesUploadsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Create"),
                request,
                FilesUploadsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn get(&self, upload: Upload) -> Result<FilesUploadsGetResponse, RpcStatus> {
        self.get_with(upload, FilesUploadsGetOptions::default()).await
    }
    pub async fn get_with(&self, upload: Upload, options: FilesUploadsGetOptions) -> Result<FilesUploadsGetResponse, RpcStatus> {
        let value = FilesUploadsGetRequest { upload: Some(upload), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesUploadsGetRequest, FilesUploadsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Get"),
                request,
                FilesUploadsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn resume(&self, upload: Upload) -> Result<FilesUploadsResumeResponse, RpcStatus> {
        self.resume_with(upload, FilesUploadsResumeOptions::default()).await
    }
    pub async fn resume_with(&self, upload: Upload, options: FilesUploadsResumeOptions) -> Result<FilesUploadsResumeResponse, RpcStatus> {
        let value = FilesUploadsResumeRequest {
            upload: Some(upload),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesUploadsResumeRequest, FilesUploadsResumeResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Resume"),
                request,
                FilesUploadsResumeResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.resume", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn append(&self, upload: Upload, offset: u64, data: Vec<u8>) -> Result<FilesUploadsAppendResponse, RpcStatus> {
        self.append_with(upload, offset, data, FilesUploadsAppendOptions::default()).await
    }
    pub async fn append_with(
        &self, upload: Upload, offset: u64, data: Vec<u8>, options: FilesUploadsAppendOptions,
    ) -> Result<FilesUploadsAppendResponse, RpcStatus> {
        let value = FilesUploadsAppendRequest {
            upload: Some(upload),
            offset,
            data,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesUploadsAppendRequest, FilesUploadsAppendResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Append"),
                request,
                FilesUploadsAppendResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.append", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn publish(&self, upload: Upload, length: u64, sha256: Vec<u8>) -> Result<FilesUploadsPublishResponse, RpcStatus> {
        self.publish_with(upload, length, sha256, FilesUploadsPublishOptions::default()).await
    }
    pub async fn publish_with(
        &self, upload: Upload, length: u64, sha256: Vec<u8>, options: FilesUploadsPublishOptions,
    ) -> Result<FilesUploadsPublishResponse, RpcStatus> {
        let value = FilesUploadsPublishRequest {
            upload: Some(upload),
            length,
            sha256,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesUploadsPublishRequest, FilesUploadsPublishResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Publish"),
                request,
                FilesUploadsPublishResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.publish", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn delete(&self, upload: Upload) -> Result<FilesUploadsDeleteResponse, RpcStatus> {
        self.delete_with(upload, FilesUploadsDeleteOptions::default()).await
    }
    pub async fn delete_with(&self, upload: Upload, options: FilesUploadsDeleteOptions) -> Result<FilesUploadsDeleteResponse, RpcStatus> {
        let value = FilesUploadsDeleteRequest {
            upload: Some(upload),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<FilesUploadsDeleteRequest, FilesUploadsDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.files.Uploads/Delete"),
                request,
                FilesUploadsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("files.uploads.delete", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Invocations {
    client: ClientTransport,
}
impl Invocations {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(&self, operation: impl Into<String>, fingerprint: Vec<u8>, payload: Vec<u8>) -> Result<Vec<u8>, RpcStatus> {
        self.create_with(operation, fingerprint, payload, InvocationsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, operation: impl Into<String>, fingerprint: Vec<u8>, payload: Vec<u8>, options: InvocationsCreateOptions,
    ) -> Result<Vec<u8>, RpcStatus> {
        let value = InvocationsCreateRequest {
            operation: operation.into(),
            fingerprint,
            payload,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<InvocationsCreateRequest, InvocationsCreateResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Invocations/Create"),
                request,
                InvocationsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("invocations.create", error))?;
        let response = response.into_inner();
        Ok(response.payload)
    }
    pub async fn get(&self, instance: u64) -> Result<InstanceState, RpcStatus> {
        self.get_with(instance, InvocationsGetOptions::default()).await
    }
    pub async fn get_with(&self, instance: u64, options: InvocationsGetOptions) -> Result<InstanceState, RpcStatus> {
        let value = InvocationsGetRequest { instance };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<InvocationsGetRequest, InvocationsGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Invocations/Get"),
                request,
                InvocationsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("invocations.get", error))?;
        let response = response.into_inner();
        response.invocation.ok_or_else(|| RpcStatus::data_loss("Read Invocations.Get: Missing Payload"))
    }
    pub async fn cancel(&self, instance: u64, reason: impl Into<String>) -> Result<InstanceState, RpcStatus> {
        self.cancel_with(instance, reason, InvocationsCancelOptions::default()).await
    }
    pub async fn cancel_with(
        &self, instance: u64, reason: impl Into<String>, options: InvocationsCancelOptions,
    ) -> Result<InstanceState, RpcStatus> {
        let value = InvocationsCancelRequest { instance, reason: reason.into() };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<InvocationsCancelRequest, InvocationsCancelResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Invocations/Cancel"),
                request,
                InvocationsCancelResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("invocations.cancel", error))?;
        let response = response.into_inner();
        response.invocation.ok_or_else(|| RpcStatus::data_loss("Read Invocations.Cancel: Missing Payload"))
    }
    pub async fn watch(&self, instance: u64) -> Result<Watching<InstanceState>, RpcStatus> {
        self.watch_with(instance, InvocationsWatchOptions::default()).await
    }
    pub async fn watch_with(&self, instance: u64, options: InvocationsWatchOptions) -> Result<Watching<InstanceState>, RpcStatus> {
        let value = InvocationsWatchRequest { instance };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<InvocationsWatchRequest, InvocationsWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Invocations/Watch"),
                request,
                InvocationsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("invocations.watch", error))?;
        let response = response.with_action("invocations.watch");
        Ok(response.map(|response| response.invocation.ok_or_else(|| RpcStatus::data_loss("Read Invocations.Watch: Missing Payload"))))
    }
}
#[derive(Clone)]
pub struct Media {
    pub captures: MediaCaptures,
}
impl Media {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            captures: MediaCaptures::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct MediaCaptures {
    client: ClientTransport,
}
impl MediaCaptures {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(
        &self, scope: ArtifactScope, kind: MediaCaptureKind, audio: MediaRecordingAudio,
    ) -> Result<MediaCapturesCreateResponse, RpcStatus> {
        self.create_with(scope, kind, audio, MediaCapturesCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, scope: ArtifactScope, kind: MediaCaptureKind, audio: MediaRecordingAudio, options: MediaCapturesCreateOptions,
    ) -> Result<MediaCapturesCreateResponse, RpcStatus> {
        let value = MediaCapturesCreateRequest {
            scope: Some(scope),
            kind: kind as i32,
            input: options.input,
            camera: options.camera,
            display: options.display,
            audio: Some(audio),
            maximum_seconds: options.maximum_seconds,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<MediaCapturesCreateRequest, MediaCapturesCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.media.Captures/Create"),
                request,
                MediaCapturesCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("media.captures.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<MediaCapturesGetResponse, RpcStatus> {
        self.get_with(name, MediaCapturesGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: MediaCapturesGetOptions) -> Result<MediaCapturesGetResponse, RpcStatus> {
        let value = MediaCapturesGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<MediaCapturesGetRequest, MediaCapturesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.media.Captures/Get"),
                request,
                MediaCapturesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("media.captures.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, name: impl Into<String>) -> Result<Watching<MediaCapturesWatchResponse>, RpcStatus> {
        self.watch_with(name, MediaCapturesWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, name: impl Into<String>, options: MediaCapturesWatchOptions,
    ) -> Result<Watching<MediaCapturesWatchResponse>, RpcStatus> {
        let value = MediaCapturesWatchRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<MediaCapturesWatchRequest, MediaCapturesWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.media.Captures/Watch"),
                request,
                MediaCapturesWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("media.captures.watch", error))?;
        let response = response.with_action("media.captures.watch");
        Ok(response)
    }
    pub async fn stop(&self, name: impl Into<String>) -> Result<MediaCapturesStopResponse, RpcStatus> {
        self.stop_with(name, MediaCapturesStopOptions::default()).await
    }
    pub async fn stop_with(
        &self, name: impl Into<String>, options: MediaCapturesStopOptions,
    ) -> Result<MediaCapturesStopResponse, RpcStatus> {
        let value = MediaCapturesStopRequest {
            name: name.into(),
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<MediaCapturesStopRequest, MediaCapturesStopResponse>(
                PathAndQuery::from_static("/dokime.host.v1.media.Captures/Stop"),
                request,
                MediaCapturesStopResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("media.captures.stop", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Operations {
    client: ClientTransport,
}
impl Operations {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<Vec<DiscoveredOperation>, RpcStatus> {
        self.list_with(OperationsListOptions::default()).await
    }
    pub async fn list_with(&self, options: OperationsListOptions) -> Result<Vec<DiscoveredOperation>, RpcStatus> {
        let value = OperationsListRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<OperationsListRequest, OperationsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Operations/List"),
                request,
                OperationsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("operations.list", error))?;
        let response = response.into_inner();
        Ok(response.operations)
    }
    pub async fn get(&self, id: impl Into<String>) -> Result<DiscoveredOperation, RpcStatus> {
        self.get_with(id, OperationsGetOptions::default()).await
    }
    pub async fn get_with(&self, id: impl Into<String>, options: OperationsGetOptions) -> Result<DiscoveredOperation, RpcStatus> {
        let value = OperationsGetRequest { id: id.into() };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<OperationsGetRequest, OperationsGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Operations/Get"),
                request,
                OperationsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("operations.get", error))?;
        let response = response.into_inner();
        response.operation.ok_or_else(|| RpcStatus::data_loss("Read Operations.Get: Missing Payload"))
    }
}
#[derive(Clone)]
pub struct Plots {
    pub settings: PlotsSettings,
    pub snapshots: PlotsSnapshots,
}
impl Plots {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            settings: PlotsSettings::new(client.clone()),
            snapshots: PlotsSnapshots::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct PlotsSettings {
    client: ClientTransport,
}
impl PlotsSettings {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, scope: DocumentScope, document_id: impl Into<String>, plot_id: impl Into<String>,
    ) -> Result<PlotsSettingsGetResponse, RpcStatus> {
        self.get_with(scope, document_id, plot_id, PlotsSettingsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, scope: DocumentScope, document_id: impl Into<String>, plot_id: impl Into<String>, options: PlotsSettingsGetOptions,
    ) -> Result<PlotsSettingsGetResponse, RpcStatus> {
        let value = PlotsSettingsGetRequest {
            scope: Some(scope),
            document_id: document_id.into(),
            plot_id: plot_id.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PlotsSettingsGetRequest, PlotsSettingsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plots.Settings/Get"),
                request,
                PlotsSettingsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plots.settings.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, scope: DocumentScope, document_id: impl Into<String>, plot_id: impl Into<String>, value: PlotsPlotSettingsValue,
        expected_revision: Vec<u8>,
    ) -> Result<PlotsSettingsSetResponse, RpcStatus> {
        self.set_with(scope, document_id, plot_id, value, expected_revision, PlotsSettingsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, scope: DocumentScope, document_id: impl Into<String>, plot_id: impl Into<String>, value: PlotsPlotSettingsValue,
        expected_revision: Vec<u8>, options: PlotsSettingsSetOptions,
    ) -> Result<PlotsSettingsSetResponse, RpcStatus> {
        let value = PlotsSettingsSetRequest {
            scope: Some(scope),
            document_id: document_id.into(),
            plot_id: plot_id.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PlotsSettingsSetRequest, PlotsSettingsSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plots.Settings/Set"),
                request,
                PlotsSettingsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plots.settings.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct PlotsSnapshots {
    client: ClientTransport,
}
impl PlotsSnapshots {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self, requests: Vec<PlotsPlotRequest>) -> Result<PlotsSnapshotsListResponse, RpcStatus> {
        self.list_with(requests, PlotsSnapshotsListOptions::default()).await
    }
    pub async fn list_with(
        &self, requests: Vec<PlotsPlotRequest>, options: PlotsSnapshotsListOptions,
    ) -> Result<PlotsSnapshotsListResponse, RpcStatus> {
        let value = PlotsSnapshotsListRequest {
            requests,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PlotsSnapshotsListRequest, PlotsSnapshotsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plots.Snapshots/List"),
                request,
                PlotsSnapshotsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plots.snapshots.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, requests: Vec<PlotsPlotRequest>) -> Result<Watching<PlotsSnapshotsWatchResponse>, RpcStatus> {
        self.watch_with(requests, PlotsSnapshotsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, requests: Vec<PlotsPlotRequest>, options: PlotsSnapshotsWatchOptions,
    ) -> Result<Watching<PlotsSnapshotsWatchResponse>, RpcStatus> {
        let value = PlotsSnapshotsWatchRequest {
            requests,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<PlotsSnapshotsWatchRequest, PlotsSnapshotsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plots.Snapshots/Watch"),
                request,
                PlotsSnapshotsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plots.snapshots.watch", error))?;
        let response = response.with_action("plots.snapshots.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Plugins {
    client: ClientTransport,
    pub secrets: PluginsSecrets,
    pub settings: PluginsSettings,
}
impl Plugins {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            secrets: PluginsSecrets::new(client.clone()),
            settings: PluginsSettings::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<PluginsGetResponse, RpcStatus> {
        self.get_with(name, PluginsGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: PluginsGetOptions) -> Result<PluginsGetResponse, RpcStatus> {
        let value = PluginsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsGetRequest, PluginsGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Get"),
                request,
                PluginsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self) -> Result<PluginsListResponse, RpcStatus> {
        self.list_with(PluginsListOptions::default()).await
    }
    pub async fn list_with(&self, options: PluginsListOptions) -> Result<PluginsListResponse, RpcStatus> {
        let value = PluginsListRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsListRequest, PluginsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/List"),
                request,
                PluginsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self) -> Result<Watching<PluginsWatchResponse>, RpcStatus> {
        self.watch_with(PluginsWatchOptions::default()).await
    }
    pub async fn watch_with(&self, options: PluginsWatchOptions) -> Result<Watching<PluginsWatchResponse>, RpcStatus> {
        let value = PluginsWatchRequest { name: options.name, at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<PluginsWatchRequest, PluginsWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Watch"),
                request,
                PluginsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.watch", error))?;
        let response = response.with_action("plugins.watch");
        Ok(response)
    }
    pub async fn create(
        &self, name: impl Into<String>, placement: PluginPlacement, value: PluginValue,
    ) -> Result<PluginsCreateResponse, RpcStatus> {
        self.create_with(name, placement, value, PluginsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, name: impl Into<String>, placement: PluginPlacement, value: PluginValue, options: PluginsCreateOptions,
    ) -> Result<PluginsCreateResponse, RpcStatus> {
        let value = PluginsCreateRequest {
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
            name: name.into(),
            placement: Some(placement),
            value: Some(value),
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsCreateRequest, PluginsCreateResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Create"),
                request,
                PluginsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: PluginValue, expected_revision: Vec<u8>,
    ) -> Result<PluginsSetResponse, RpcStatus> {
        self.set_with(name, value, expected_revision, PluginsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: PluginValue, expected_revision: Vec<u8>, options: PluginsSetOptions,
    ) -> Result<PluginsSetResponse, RpcStatus> {
        let value = PluginsSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsSetRequest, PluginsSetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Set"),
                request,
                PluginsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn delete(&self, name: impl Into<String>, expected_revision: Vec<u8>) -> Result<Receipt, RpcStatus> {
        self.delete_with(name, expected_revision, PluginsDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, name: impl Into<String>, expected_revision: Vec<u8>, options: PluginsDeleteOptions,
    ) -> Result<Receipt, RpcStatus> {
        let value = PluginsDeleteRequest { name: name.into(), expected_revision };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsDeleteRequest, PluginsDeleteResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Delete"),
                request,
                PluginsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.delete", error))?;
        let response = response.into_inner();
        response.receipt.ok_or_else(|| RpcStatus::data_loss("Read Plugins.Delete: Missing Payload"))
    }
    pub async fn restart(&self, name: impl Into<String>, expected_revision: Vec<u8>) -> Result<PluginEntry, RpcStatus> {
        self.restart_with(name, expected_revision, PluginsRestartOptions::default()).await
    }
    pub async fn restart_with(
        &self, name: impl Into<String>, expected_revision: Vec<u8>, options: PluginsRestartOptions,
    ) -> Result<PluginEntry, RpcStatus> {
        let value = PluginsRestartRequest { name: name.into(), expected_revision };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsRestartRequest, PluginsRestartResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Restart"),
                request,
                PluginsRestartResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.restart", error))?;
        let response = response.into_inner();
        response.entry.ok_or_else(|| RpcStatus::data_loss("Read Plugins.Restart: Missing Payload"))
    }
    pub async fn replace(
        &self, name: impl Into<String>, placement: PluginPlacement, expected_revision: Vec<u8>,
    ) -> Result<PluginsReplaceResponse, RpcStatus> {
        self.replace_with(name, placement, expected_revision, PluginsReplaceOptions::default()).await
    }
    pub async fn replace_with(
        &self, name: impl Into<String>, placement: PluginPlacement, expected_revision: Vec<u8>, options: PluginsReplaceOptions,
    ) -> Result<PluginsReplaceResponse, RpcStatus> {
        let value = PluginsReplaceRequest {
            name: name.into(),
            placement: Some(placement),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsReplaceRequest, PluginsReplaceResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Plugins/Replace"),
                request,
                PluginsReplaceResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.replace", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct PluginsSecrets {
    client: ClientTransport,
}
impl PluginsSecrets {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<PluginsSecretsListResponse, RpcStatus> {
        self.list_with(PluginsSecretsListOptions::default()).await
    }
    pub async fn list_with(&self, options: PluginsSecretsListOptions) -> Result<PluginsSecretsListResponse, RpcStatus> {
        let value = PluginsSecretsListRequest { name: options.name, at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsSecretsListRequest, PluginsSecretsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plugins.Secrets/List"),
                request,
                PluginsSecretsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.secrets.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, bindings: Vec<PluginsSecretBindingValue>, expected_revision: Vec<u8>,
    ) -> Result<PluginsSecretsSetResponse, RpcStatus> {
        self.set_with(name, bindings, expected_revision, PluginsSecretsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, bindings: Vec<PluginsSecretBindingValue>, expected_revision: Vec<u8>,
        options: PluginsSecretsSetOptions,
    ) -> Result<PluginsSecretsSetResponse, RpcStatus> {
        let value = PluginsSecretsSetRequest {
            name: name.into(),
            bindings,
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsSecretsSetRequest, PluginsSecretsSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plugins.Secrets/Set"),
                request,
                PluginsSecretsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.secrets.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn delete(
        &self, name: impl Into<String>, identifiers: Vec<String>, expected_revision: Vec<u8>,
    ) -> Result<Receipt, RpcStatus> {
        self.delete_with(name, identifiers, expected_revision, PluginsSecretsDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, name: impl Into<String>, identifiers: Vec<String>, expected_revision: Vec<u8>, options: PluginsSecretsDeleteOptions,
    ) -> Result<Receipt, RpcStatus> {
        let value = PluginsSecretsDeleteRequest {
            name: name.into(),
            identifiers: identifiers.into(),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsSecretsDeleteRequest, PluginsSecretsDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.plugins.Secrets/Delete"),
                request,
                PluginsSecretsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.secrets.delete", error))?;
        let response = response.into_inner();
        response.receipt.ok_or_else(|| RpcStatus::data_loss("Read Secrets.Delete: Missing Payload"))
    }
}
#[derive(Clone)]
pub struct PluginsSettings {
    client: ClientTransport,
}
impl PluginsSettings {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self, name: impl Into<String>) -> Result<PluginsSettingsListResponse, RpcStatus> {
        self.list_with(name, PluginsSettingsListOptions::default()).await
    }
    pub async fn list_with(
        &self, name: impl Into<String>, options: PluginsSettingsListOptions,
    ) -> Result<PluginsSettingsListResponse, RpcStatus> {
        let value = PluginsSettingsListRequest { name: Some(name.into()), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsSettingsListRequest, PluginsSettingsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.plugins.Settings/List"),
                request,
                PluginsSettingsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.settings.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: SettingValues, expected_revision: Vec<u8>,
    ) -> Result<PluginsSettingsSetResponse, RpcStatus> {
        self.set_with(name, value, expected_revision, PluginsSettingsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: SettingValues, expected_revision: Vec<u8>, options: PluginsSettingsSetOptions,
    ) -> Result<PluginsSettingsSetResponse, RpcStatus> {
        let value = PluginsSettingsSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PluginsSettingsSetRequest, PluginsSettingsSetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.plugins.Settings/Set"),
                request,
                PluginsSettingsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("plugins.settings.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct PromptDefinitions {
    client: ClientTransport,
}
impl PromptDefinitions {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32,
    ) -> Result<PromptDefinitionsGetResponse, RpcStatus> {
        self.get_with(name, parent, schema_version, PromptDefinitionsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32, options: PromptDefinitionsGetOptions,
    ) -> Result<PromptDefinitionsGetResponse, RpcStatus> {
        let value = PromptDefinitionsGetRequest {
            name: name.into(),
            parent: parent.into(),
            schema_version,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PromptDefinitionsGetRequest, PromptDefinitionsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.PromptDefinitions/Get"),
                request,
                PromptDefinitionsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompt_definitions.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<PromptDefinitionsListResponse, RpcStatus> {
        self.list_with(parent, PromptDefinitionsListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, options: PromptDefinitionsListOptions,
    ) -> Result<PromptDefinitionsListResponse, RpcStatus> {
        let value = PromptDefinitionsListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PromptDefinitionsListRequest, PromptDefinitionsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.PromptDefinitions/List"),
                request,
                PromptDefinitionsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompt_definitions.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Prompts {
    client: ClientTransport,
    pub answers: PromptsAnswers,
}
impl Prompts {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            answers: PromptsAnswers::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<PromptsGetResponse, RpcStatus> {
        self.get_with(name, PromptsGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: PromptsGetOptions) -> Result<PromptsGetResponse, RpcStatus> {
        let value = PromptsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PromptsGetRequest, PromptsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Prompts/Get"),
                request,
                PromptsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompts.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>, pending_only: bool) -> Result<PromptsListResponse, RpcStatus> {
        self.list_with(parent, pending_only, PromptsListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, pending_only: bool, options: PromptsListOptions,
    ) -> Result<PromptsListResponse, RpcStatus> {
        let value = PromptsListRequest {
            parent: parent.into(),
            cycle_id: options.cycle_id,
            execution_id: options.execution_id,
            pending_only,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PromptsListRequest, PromptsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Prompts/List"),
                request,
                PromptsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompts.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, parent: impl Into<String>, pending_only: bool) -> Result<Watching<PromptsWatchResponse>, RpcStatus> {
        self.watch_with(parent, pending_only, PromptsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, parent: impl Into<String>, pending_only: bool, options: PromptsWatchOptions,
    ) -> Result<Watching<PromptsWatchResponse>, RpcStatus> {
        let value = PromptsWatchRequest {
            parent: parent.into(),
            cycle_id: options.cycle_id,
            execution_id: options.execution_id,
            pending_only,
            at: options.at,
            page: options.page,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<PromptsWatchRequest, PromptsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Prompts/Watch"),
                request,
                PromptsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompts.watch", error))?;
        let response = response.with_action("prompts.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct PromptsAnswers {
    client: ClientTransport,
}
impl PromptsAnswers {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn create(
        &self, parent: impl Into<String>, action: impl Into<String>, values: MapValue, accepted: bool,
    ) -> Result<PromptsAnswersCreateResponse, RpcStatus> {
        self.create_with(parent, action, values, accepted, PromptsAnswersCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, parent: impl Into<String>, action: impl Into<String>, values: MapValue, accepted: bool, options: PromptsAnswersCreateOptions,
    ) -> Result<PromptsAnswersCreateResponse, RpcStatus> {
        let value = PromptsAnswersCreateRequest {
            parent: parent.into(),
            action: action.into(),
            values: Some(values),
            accepted,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<PromptsAnswersCreateRequest, PromptsAnswersCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.prompts.Answers/Create"),
                request,
                PromptsAnswersCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("prompts.answers.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct RecordSchemas {
    client: ClientTransport,
}
impl RecordSchemas {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32,
    ) -> Result<RecordSchemasGetResponse, RpcStatus> {
        self.get_with(name, parent, schema_version, RecordSchemasGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32, options: RecordSchemasGetOptions,
    ) -> Result<RecordSchemasGetResponse, RpcStatus> {
        let value = RecordSchemasGetRequest {
            name: name.into(),
            parent: parent.into(),
            schema_version,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<RecordSchemasGetRequest, RecordSchemasGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.RecordSchemas/Get"),
                request,
                RecordSchemasGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("record_schemas.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<RecordSchemasListResponse, RpcStatus> {
        self.list_with(parent, RecordSchemasListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, options: RecordSchemasListOptions,
    ) -> Result<RecordSchemasListResponse, RpcStatus> {
        let value = RecordSchemasListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<RecordSchemasListRequest, RecordSchemasListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.RecordSchemas/List"),
                request,
                RecordSchemasListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("record_schemas.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Reports {
    client: ClientTransport,
}
impl Reports {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, subject: ReportSubject, name: impl Into<String>) -> Result<ReportsGetResponse, RpcStatus> {
        self.get_with(subject, name, ReportsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, subject: ReportSubject, name: impl Into<String>, options: ReportsGetOptions,
    ) -> Result<ReportsGetResponse, RpcStatus> {
        let value = ReportsGetRequest {
            subject: Some(subject),
            name: name.into(),
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ReportsGetRequest, ReportsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Reports/Get"),
                request,
                ReportsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("reports.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn create(&self, subject: ReportSubject, document_id: impl Into<String>) -> Result<ReportsCreateResponse, RpcStatus> {
        self.create_with(subject, document_id, ReportsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, subject: ReportSubject, document_id: impl Into<String>, options: ReportsCreateOptions,
    ) -> Result<ReportsCreateResponse, RpcStatus> {
        let value = ReportsCreateRequest {
            subject: Some(subject),
            document_id: document_id.into(),
            at: options.at,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ReportsCreateRequest, ReportsCreateResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Reports/Create"),
                request,
                ReportsCreateResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("reports.create", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn publish(
        &self, name: impl Into<String>, subject: ReportSubject, expected_revision: Vec<u8>,
    ) -> Result<ReportsPublishResponse, RpcStatus> {
        self.publish_with(name, subject, expected_revision, ReportsPublishOptions::default()).await
    }
    pub async fn publish_with(
        &self, name: impl Into<String>, subject: ReportSubject, expected_revision: Vec<u8>, options: ReportsPublishOptions,
    ) -> Result<ReportsPublishResponse, RpcStatus> {
        let value = ReportsPublishRequest {
            name: name.into(),
            subject: Some(subject),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<ReportsPublishRequest, ReportsPublishResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Reports/Publish"),
                request,
                ReportsPublishResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("reports.publish", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Schemas {
    client: ClientTransport,
}
impl Schemas {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<Vec<SchemaDescription>, RpcStatus> {
        self.list_with(SchemasListOptions::default()).await
    }
    pub async fn list_with(&self, options: SchemasListOptions) -> Result<Vec<SchemaDescription>, RpcStatus> {
        let value = SchemasListRequest { schema: options.schema };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SchemasListRequest, SchemasListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Schemas/List"),
                request,
                SchemasListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("schemas.list", error))?;
        let response = response.into_inner();
        Ok(response.schemas)
    }
}
#[derive(Clone)]
pub struct Secrets {
    client: ClientTransport,
}
impl Secrets {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>) -> Result<SecretsGetResponse, RpcStatus> {
        self.get_with(name, SecretsGetOptions::default()).await
    }
    pub async fn get_with(&self, name: impl Into<String>, options: SecretsGetOptions) -> Result<SecretsGetResponse, RpcStatus> {
        let value = SecretsGetRequest { name: name.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsGetRequest, SecretsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Get"),
                request,
                SecretsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(
        &self, name: impl Into<String>, value: SecretWritable, expected_revision: Vec<u8>,
    ) -> Result<SecretsSetResponse, RpcStatus> {
        self.set_with(name, value, expected_revision, SecretsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, name: impl Into<String>, value: SecretWritable, expected_revision: Vec<u8>, options: SecretsSetOptions,
    ) -> Result<SecretsSetResponse, RpcStatus> {
        let value = SecretsSetRequest {
            name: name.into(),
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsSetRequest, SecretsSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Set"),
                request,
                SecretsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self) -> Result<SecretsListResponse, RpcStatus> {
        self.list_with(SecretsListOptions::default()).await
    }
    pub async fn list_with(&self, options: SecretsListOptions) -> Result<SecretsListResponse, RpcStatus> {
        let value = SecretsListRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsListRequest, SecretsListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/List"),
                request,
                SecretsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self) -> Result<Watching<SecretsWatchResponse>, RpcStatus> {
        self.watch_with(SecretsWatchOptions::default()).await
    }
    pub async fn watch_with(&self, options: SecretsWatchOptions) -> Result<Watching<SecretsWatchResponse>, RpcStatus> {
        let value = SecretsWatchRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<SecretsWatchRequest, SecretsWatchResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Watch"),
                request,
                SecretsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.watch", error))?;
        let response = response.with_action("secrets.watch");
        Ok(response)
    }
    pub async fn delete(&self, names: Vec<String>, expected_revision: Vec<u8>) -> Result<SecretsDeleteResponse, RpcStatus> {
        self.delete_with(names, expected_revision, SecretsDeleteOptions::default()).await
    }
    pub async fn delete_with(
        &self, names: Vec<String>, expected_revision: Vec<u8>, options: SecretsDeleteOptions,
    ) -> Result<SecretsDeleteResponse, RpcStatus> {
        let value = SecretsDeleteRequest {
            names: names.into(),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsDeleteRequest, SecretsDeleteResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Delete"),
                request,
                SecretsDeleteResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.delete", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn export(&self, names: Vec<String>, generation: u64) -> Result<SecretsExportResponse, RpcStatus> {
        self.export_with(names, generation, SecretsExportOptions::default()).await
    }
    pub async fn export_with(
        &self, names: Vec<String>, generation: u64, options: SecretsExportOptions,
    ) -> Result<SecretsExportResponse, RpcStatus> {
        let value = SecretsExportRequest {
            names: names.into(),
            generation,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsExportRequest, SecretsExportResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Export"),
                request,
                SecretsExportResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.export", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn import(
        &self, data: Vec<u8>, choices: Vec<SecretImportChoice>, generation: u64, validate_only: bool,
    ) -> Result<SecretsImportResponse, RpcStatus> {
        self.import_with(data, choices, generation, validate_only, SecretsImportOptions::default()).await
    }
    pub async fn import_with(
        &self, data: Vec<u8>, choices: Vec<SecretImportChoice>, generation: u64, validate_only: bool, options: SecretsImportOptions,
    ) -> Result<SecretsImportResponse, RpcStatus> {
        let value = SecretsImportRequest {
            data,
            choices,
            generation,
            validate_only,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SecretsImportRequest, SecretsImportResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Secrets/Import"),
                request,
                SecretsImportResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("secrets.import", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Settings {
    client: ClientTransport,
}
impl Settings {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<SettingsGetResponse, RpcStatus> {
        self.get_with(SettingsGetOptions::default()).await
    }
    pub async fn get_with(&self, options: SettingsGetOptions) -> Result<SettingsGetResponse, RpcStatus> {
        let value = SettingsGetRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SettingsGetRequest, SettingsGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Settings/Get"),
                request,
                SettingsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("settings.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(&self, value: ApplicationSettings, expected_revision: Vec<u8>) -> Result<SettingsSetResponse, RpcStatus> {
        self.set_with(value, expected_revision, SettingsSetOptions::default()).await
    }
    pub async fn set_with(
        &self, value: ApplicationSettings, expected_revision: Vec<u8>, options: SettingsSetOptions,
    ) -> Result<SettingsSetResponse, RpcStatus> {
        let value = SettingsSetRequest {
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SettingsSetRequest, SettingsSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Settings/Set"),
                request,
                SettingsSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("settings.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn schema(&self) -> Result<SettingsSchemaResponse, RpcStatus> {
        self.schema_with(SettingsSchemaOptions::default()).await
    }
    pub async fn schema_with(&self, options: SettingsSchemaOptions) -> Result<SettingsSchemaResponse, RpcStatus> {
        let value = SettingsSchemaRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SettingsSchemaRequest, SettingsSchemaResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Settings/Schema"),
                request,
                SettingsSchemaResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("settings.schema", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Status {
    client: ClientTransport,
    pub diagnostics: StatusDiagnostics,
    pub logs: StatusLogs,
    pub trace_events: StatusTraceEvents,
}
impl Status {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            diagnostics: StatusDiagnostics::new(client.clone()),
            logs: StatusLogs::new(client.clone()),
            trace_events: StatusTraceEvents::new(client.clone()),
            client,
        }
    }
    pub async fn get(&self) -> Result<StatusGetResponse, RpcStatus> {
        self.get_with(StatusGetOptions::default()).await
    }
    pub async fn get_with(&self, options: StatusGetOptions) -> Result<StatusGetResponse, RpcStatus> {
        let value = StatusGetRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StatusGetRequest, StatusGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Status/Get"),
                request,
                StatusGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self) -> Result<Watching<StatusWatchResponse>, RpcStatus> {
        self.watch_with(StatusWatchOptions::default()).await
    }
    pub async fn watch_with(&self, options: StatusWatchOptions) -> Result<Watching<StatusWatchResponse>, RpcStatus> {
        let value = StatusWatchRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<StatusWatchRequest, StatusWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Status/Watch"),
                request,
                StatusWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.watch", error))?;
        let response = response.with_action("status.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct StatusDiagnostics {
    client: ClientTransport,
}
impl StatusDiagnostics {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self, name: impl Into<String>, offset: u64, max_bytes: u64) -> Result<StatusDiagnosticsGetResponse, RpcStatus> {
        self.get_with(name, offset, max_bytes, StatusDiagnosticsGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, offset: u64, max_bytes: u64, options: StatusDiagnosticsGetOptions,
    ) -> Result<StatusDiagnosticsGetResponse, RpcStatus> {
        let value = StatusDiagnosticsGetRequest { name: name.into(), offset, max_bytes };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StatusDiagnosticsGetRequest, StatusDiagnosticsGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.Diagnostics/Get"),
                request,
                StatusDiagnosticsGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.diagnostics.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct StatusLogs {
    client: ClientTransport,
}
impl StatusLogs {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<StatusLogsListResponse, RpcStatus> {
        self.list_with(StatusLogsListOptions::default()).await
    }
    pub async fn list_with(&self, options: StatusLogsListOptions) -> Result<StatusLogsListResponse, RpcStatus> {
        let value = StatusLogsListRequest { at: options.at, page: options.page };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StatusLogsListRequest, StatusLogsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.Logs/List"),
                request,
                StatusLogsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.logs.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, max_bytes: u64, max_events: u32) -> Result<Watching<StatusLogsWatchResponse>, RpcStatus> {
        self.watch_with(max_bytes, max_events, StatusLogsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, max_bytes: u64, max_events: u32, options: StatusLogsWatchOptions,
    ) -> Result<Watching<StatusLogsWatchResponse>, RpcStatus> {
        let value = StatusLogsWatchRequest { at: options.at, max_bytes, max_events };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<StatusLogsWatchRequest, StatusLogsWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.Logs/Watch"),
                request,
                StatusLogsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.logs.watch", error))?;
        let response = response.with_action("status.logs.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct StatusTraceEvents {
    client: ClientTransport,
}
impl StatusTraceEvents {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<StatusTraceEventsListResponse, RpcStatus> {
        self.list_with(StatusTraceEventsListOptions::default()).await
    }
    pub async fn list_with(&self, options: StatusTraceEventsListOptions) -> Result<StatusTraceEventsListResponse, RpcStatus> {
        let value = StatusTraceEventsListRequest { at: options.at, page: options.page };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StatusTraceEventsListRequest, StatusTraceEventsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.TraceEvents/List"),
                request,
                StatusTraceEventsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.trace_events.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn watch(&self, max_bytes: u64, max_events: u32) -> Result<Watching<StatusTraceEventsWatchResponse>, RpcStatus> {
        self.watch_with(max_bytes, max_events, StatusTraceEventsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, max_bytes: u64, max_events: u32, options: StatusTraceEventsWatchOptions,
    ) -> Result<Watching<StatusTraceEventsWatchResponse>, RpcStatus> {
        let value = StatusTraceEventsWatchRequest { at: options.at, max_bytes, max_events };
        let request = options.context.request(value);
        let response = self
            .client
            .watch_known::<StatusTraceEventsWatchRequest, StatusTraceEventsWatchResponse>(
                PathAndQuery::from_static("/harmos.host.v1.status.TraceEvents/Watch"),
                request,
                StatusTraceEventsWatchResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("status.trace_events.watch", error))?;
        let response = response.with_action("status.trace_events.watch");
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Storage {
    pub sources: StorageSources,
}
impl Storage {
    pub fn new(client: ClientTransport) -> Self {
        Self {
            sources: StorageSources::new(client.clone()),
        }
    }
}
#[derive(Clone)]
pub struct StorageSources {
    client: ClientTransport,
}
impl StorageSources {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<StorageSourcesGetResponse, RpcStatus> {
        self.get_with(StorageSourcesGetOptions::default()).await
    }
    pub async fn get_with(&self, options: StorageSourcesGetOptions) -> Result<StorageSourcesGetResponse, RpcStatus> {
        let value = StorageSourcesGetRequest { at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StorageSourcesGetRequest, StorageSourcesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.storage.Sources/Get"),
                request,
                StorageSourcesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("storage.sources.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn set(&self, value: StorageStorageSources, expected_revision: Vec<u8>) -> Result<StorageSourcesSetResponse, RpcStatus> {
        self.set_with(value, expected_revision, StorageSourcesSetOptions::default()).await
    }
    pub async fn set_with(
        &self, value: StorageStorageSources, expected_revision: Vec<u8>, options: StorageSourcesSetOptions,
    ) -> Result<StorageSourcesSetResponse, RpcStatus> {
        let value = StorageSourcesSetRequest {
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => request_id()?,
            },
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StorageSourcesSetRequest, StorageSourcesSetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.storage.Sources/Set"),
                request,
                StorageSourcesSetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("storage.sources.set", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct StreamSchemas {
    client: ClientTransport,
}
impl StreamSchemas {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32,
    ) -> Result<StreamSchemasGetResponse, RpcStatus> {
        self.get_with(name, parent, schema_version, StreamSchemasGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32, options: StreamSchemasGetOptions,
    ) -> Result<StreamSchemasGetResponse, RpcStatus> {
        let value = StreamSchemasGetRequest {
            name: name.into(),
            parent: parent.into(),
            schema_version,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StreamSchemasGetRequest, StreamSchemasGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.StreamSchemas/Get"),
                request,
                StreamSchemasGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("stream_schemas.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<StreamSchemasListResponse, RpcStatus> {
        self.list_with(parent, StreamSchemasListOptions::default()).await
    }
    pub async fn list_with(
        &self, parent: impl Into<String>, options: StreamSchemasListOptions,
    ) -> Result<StreamSchemasListResponse, RpcStatus> {
        let value = StreamSchemasListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StreamSchemasListRequest, StreamSchemasListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.StreamSchemas/List"),
                request,
                StreamSchemasListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("stream_schemas.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Streams {
    client: ClientTransport,
}
impl Streams {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<Vec<StreamIdentity>, RpcStatus> {
        self.list_with(StreamsListOptions::default()).await
    }
    pub async fn list_with(&self, options: StreamsListOptions) -> Result<Vec<StreamIdentity>, RpcStatus> {
        let value = StreamsListRequest { schema: options.schema };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<StreamsListRequest, StreamsListResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Streams/List"),
                request,
                StreamsListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("streams.list", error))?;
        let response = response.into_inner();
        Ok(response.streams)
    }
}
#[derive(Clone)]
pub struct Suites {
    client: ClientTransport,
}
impl Suites {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32,
    ) -> Result<SuitesGetResponse, RpcStatus> {
        self.get_with(name, parent, schema_version, SuitesGetOptions::default()).await
    }
    pub async fn get_with(
        &self, name: impl Into<String>, parent: impl Into<String>, schema_version: u32, options: SuitesGetOptions,
    ) -> Result<SuitesGetResponse, RpcStatus> {
        let value = SuitesGetRequest {
            name: name.into(),
            parent: parent.into(),
            schema_version,
            at: options.at,
        };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SuitesGetRequest, SuitesGetResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Suites/Get"),
                request,
                SuitesGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("suites.get", error))?;
        let response = response.into_inner();
        Ok(response)
    }
    pub async fn list(&self, parent: impl Into<String>) -> Result<SuitesListResponse, RpcStatus> {
        self.list_with(parent, SuitesListOptions::default()).await
    }
    pub async fn list_with(&self, parent: impl Into<String>, options: SuitesListOptions) -> Result<SuitesListResponse, RpcStatus> {
        let value = SuitesListRequest { parent: parent.into(), at: options.at };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<SuitesListRequest, SuitesListResponse>(
                PathAndQuery::from_static("/dokime.host.v1.Suites/List"),
                request,
                SuitesListResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("suites.list", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
#[derive(Clone)]
pub struct Time {
    client: ClientTransport,
}
impl Time {
    pub fn new(client: ClientTransport) -> Self {
        Self { client }
    }
    pub async fn get(&self) -> Result<Timestamp, RpcStatus> {
        self.get_with(TimeGetOptions::default()).await
    }
    pub async fn get_with(&self, options: TimeGetOptions) -> Result<Timestamp, RpcStatus> {
        let value = TimeGetRequest {};
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<TimeGetRequest, TimeGetResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Time/Get"),
                request,
                TimeGetResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("time.get", error))?;
        let response = response.into_inner();
        response.now.ok_or_else(|| RpcStatus::data_loss("Read Time.Get: Missing Payload"))
    }
    pub async fn sleep(&self, duration: Duration) -> Result<TimeSleepResponse, RpcStatus> {
        self.sleep_with(duration, TimeSleepOptions::default()).await
    }
    pub async fn sleep_with(&self, duration: Duration, options: TimeSleepOptions) -> Result<TimeSleepResponse, RpcStatus> {
        let value = TimeSleepRequest { duration: Some(duration) };
        let request = options.context.request(value);
        let response = self
            .client
            .unary::<TimeSleepRequest, TimeSleepResponse>(
                PathAndQuery::from_static("/harmos.host.v1.Time/Sleep"),
                request,
                TimeSleepResponse::default().descriptor(),
            )
            .await
            .map_err(|error| action_error("time.sleep", error))?;
        let response = response.into_inner();
        Ok(response)
    }
}
