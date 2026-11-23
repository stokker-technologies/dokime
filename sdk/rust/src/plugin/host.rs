// @generated from protocol descriptors; do not edit.
use crate::plugin::{Dependency, Registrar, Registration};
use crate::{Context, Error, OperationPolicy, Result};
use dokime_protocol::host::BackendEnvironmentRequest;
use dokime_protocol::host::BackendEnvironmentResponse;
use dokime_protocol::host::DocumentsAppendPartsRequest;
use dokime_protocol::host::DocumentsAppendPartsResponse;
use dokime_protocol::host::EvidenceStatisticsRequest;
use dokime_protocol::host::EvidenceStatisticsResponse;
use dokime_protocol::host::FilesCaptureRequest;
use dokime_protocol::host::FilesCaptureResponse;
use dokime_protocol::host::FilesDeleteRequest;
use dokime_protocol::host::FilesDeleteResponse;
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::FilesGetResponse;
use dokime_protocol::host::FilesReadRequest;
use dokime_protocol::host::FilesReadResponse;
use dokime_protocol::host::FilesWatchRequest;
use dokime_protocol::host::FilesWatchResponse;
use dokime_protocol::host::Output;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::host::SecretsGetRequest;
use dokime_protocol::host::SecretsGetResponse;
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::content::ArtifactsCreateResponse;
use dokime_protocol::host::evidence::CapturedRecordSelection;
use dokime_protocol::host::evidence::LogsAppendRequest;
use dokime_protocol::host::evidence::LogsAppendResponse;
use dokime_protocol::host::evidence::RecordScope;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::evidence::RecordsAppendResponse;
use dokime_protocol::host::evidence::RecordsListRequest;
use dokime_protocol::host::evidence::RecordsListResponse;
use dokime_protocol::host::evidence::TraceabilityGetRequest;
use dokime_protocol::host::evidence::TraceabilityGetResponse;
use dokime_protocol::host::evidence::TraceabilitySetRequest;
use dokime_protocol::host::evidence::TraceabilitySetResponse;
use dokime_protocol::host::executions::ProgressGetRequest;
use dokime_protocol::host::executions::ProgressGetResponse;
use dokime_protocol::host::executions::ProgressSetRequest;
use dokime_protocol::host::executions::ProgressSetResponse;
use dokime_protocol::host::files::CaptureMode;
use dokime_protocol::host::files::CapturesStartRequest;
use dokime_protocol::host::files::CapturesStartResponse;
use dokime_protocol::host::files::CapturesStopRequest;
use dokime_protocol::host::files::CapturesStopResponse;
use dokime_protocol::host::files::UploadsAppendRequest;
use dokime_protocol::host::files::UploadsAppendResponse;
use dokime_protocol::host::files::UploadsCreateRequest;
use dokime_protocol::host::files::UploadsCreateResponse;
use dokime_protocol::host::files::UploadsDeleteRequest;
use dokime_protocol::host::files::UploadsDeleteResponse;
use dokime_protocol::host::files::UploadsGetRequest;
use dokime_protocol::host::files::UploadsGetResponse;
use dokime_protocol::host::files::UploadsPublishRequest;
use dokime_protocol::host::files::UploadsPublishResponse;
use dokime_protocol::host::files::UploadsResumeRequest;
use dokime_protocol::host::files::UploadsResumeResponse;
use dokime_protocol::model::ArtifactScope;
use dokime_protocol::model::Card;
use dokime_protocol::model::EvidenceScope;
use dokime_protocol::model::ExecutionProgressValue;
use dokime_protocol::model::FileSource;
use dokime_protocol::model::Metadata;
use dokime_protocol::model::Prompt;
use dokime_protocol::model::Section;
use dokime_protocol::model::Severity;
use dokime_protocol::model::TraceabilityEntry;
use dokime_protocol::model::Value;
use dokime_protocol::model::Window;
use harmos_protocol::host::InvocationsCancelRequest;
use harmos_protocol::host::InvocationsCancelResponse;
use harmos_protocol::host::InvocationsGetRequest;
use harmos_protocol::host::InvocationsGetResponse;
use harmos_protocol::host::InvocationsWatchRequest;
use harmos_protocol::host::InvocationsWatchResponse;
use harmos_protocol::host::OperationsGetRequest;
use harmos_protocol::host::OperationsGetResponse;
use harmos_protocol::host::OperationsListRequest;
use harmos_protocol::host::OperationsListResponse;
use harmos_protocol::host::TimeGetRequest;
use harmos_protocol::host::TimeSleepRequest;
use harmos_protocol::host::plugins::SettingsListRequest;
use harmos_protocol::host::plugins::SettingsListResponse;
use harmos_protocol::host::plugins::StateGetRequest;
use harmos_protocol::host::plugins::StateGetResponse;
use harmos_protocol::host::plugins::StatePutRequest;
use harmos_protocol::host::plugins::StatePutResponse;
use harmos_protocol::model::DiscoveredOperation;
use harmos_protocol::model::FileMetadata;
use harmos_protocol::model::FileReference;
use harmos_protocol::model::InstanceState;
use harmos_protocol::model::PageRequest;
use harmos_protocol::model::Position;
use harmos_protocol::model::ReadCut;
use harmos_protocol::model::Receipt;
use harmos_protocol::model::Upload;
use harmos_protocol::plugin::SubscriptionsSubscribeRequest;
use prost_types::Timestamp;
use std::num::NonZeroUsize;
pub enum HostOperation {
    BackendEnvironment,
    ContentArtifactsCreate,
    DocumentsAppendParts,
    EvidenceLogsAppend,
    EvidenceRecordsList,
    EvidenceRecordsAppend,
    EvidenceTraceabilityGet,
    EvidenceTraceabilitySet,
    EvidenceStatistics,
    ExecutionsProgressGet,
    ExecutionsProgressSet,
    FilesCapturesStart,
    FilesCapturesStop,
    FilesUploadsCreate,
    FilesUploadsGet,
    FilesUploadsResume,
    FilesUploadsAppend,
    FilesUploadsPublish,
    FilesUploadsDelete,
    FilesGet,
    FilesRead,
    FilesWatch,
    FilesDelete,
    FilesCapture,
    PromptsCreate,
    SecretsGet,
    InvocationsGet,
    InvocationsCancel,
    InvocationsWatch,
    OperationsList,
    OperationsGet,
    PluginsSettingsList,
    PluginsStateGet,
    PluginsStatePut,
    SubscriptionsSubscribe,
    TimeGet,
    TimeSleep,
}
impl HostOperation {
    pub const ALL: [Self; 37] = [
        Self::BackendEnvironment,
        Self::ContentArtifactsCreate,
        Self::DocumentsAppendParts,
        Self::EvidenceLogsAppend,
        Self::EvidenceRecordsList,
        Self::EvidenceRecordsAppend,
        Self::EvidenceTraceabilityGet,
        Self::EvidenceTraceabilitySet,
        Self::EvidenceStatistics,
        Self::ExecutionsProgressGet,
        Self::ExecutionsProgressSet,
        Self::FilesCapturesStart,
        Self::FilesCapturesStop,
        Self::FilesUploadsCreate,
        Self::FilesUploadsGet,
        Self::FilesUploadsResume,
        Self::FilesUploadsAppend,
        Self::FilesUploadsPublish,
        Self::FilesUploadsDelete,
        Self::FilesGet,
        Self::FilesRead,
        Self::FilesWatch,
        Self::FilesDelete,
        Self::FilesCapture,
        Self::PromptsCreate,
        Self::SecretsGet,
        Self::InvocationsGet,
        Self::InvocationsCancel,
        Self::InvocationsWatch,
        Self::OperationsList,
        Self::OperationsGet,
        Self::PluginsSettingsList,
        Self::PluginsStateGet,
        Self::PluginsStatePut,
        Self::SubscriptionsSubscribe,
        Self::TimeGet,
        Self::TimeSleep,
    ];
}
impl Registration for HostOperation {
    fn register(self, ctx: &mut Registrar<'_>) -> Result<()> {
        match self {
            HostOperation::BackendEnvironment => ctx.register(Dependency::<BackendEnvironmentRequest>::new()),
            HostOperation::ContentArtifactsCreate => ctx.register(Dependency::<ArtifactsCreateRequest>::new()),
            HostOperation::DocumentsAppendParts => ctx.register(Dependency::<DocumentsAppendPartsRequest>::new()),
            HostOperation::EvidenceLogsAppend => ctx.register(Dependency::<LogsAppendRequest>::new()),
            HostOperation::EvidenceRecordsList => ctx.register(Dependency::<RecordsListRequest>::new()),
            HostOperation::EvidenceRecordsAppend => ctx.register(Dependency::<RecordsAppendRequest>::new()),
            HostOperation::EvidenceTraceabilityGet => ctx.register(Dependency::<TraceabilityGetRequest>::new()),
            HostOperation::EvidenceTraceabilitySet => ctx.register(Dependency::<TraceabilitySetRequest>::new()),
            HostOperation::EvidenceStatistics => ctx.register(Dependency::<EvidenceStatisticsRequest>::new()),
            HostOperation::ExecutionsProgressGet => ctx.register(Dependency::<ProgressGetRequest>::new()),
            HostOperation::ExecutionsProgressSet => ctx.register(Dependency::<ProgressSetRequest>::new()),
            HostOperation::FilesCapturesStart => ctx.register(Dependency::<CapturesStartRequest>::new()),
            HostOperation::FilesCapturesStop => ctx.register(Dependency::<CapturesStopRequest>::new()),
            HostOperation::FilesUploadsCreate => ctx.register(Dependency::<UploadsCreateRequest>::new()),
            HostOperation::FilesUploadsGet => ctx.register(Dependency::<UploadsGetRequest>::new()),
            HostOperation::FilesUploadsResume => ctx.register(Dependency::<UploadsResumeRequest>::new()),
            HostOperation::FilesUploadsAppend => ctx.register(Dependency::<UploadsAppendRequest>::new()),
            HostOperation::FilesUploadsPublish => ctx.register(Dependency::<UploadsPublishRequest>::new()),
            HostOperation::FilesUploadsDelete => ctx.register(Dependency::<UploadsDeleteRequest>::new()),
            HostOperation::FilesGet => ctx.register(Dependency::<FilesGetRequest>::new()),
            HostOperation::FilesRead => ctx.register(Dependency::<FilesReadRequest>::new()),
            HostOperation::FilesWatch => ctx.register(Dependency::<FilesWatchRequest>::new()),
            HostOperation::FilesDelete => ctx.register(Dependency::<FilesDeleteRequest>::new()),
            HostOperation::FilesCapture => ctx.register(Dependency::<FilesCaptureRequest>::new()),
            HostOperation::PromptsCreate => ctx.register(Dependency::<PromptsCreateRequest>::new()),
            HostOperation::SecretsGet => ctx.register(Dependency::<SecretsGetRequest>::new()),
            HostOperation::InvocationsGet => ctx.register(Dependency::<InvocationsGetRequest>::new()),
            HostOperation::InvocationsCancel => ctx.register(Dependency::<InvocationsCancelRequest>::new()),
            HostOperation::InvocationsWatch => ctx.register(Dependency::<InvocationsWatchRequest>::new()),
            HostOperation::OperationsList => ctx.register(Dependency::<OperationsListRequest>::new()),
            HostOperation::OperationsGet => ctx.register(Dependency::<OperationsGetRequest>::new()),
            HostOperation::PluginsSettingsList => ctx.register(Dependency::<SettingsListRequest>::new()),
            HostOperation::PluginsStateGet => ctx.register(Dependency::<StateGetRequest>::new()),
            HostOperation::PluginsStatePut => ctx.register(Dependency::<StatePutRequest>::new()),
            HostOperation::SubscriptionsSubscribe => ctx.register(Dependency::<SubscriptionsSubscribeRequest>::new()),
            HostOperation::TimeGet => ctx.register(Dependency::<TimeGetRequest>::new()),
            HostOperation::TimeSleep => ctx.register(Dependency::<TimeSleepRequest>::new()),
        }
    }
}
impl Context {
    pub(crate) fn host(&self) -> HostRoot {
        HostRoot::new(self.clone())
    }
}
#[derive(Clone)]
pub struct HostRoot {
    pub backend: HostBackendResource,
    pub content: HostContentResource,
    pub documents: HostDocumentsResource,
    pub evidence: HostEvidenceResource,
    pub executions: HostExecutionsResource,
    pub files: HostFilesResource,
    pub invocations: HostInvocationsResource,
    pub operations: HostOperationsResource,
    pub plugins: HostPluginsResource,
    pub prompts: HostPromptsResource,
    pub secrets: HostSecretsResource,
}
impl HostRoot {
    fn new(context: Context) -> Self {
        Self {
            backend: HostBackendResource::new(context.clone()),
            content: HostContentResource::new(context.clone()),
            documents: HostDocumentsResource::new(context.clone()),
            evidence: HostEvidenceResource::new(context.clone()),
            executions: HostExecutionsResource::new(context.clone()),
            files: HostFilesResource::new(context.clone()),
            invocations: HostInvocationsResource::new(context.clone()),
            operations: HostOperationsResource::new(context.clone()),
            plugins: HostPluginsResource::new(context.clone()),
            prompts: HostPromptsResource::new(context.clone()),
            secrets: HostSecretsResource::new(context.clone()),
        }
    }
}
#[derive(Default)]
pub struct HostBackendEnvironmentOptions {
    pub at: Option<Position>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostContentArtifactsCreateOptions {
    pub scope: Option<ArtifactScope>,
    pub request_id: Option<String>,
    pub metadata: Option<Metadata>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostDocumentsAppendPartsOptions {
    pub subsection: Option<Section>,
    pub card: Option<Card>,
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostEvidenceLogsAppendOptions {
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostEvidenceRecordsListOptions {
    pub scope: Option<RecordScope>,
    pub versions: Option<Vec<u32>>,
    pub between: Option<Window>,
    pub step: Option<String>,
    pub selection: Option<CapturedRecordSelection>,
    pub at: Option<ReadCut>,
    pub page: Option<PageRequest>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostEvidenceRecordsAppendOptions {
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostEvidenceTraceabilityGetOptions {
    pub scope: Option<EvidenceScope>,
    pub page: Option<PageRequest>,
    pub at: Option<ReadCut>,
    pub entry: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostEvidenceTraceabilitySetOptions {
    pub name: Option<String>,
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostEvidenceStatisticsOptions {
    pub scope: Option<EvidenceScope>,
    pub versions: Option<Vec<u32>>,
    pub between: Option<Window>,
    pub outputs: Option<Vec<Output>>,
    pub step: Option<String>,
    pub at: Option<ReadCut>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostExecutionsProgressGetOptions {
    pub name: Option<String>,
    pub at: Option<Position>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostExecutionsProgressSetOptions {
    pub name: Option<String>,
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesCapturesStartOptions {
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesCapturesStopOptions {
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesUploadsCreateOptions {
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesUploadsGetOptions {
    pub at: Option<ReadCut>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesUploadsResumeOptions {
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesUploadsAppendOptions {
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesUploadsPublishOptions {
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesUploadsDeleteOptions {
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesGetOptions {
    pub at: Option<ReadCut>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesReadOptions {
    pub at: Option<ReadCut>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesWatchOptions {
    pub file: Option<FileReference>,
    pub upload: Option<Upload>,
    pub at: Option<ReadCut>,
}
pub struct HostFilesWatch {
    incoming: crate::context::Watching<FilesWatchResponse>,
}
impl HostFilesWatch {
    pub async fn next(&mut self) -> Result<Option<FilesWatchResponse>> {
        let Some(response) = self.incoming.next().await? else {
            return Ok(None);
        };
        Ok(Some(response))
    }
}
#[derive(Default)]
pub struct HostFilesDeleteOptions {
    pub request_id: Option<String>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostFilesCaptureOptions {
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostPromptsCreateOptions {
    pub definition: Option<Prompt>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostSecretsGetOptions {
    pub at: Option<ReadCut>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostInvocationsGetOptions {
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostInvocationsCancelOptions {
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostInvocationsWatchOptions {}
pub struct HostInvocationsWatch {
    incoming: crate::context::Watching<InvocationsWatchResponse>,
}
impl HostInvocationsWatch {
    pub async fn next(&mut self) -> Result<Option<InstanceState>> {
        let Some(response) = self.incoming.next().await? else {
            return Ok(None);
        };
        response.invocation.map(Some).ok_or_else(|| Error::Invalid("Read InvocationsWatch: Missing Payload".into()))
    }
}
#[derive(Default)]
pub struct HostOperationsListOptions {
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostOperationsGetOptions {
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostPluginsSettingsListOptions {
    pub name: Option<String>,
    pub at: Option<ReadCut>,
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostPluginsStateGetOptions {
    pub policy: Option<OperationPolicy>,
}
#[derive(Default)]
pub struct HostPluginsStatePutOptions {
    pub policy: Option<OperationPolicy>,
}
#[derive(Clone)]
pub struct HostBackendResource {
    context: Context,
}
impl HostBackendResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn environment(&self) -> Result<BackendEnvironmentResponse> {
        self.environment_with(HostBackendEnvironmentOptions::default()).await
    }
    pub async fn environment_with(&self, options: HostBackendEnvironmentOptions) -> Result<BackendEnvironmentResponse> {
        let request = BackendEnvironmentRequest { at: options.at };
        let response: BackendEnvironmentResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostContentResource {
    pub artifacts: HostContentArtifactsResource,
}
impl HostContentResource {
    fn new(context: Context) -> Self {
        Self {
            artifacts: HostContentArtifactsResource::new(context.clone()),
        }
    }
}
#[derive(Clone)]
pub struct HostContentArtifactsResource {
    context: Context,
}
impl HostContentArtifactsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn create(&self, file: FileReference, filename: String, content_type: String) -> Result<ArtifactsCreateResponse> {
        self.create_with(file, filename, content_type, HostContentArtifactsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, file: FileReference, filename: String, content_type: String, options: HostContentArtifactsCreateOptions,
    ) -> Result<ArtifactsCreateResponse> {
        let request = ArtifactsCreateRequest {
            file: Some(file),
            scope: options.scope,
            filename,
            content_type,
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
            metadata: options.metadata,
        };
        let response: ArtifactsCreateResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostDocumentsResource {
    context: Context,
}
impl HostDocumentsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn append_parts(&self, execution: String, document: String, section: String) -> Result<Receipt> {
        self.append_parts_with(execution, document, section, HostDocumentsAppendPartsOptions::default()).await
    }
    pub async fn append_parts_with(
        &self, execution: String, document: String, section: String, options: HostDocumentsAppendPartsOptions,
    ) -> Result<Receipt> {
        let request = DocumentsAppendPartsRequest {
            part: match (options.subsection, options.card) {
                (Some(value), None) => Some(dokime_protocol::host::documents_append_parts_request::Part::Subsection(value)),
                (None, Some(value)) => Some(dokime_protocol::host::documents_append_parts_request::Part::Card(value)),
                _ => return Err(Error::Invalid("Choose Exactly One part Variant".into())),
            },
            execution,
            document,
            section,
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: DocumentsAppendPartsResponse = self.context.operation_with(request, options.policy).await?;
        response.receipt.ok_or_else(|| Error::Invalid("Read DocumentsAppendParts: Missing Payload".into()))
    }
}
#[derive(Clone)]
pub struct HostEvidenceResource {
    context: Context,
    pub logs: HostEvidenceLogsResource,
    pub records: HostEvidenceRecordsResource,
    pub traceability: HostEvidenceTraceabilityResource,
}
impl HostEvidenceResource {
    fn new(context: Context) -> Self {
        Self {
            logs: HostEvidenceLogsResource::new(context.clone()),
            records: HostEvidenceRecordsResource::new(context.clone()),
            traceability: HostEvidenceTraceabilityResource::new(context.clone()),
            context,
        }
    }
    pub async fn statistics(&self, stream: String, field: String) -> Result<EvidenceStatisticsResponse> {
        self.statistics_with(stream, field, HostEvidenceStatisticsOptions::default()).await
    }
    pub async fn statistics_with(
        &self, stream: String, field: String, options: HostEvidenceStatisticsOptions,
    ) -> Result<EvidenceStatisticsResponse> {
        let request = EvidenceStatisticsRequest {
            scope: options.scope,
            stream,
            versions: options.versions.unwrap_or_default(),
            between: options.between,
            outputs: options.outputs.unwrap_or_default(),
            field,
            step: options.step.unwrap_or_default(),
            at: options.at,
        };
        let response: EvidenceStatisticsResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostEvidenceLogsResource {
    context: Context,
}
impl HostEvidenceLogsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn append(&self, execution: String, severity: Severity, message: String, at: Timestamp) -> Result<Receipt> {
        self.append_with(execution, severity, message, at, HostEvidenceLogsAppendOptions::default()).await
    }
    pub async fn append_with(
        &self, execution: String, severity: Severity, message: String, at: Timestamp, options: HostEvidenceLogsAppendOptions,
    ) -> Result<Receipt> {
        let request = LogsAppendRequest {
            execution,
            severity: severity as i32,
            message,
            at: Some(at),
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: LogsAppendResponse = self.context.operation_with(request, options.policy).await?;
        response.receipt.ok_or_else(|| Error::Invalid("Read EvidenceLogsAppend: Missing Payload".into()))
    }
}
#[derive(Clone)]
pub struct HostEvidenceRecordsResource {
    context: Context,
}
impl HostEvidenceRecordsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn list(&self, record_type: String) -> Result<RecordsListResponse> {
        self.list_with(record_type, HostEvidenceRecordsListOptions::default()).await
    }
    pub async fn list_with(&self, record_type: String, options: HostEvidenceRecordsListOptions) -> Result<RecordsListResponse> {
        let request = RecordsListRequest {
            scope: options.scope,
            record_type,
            versions: options.versions.unwrap_or_default(),
            between: options.between,
            step: options.step.unwrap_or_default(),
            selection: options.selection,
            at: options.at,
            page: options.page,
        };
        let response: RecordsListResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn append(&self, execution: String, kind: String, payload: Vec<u8>, source_at: Timestamp) -> Result<RecordsAppendResponse> {
        self.append_with(execution, kind, payload, source_at, HostEvidenceRecordsAppendOptions::default()).await
    }
    pub async fn append_with(
        &self, execution: String, kind: String, payload: Vec<u8>, source_at: Timestamp, options: HostEvidenceRecordsAppendOptions,
    ) -> Result<RecordsAppendResponse> {
        let request = RecordsAppendRequest {
            execution,
            kind,
            payload,
            source_at: Some(source_at),
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: RecordsAppendResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostEvidenceTraceabilityResource {
    context: Context,
}
impl HostEvidenceTraceabilityResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn get(&self) -> Result<TraceabilityGetResponse> {
        self.get_with(HostEvidenceTraceabilityGetOptions::default()).await
    }
    pub async fn get_with(&self, options: HostEvidenceTraceabilityGetOptions) -> Result<TraceabilityGetResponse> {
        let request = TraceabilityGetRequest {
            scope: options.scope,
            page: options.page,
            at: options.at,
            entry: options.entry,
        };
        let response: TraceabilityGetResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn set(&self, value: TraceabilityEntry, expected_revision: Vec<u8>, entry: String) -> Result<TraceabilitySetResponse> {
        self.set_with(value, expected_revision, entry, HostEvidenceTraceabilitySetOptions::default()).await
    }
    pub async fn set_with(
        &self, value: TraceabilityEntry, expected_revision: Vec<u8>, entry: String, options: HostEvidenceTraceabilitySetOptions,
    ) -> Result<TraceabilitySetResponse> {
        let request = TraceabilitySetRequest {
            name: options.name,
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
            entry,
        };
        let response: TraceabilitySetResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostExecutionsResource {
    pub progress: HostExecutionsProgressResource,
}
impl HostExecutionsResource {
    fn new(context: Context) -> Self {
        Self {
            progress: HostExecutionsProgressResource::new(context.clone()),
        }
    }
}
#[derive(Clone)]
pub struct HostExecutionsProgressResource {
    context: Context,
}
impl HostExecutionsProgressResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn get(&self) -> Result<ProgressGetResponse> {
        self.get_with(HostExecutionsProgressGetOptions::default()).await
    }
    pub async fn get_with(&self, options: HostExecutionsProgressGetOptions) -> Result<ProgressGetResponse> {
        let request = ProgressGetRequest { name: options.name, at: options.at };
        let response: ProgressGetResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn set(&self, value: ExecutionProgressValue, expected_revision: Vec<u8>) -> Result<ProgressSetResponse> {
        self.set_with(value, expected_revision, HostExecutionsProgressSetOptions::default()).await
    }
    pub async fn set_with(
        &self, value: ExecutionProgressValue, expected_revision: Vec<u8>, options: HostExecutionsProgressSetOptions,
    ) -> Result<ProgressSetResponse> {
        let request = ProgressSetRequest {
            name: options.name,
            value: Some(value),
            expected_revision,
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: ProgressSetResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostFilesResource {
    context: Context,
    pub captures: HostFilesCapturesResource,
    pub uploads: HostFilesUploadsResource,
}
impl HostFilesResource {
    fn new(context: Context) -> Self {
        Self {
            captures: HostFilesCapturesResource::new(context.clone()),
            uploads: HostFilesUploadsResource::new(context.clone()),
            context,
        }
    }
    pub async fn get(&self, file: FileReference) -> Result<FilesGetResponse> {
        self.get_with(file, HostFilesGetOptions::default()).await
    }
    pub async fn get_with(&self, file: FileReference, options: HostFilesGetOptions) -> Result<FilesGetResponse> {
        let request = FilesGetRequest { file: Some(file), at: options.at };
        let response: FilesGetResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn read(&self, file: FileReference, offset: u64, max_bytes: u64) -> Result<FilesReadResponse> {
        self.read_with(file, offset, max_bytes, HostFilesReadOptions::default()).await
    }
    pub async fn read_with(
        &self, file: FileReference, offset: u64, max_bytes: u64, options: HostFilesReadOptions,
    ) -> Result<FilesReadResponse> {
        let request = FilesReadRequest {
            file: Some(file),
            offset,
            max_bytes,
            at: options.at,
        };
        let response: FilesReadResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn watch(&self, offset: u64, max_item_bytes: NonZeroUsize) -> Result<HostFilesWatch> {
        self.watch_with(offset, max_item_bytes, HostFilesWatchOptions::default()).await
    }
    pub async fn watch_with(&self, offset: u64, max_item_bytes: NonZeroUsize, options: HostFilesWatchOptions) -> Result<HostFilesWatch> {
        let request = FilesWatchRequest {
            target: match (options.file, options.upload) {
                (Some(value), None) => Some(dokime_protocol::host::files_watch_request::Target::File(value)),
                (None, Some(value)) => Some(dokime_protocol::host::files_watch_request::Target::Upload(value)),
                _ => return Err(Error::Invalid("Choose Exactly One target Variant".into())),
            },
            offset,
            at: options.at,
        };
        Ok(HostFilesWatch {
            incoming: self.context.watch(request, max_item_bytes).await?,
        })
    }
    pub async fn delete(&self, file: FileReference) -> Result<Receipt> {
        self.delete_with(file, HostFilesDeleteOptions::default()).await
    }
    pub async fn delete_with(&self, file: FileReference, options: HostFilesDeleteOptions) -> Result<Receipt> {
        let request = FilesDeleteRequest {
            file: Some(file),
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: FilesDeleteResponse = self.context.operation_with(request, options.policy).await?;
        response.receipt.ok_or_else(|| Error::Invalid("Read FilesDelete: Missing Payload".into()))
    }
    pub async fn capture(&self, source: FileSource, output: String) -> Result<FilesCaptureResponse> {
        self.capture_with(source, output, HostFilesCaptureOptions::default()).await
    }
    pub async fn capture_with(&self, source: FileSource, output: String, options: HostFilesCaptureOptions) -> Result<FilesCaptureResponse> {
        let request = FilesCaptureRequest { source: Some(source), output };
        let response: FilesCaptureResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostFilesCapturesResource {
    context: Context,
}
impl HostFilesCapturesResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn start(&self, source: FileSource, output: String, mode: CaptureMode) -> Result<u64> {
        self.start_with(source, output, mode, HostFilesCapturesStartOptions::default()).await
    }
    pub async fn start_with(
        &self, source: FileSource, output: String, mode: CaptureMode, options: HostFilesCapturesStartOptions,
    ) -> Result<u64> {
        let request = CapturesStartRequest {
            source: Some(source),
            output,
            mode: Some(mode),
        };
        let response: CapturesStartResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response.capture_id)
    }
    pub async fn stop(&self, capture_id: u64) -> Result<CapturesStopResponse> {
        self.stop_with(capture_id, HostFilesCapturesStopOptions::default()).await
    }
    pub async fn stop_with(&self, capture_id: u64, options: HostFilesCapturesStopOptions) -> Result<CapturesStopResponse> {
        let request = CapturesStopRequest { capture_id };
        let response: CapturesStopResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostFilesUploadsResource {
    context: Context,
}
impl HostFilesUploadsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn create(&self, metadata: FileMetadata) -> Result<UploadsCreateResponse> {
        self.create_with(metadata, HostFilesUploadsCreateOptions::default()).await
    }
    pub async fn create_with(&self, metadata: FileMetadata, options: HostFilesUploadsCreateOptions) -> Result<UploadsCreateResponse> {
        let request = UploadsCreateRequest {
            metadata: Some(metadata),
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: UploadsCreateResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn get(&self, upload: Upload) -> Result<UploadsGetResponse> {
        self.get_with(upload, HostFilesUploadsGetOptions::default()).await
    }
    pub async fn get_with(&self, upload: Upload, options: HostFilesUploadsGetOptions) -> Result<UploadsGetResponse> {
        let request = UploadsGetRequest { upload: Some(upload), at: options.at };
        let response: UploadsGetResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn resume(&self, upload: Upload) -> Result<UploadsResumeResponse> {
        self.resume_with(upload, HostFilesUploadsResumeOptions::default()).await
    }
    pub async fn resume_with(&self, upload: Upload, options: HostFilesUploadsResumeOptions) -> Result<UploadsResumeResponse> {
        let request = UploadsResumeRequest {
            upload: Some(upload),
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: UploadsResumeResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn append(&self, upload: Upload, offset: u64, data: Vec<u8>) -> Result<UploadsAppendResponse> {
        self.append_with(upload, offset, data, HostFilesUploadsAppendOptions::default()).await
    }
    pub async fn append_with(
        &self, upload: Upload, offset: u64, data: Vec<u8>, options: HostFilesUploadsAppendOptions,
    ) -> Result<UploadsAppendResponse> {
        let request = UploadsAppendRequest {
            upload: Some(upload),
            offset,
            data,
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: UploadsAppendResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn publish(&self, upload: Upload, length: u64, sha256: Vec<u8>) -> Result<UploadsPublishResponse> {
        self.publish_with(upload, length, sha256, HostFilesUploadsPublishOptions::default()).await
    }
    pub async fn publish_with(
        &self, upload: Upload, length: u64, sha256: Vec<u8>, options: HostFilesUploadsPublishOptions,
    ) -> Result<UploadsPublishResponse> {
        let request = UploadsPublishRequest {
            upload: Some(upload),
            length,
            sha256,
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: UploadsPublishResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn delete(&self, upload: Upload) -> Result<UploadsDeleteResponse> {
        self.delete_with(upload, HostFilesUploadsDeleteOptions::default()).await
    }
    pub async fn delete_with(&self, upload: Upload, options: HostFilesUploadsDeleteOptions) -> Result<UploadsDeleteResponse> {
        let request = UploadsDeleteRequest {
            upload: Some(upload),
            request_id: match options.request_id {
                Some(value) => value,
                None => harmos::client::request_id()?,
            },
        };
        let response: UploadsDeleteResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostInvocationsResource {
    context: Context,
}
impl HostInvocationsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn get(&self, instance: u64) -> Result<InstanceState> {
        self.get_with(instance, HostInvocationsGetOptions::default()).await
    }
    pub async fn get_with(&self, instance: u64, options: HostInvocationsGetOptions) -> Result<InstanceState> {
        let request = InvocationsGetRequest { instance };
        let response: InvocationsGetResponse = self.context.operation_with(request, options.policy).await?;
        response.invocation.ok_or_else(|| Error::Invalid("Read InvocationsGet: Missing Payload".into()))
    }
    pub async fn cancel(&self, instance: u64, reason: String) -> Result<InstanceState> {
        self.cancel_with(instance, reason, HostInvocationsCancelOptions::default()).await
    }
    pub async fn cancel_with(&self, instance: u64, reason: String, options: HostInvocationsCancelOptions) -> Result<InstanceState> {
        let request = InvocationsCancelRequest { instance, reason };
        let response: InvocationsCancelResponse = self.context.operation_with(request, options.policy).await?;
        response.invocation.ok_or_else(|| Error::Invalid("Read InvocationsCancel: Missing Payload".into()))
    }
    pub async fn watch(&self, instance: u64, max_item_bytes: NonZeroUsize) -> Result<HostInvocationsWatch> {
        self.watch_with(instance, max_item_bytes, HostInvocationsWatchOptions::default()).await
    }
    pub async fn watch_with(
        &self, instance: u64, max_item_bytes: NonZeroUsize, options: HostInvocationsWatchOptions,
    ) -> Result<HostInvocationsWatch> {
        let request = InvocationsWatchRequest { instance };
        let HostInvocationsWatchOptions {} = options;
        Ok(HostInvocationsWatch {
            incoming: self.context.watch(request, max_item_bytes).await?,
        })
    }
}
#[derive(Clone)]
pub struct HostOperationsResource {
    context: Context,
}
impl HostOperationsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn list(&self) -> Result<Vec<DiscoveredOperation>> {
        self.list_with(HostOperationsListOptions::default()).await
    }
    pub async fn list_with(&self, options: HostOperationsListOptions) -> Result<Vec<DiscoveredOperation>> {
        let request = OperationsListRequest {};
        let response: OperationsListResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response.operations)
    }
    pub async fn get(&self, id: String) -> Result<DiscoveredOperation> {
        self.get_with(id, HostOperationsGetOptions::default()).await
    }
    pub async fn get_with(&self, id: String, options: HostOperationsGetOptions) -> Result<DiscoveredOperation> {
        let request = OperationsGetRequest { id };
        let response: OperationsGetResponse = self.context.operation_with(request, options.policy).await?;
        response.operation.ok_or_else(|| Error::Invalid("Read OperationsGet: Missing Payload".into()))
    }
}
#[derive(Clone)]
pub struct HostPluginsResource {
    pub settings: HostPluginsSettingsResource,
    pub state: HostPluginsStateResource,
}
impl HostPluginsResource {
    fn new(context: Context) -> Self {
        Self {
            settings: HostPluginsSettingsResource::new(context.clone()),
            state: HostPluginsStateResource::new(context.clone()),
        }
    }
}
#[derive(Clone)]
pub struct HostPluginsSettingsResource {
    context: Context,
}
impl HostPluginsSettingsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn list(&self) -> Result<SettingsListResponse> {
        self.list_with(HostPluginsSettingsListOptions::default()).await
    }
    pub async fn list_with(&self, options: HostPluginsSettingsListOptions) -> Result<SettingsListResponse> {
        let request = SettingsListRequest { name: options.name, at: options.at };
        let response: SettingsListResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostPluginsStateResource {
    context: Context,
}
impl HostPluginsStateResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn get(&self) -> Result<StateGetResponse> {
        self.get_with(HostPluginsStateGetOptions::default()).await
    }
    pub async fn get_with(&self, options: HostPluginsStateGetOptions) -> Result<StateGetResponse> {
        let request = StateGetRequest {};
        let response: StateGetResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
    pub async fn put(&self, expected_revision: u64, value: Vec<u8>) -> Result<u64> {
        self.put_with(expected_revision, value, HostPluginsStatePutOptions::default()).await
    }
    pub async fn put_with(&self, expected_revision: u64, value: Vec<u8>, options: HostPluginsStatePutOptions) -> Result<u64> {
        let request = StatePutRequest { expected_revision, value };
        let response: StatePutResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response.revision)
    }
}
#[derive(Clone)]
pub struct HostPromptsResource {
    context: Context,
}
impl HostPromptsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn create(
        &self, prompt: String, prompt_type: String, values: std::collections::HashMap<String, Value>,
    ) -> Result<PromptsCreateResponse> {
        self.create_with(prompt, prompt_type, values, HostPromptsCreateOptions::default()).await
    }
    pub async fn create_with(
        &self, prompt: String, prompt_type: String, values: std::collections::HashMap<String, Value>, options: HostPromptsCreateOptions,
    ) -> Result<PromptsCreateResponse> {
        let request = PromptsCreateRequest {
            prompt,
            prompt_type,
            values,
            definition: options.definition,
        };
        let response: PromptsCreateResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
#[derive(Clone)]
pub struct HostSecretsResource {
    context: Context,
}
impl HostSecretsResource {
    fn new(context: Context) -> Self {
        Self { context }
    }
    pub async fn get(&self, name: String) -> Result<SecretsGetResponse> {
        self.get_with(name, HostSecretsGetOptions::default()).await
    }
    pub async fn get_with(&self, name: String, options: HostSecretsGetOptions) -> Result<SecretsGetResponse> {
        let request = SecretsGetRequest { name, at: options.at };
        let response: SecretsGetResponse = self.context.operation_with(request, options.policy).await?;
        Ok(response)
    }
}
