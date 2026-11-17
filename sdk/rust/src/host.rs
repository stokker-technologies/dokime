// @generated from protocol descriptors; do not edit.
pub use crate::message::Wire;
use prost_reflect::ReflectMessage;
pub trait Request: ReflectMessage + Default + 'static {
    type Response: ReflectMessage + Default + 'static;
    const ID: &'static str;
}
impl<T: Request> harmos::OperationRequest for Wire<T> {
    type Response = Wire<T::Response>;
    const ID: &'static str = T::ID;
}
impl Request for dokime_protocol::host::BackendEnvironmentRequest {
    type Response = dokime_protocol::host::BackendEnvironmentResponse;
    const ID: &'static str = "dokime.host.v1.Backend/Environment";
}
impl Request for dokime_protocol::host::content::ArtifactsCreateRequest {
    type Response = dokime_protocol::host::content::ArtifactsCreateResponse;
    const ID: &'static str = "dokime.host.v1.content.Artifacts/Create";
}
impl Request for dokime_protocol::host::DocumentsAppendPartsRequest {
    type Response = dokime_protocol::host::DocumentsAppendPartsResponse;
    const ID: &'static str = "dokime.host.v1.Documents/AppendParts";
}
impl Request for dokime_protocol::host::EffectsCreateRequest {
    type Response = dokime_protocol::host::EffectsCreateResponse;
    const ID: &'static str = "dokime.host.v1.Effects/Create";
}
impl Request for dokime_protocol::host::EffectsGetRequest {
    type Response = dokime_protocol::host::EffectsGetResponse;
    const ID: &'static str = "dokime.host.v1.Effects/Get";
}
impl Request for dokime_protocol::host::EffectsSetRequest {
    type Response = dokime_protocol::host::EffectsSetResponse;
    const ID: &'static str = "dokime.host.v1.Effects/Set";
}
impl Request for dokime_protocol::host::evidence::LogsAppendRequest {
    type Response = dokime_protocol::host::evidence::LogsAppendResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Logs/Append";
}
impl Request for dokime_protocol::host::evidence::RecordsListRequest {
    type Response = dokime_protocol::host::evidence::RecordsListResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Records/List";
}
impl Request for dokime_protocol::host::evidence::RecordsAppendRequest {
    type Response = dokime_protocol::host::evidence::RecordsAppendResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Records/Append";
}
impl Request for dokime_protocol::host::evidence::TraceabilityGetRequest {
    type Response = dokime_protocol::host::evidence::TraceabilityGetResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Traceability/Get";
}
impl Request for dokime_protocol::host::evidence::TraceabilitySetRequest {
    type Response = dokime_protocol::host::evidence::TraceabilitySetResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Traceability/Set";
}
impl Request for dokime_protocol::host::EvidenceStatisticsRequest {
    type Response = dokime_protocol::host::EvidenceStatisticsResponse;
    const ID: &'static str = "dokime.host.v1.Evidence/Statistics";
}
impl Request for dokime_protocol::host::executions::ProgressGetRequest {
    type Response = dokime_protocol::host::executions::ProgressGetResponse;
    const ID: &'static str = "dokime.host.v1.executions.Progress/Get";
}
impl Request for dokime_protocol::host::executions::ProgressSetRequest {
    type Response = dokime_protocol::host::executions::ProgressSetResponse;
    const ID: &'static str = "dokime.host.v1.executions.Progress/Set";
}
impl Request for dokime_protocol::host::files::CapturesStartRequest {
    type Response = dokime_protocol::host::files::CapturesStartResponse;
    const ID: &'static str = "dokime.host.v1.files.Captures/Start";
}
impl Request for dokime_protocol::host::files::CapturesStopRequest {
    type Response = dokime_protocol::host::files::CapturesStopResponse;
    const ID: &'static str = "dokime.host.v1.files.Captures/Stop";
}
impl Request for dokime_protocol::host::files::UploadsCreateRequest {
    type Response = dokime_protocol::host::files::UploadsCreateResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Create";
}
impl Request for dokime_protocol::host::files::UploadsGetRequest {
    type Response = dokime_protocol::host::files::UploadsGetResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Get";
}
impl Request for dokime_protocol::host::files::UploadsResumeRequest {
    type Response = dokime_protocol::host::files::UploadsResumeResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Resume";
}
impl Request for dokime_protocol::host::files::UploadsAppendRequest {
    type Response = dokime_protocol::host::files::UploadsAppendResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Append";
}
impl Request for dokime_protocol::host::files::UploadsPublishRequest {
    type Response = dokime_protocol::host::files::UploadsPublishResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Publish";
}
impl Request for dokime_protocol::host::files::UploadsDeleteRequest {
    type Response = dokime_protocol::host::files::UploadsDeleteResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Delete";
}
impl Request for dokime_protocol::host::FilesGetRequest {
    type Response = dokime_protocol::host::FilesGetResponse;
    const ID: &'static str = "dokime.host.v1.Files/Get";
}
impl Request for dokime_protocol::host::FilesReadRequest {
    type Response = dokime_protocol::host::FilesReadResponse;
    const ID: &'static str = "dokime.host.v1.Files/Read";
}
impl Request for dokime_protocol::host::FilesWatchRequest {
    type Response = dokime_protocol::host::FilesWatchResponse;
    const ID: &'static str = "dokime.host.v1.Files/Watch";
}
impl Request for dokime_protocol::host::FilesDeleteRequest {
    type Response = dokime_protocol::host::FilesDeleteResponse;
    const ID: &'static str = "dokime.host.v1.Files/Delete";
}
impl Request for dokime_protocol::host::FilesCaptureRequest {
    type Response = dokime_protocol::host::FilesCaptureResponse;
    const ID: &'static str = "dokime.host.v1.Files/Capture";
}
impl Request for dokime_protocol::host::PromptsCreateRequest {
    type Response = dokime_protocol::host::PromptsCreateResponse;
    const ID: &'static str = "dokime.host.v1.Prompts/Create";
}
impl Request for dokime_protocol::host::SecretsGetRequest {
    type Response = dokime_protocol::host::SecretsGetResponse;
    const ID: &'static str = "dokime.host.v1.Secrets/Get";
}
impl Request for harmos_protocol::host::InvocationsGetRequest {
    type Response = harmos_protocol::host::InvocationsGetResponse;
    const ID: &'static str = "harmos.host.v1.Invocations/Get";
}
impl Request for harmos_protocol::host::InvocationsCancelRequest {
    type Response = harmos_protocol::host::InvocationsCancelResponse;
    const ID: &'static str = "harmos.host.v1.Invocations/Cancel";
}
impl Request for harmos_protocol::host::InvocationsWatchRequest {
    type Response = harmos_protocol::host::InvocationsWatchResponse;
    const ID: &'static str = "harmos.host.v1.Invocations/Watch";
}
impl Request for harmos_protocol::host::OperationsListRequest {
    type Response = harmos_protocol::host::OperationsListResponse;
    const ID: &'static str = "harmos.host.v1.Operations/List";
}
impl Request for harmos_protocol::host::OperationsGetRequest {
    type Response = harmos_protocol::host::OperationsGetResponse;
    const ID: &'static str = "harmos.host.v1.Operations/Get";
}
impl Request for harmos_protocol::host::plugins::SettingsListRequest {
    type Response = harmos_protocol::host::plugins::SettingsListResponse;
    const ID: &'static str = "harmos.host.v1.plugins.Settings/List";
}
impl Request for harmos_protocol::host::TimeGetRequest {
    type Response = harmos_protocol::host::TimeGetResponse;
    const ID: &'static str = "harmos.host.v1.Time/Get";
}
impl Request for harmos_protocol::host::TimeSleepRequest {
    type Response = harmos_protocol::host::TimeSleepResponse;
    const ID: &'static str = "harmos.host.v1.Time/Sleep";
}
