// @generated from protocol descriptors; do not edit.
pub use crate::message::Wire;
use prost_reflect::ReflectMessage;
pub trait Request: ReflectMessage + Default + 'static {
    type Response: ReflectMessage + Default + 'static;
    const ID: &'static str;
    const WATCH: bool;
}
impl<T: Request> harmos::OperationRequest for Wire<T> {
    type Response = Wire<T::Response>;
    const ID: &'static str = T::ID;
    const WATCH: bool = T::WATCH;
}
impl Request for dokime_protocol::host::BackendEnvironmentRequest {
    type Response = dokime_protocol::host::BackendEnvironmentResponse;
    const ID: &'static str = "dokime.host.v1.Backend/Environment";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::content::ArtifactsCreateRequest {
    type Response = dokime_protocol::host::content::ArtifactsCreateResponse;
    const ID: &'static str = "dokime.host.v1.content.Artifacts/Create";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::DocumentsAppendPartsRequest {
    type Response = dokime_protocol::host::DocumentsAppendPartsResponse;
    const ID: &'static str = "dokime.host.v1.Documents/AppendParts";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::evidence::LogsAppendRequest {
    type Response = dokime_protocol::host::evidence::LogsAppendResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Logs/Append";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::evidence::RecordsListRequest {
    type Response = dokime_protocol::host::evidence::RecordsListResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Records/List";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::evidence::RecordsAppendRequest {
    type Response = dokime_protocol::host::evidence::RecordsAppendResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Records/Append";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::evidence::TraceabilityGetRequest {
    type Response = dokime_protocol::host::evidence::TraceabilityGetResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Traceability/Get";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::evidence::TraceabilitySetRequest {
    type Response = dokime_protocol::host::evidence::TraceabilitySetResponse;
    const ID: &'static str = "dokime.host.v1.evidence.Traceability/Set";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::EvidenceStatisticsRequest {
    type Response = dokime_protocol::host::EvidenceStatisticsResponse;
    const ID: &'static str = "dokime.host.v1.Evidence/Statistics";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::executions::ProgressGetRequest {
    type Response = dokime_protocol::host::executions::ProgressGetResponse;
    const ID: &'static str = "dokime.host.v1.executions.Progress/Get";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::executions::ProgressSetRequest {
    type Response = dokime_protocol::host::executions::ProgressSetResponse;
    const ID: &'static str = "dokime.host.v1.executions.Progress/Set";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::files::CapturesStartRequest {
    type Response = dokime_protocol::host::files::CapturesStartResponse;
    const ID: &'static str = "dokime.host.v1.files.Captures/Start";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::files::CapturesStopRequest {
    type Response = dokime_protocol::host::files::CapturesStopResponse;
    const ID: &'static str = "dokime.host.v1.files.Captures/Stop";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::files::UploadsCreateRequest {
    type Response = dokime_protocol::host::files::UploadsCreateResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Create";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::files::UploadsGetRequest {
    type Response = dokime_protocol::host::files::UploadsGetResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Get";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::files::UploadsResumeRequest {
    type Response = dokime_protocol::host::files::UploadsResumeResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Resume";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::files::UploadsAppendRequest {
    type Response = dokime_protocol::host::files::UploadsAppendResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Append";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::files::UploadsPublishRequest {
    type Response = dokime_protocol::host::files::UploadsPublishResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Publish";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::files::UploadsDeleteRequest {
    type Response = dokime_protocol::host::files::UploadsDeleteResponse;
    const ID: &'static str = "dokime.host.v1.files.Uploads/Delete";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::FilesGetRequest {
    type Response = dokime_protocol::host::FilesGetResponse;
    const ID: &'static str = "dokime.host.v1.Files/Get";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::FilesReadRequest {
    type Response = dokime_protocol::host::FilesReadResponse;
    const ID: &'static str = "dokime.host.v1.Files/Read";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::FilesWatchRequest {
    type Response = dokime_protocol::host::FilesWatchResponse;
    const ID: &'static str = "dokime.host.v1.Files/Watch";
    const WATCH: bool = true;
}
impl Request for dokime_protocol::host::FilesDeleteRequest {
    type Response = dokime_protocol::host::FilesDeleteResponse;
    const ID: &'static str = "dokime.host.v1.Files/Delete";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::FilesCaptureRequest {
    type Response = dokime_protocol::host::FilesCaptureResponse;
    const ID: &'static str = "dokime.host.v1.Files/Capture";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::PromptsCreateRequest {
    type Response = dokime_protocol::host::PromptsCreateResponse;
    const ID: &'static str = "dokime.host.v1.Prompts/Create";
    const WATCH: bool = false;
}
impl Request for dokime_protocol::host::SecretsGetRequest {
    type Response = dokime_protocol::host::SecretsGetResponse;
    const ID: &'static str = "dokime.host.v1.Secrets/Get";
    const WATCH: bool = false;
}
impl Request for harmos_protocol::host::InvocationsGetRequest {
    type Response = harmos_protocol::host::InvocationsGetResponse;
    const ID: &'static str = "harmos.plugin.v1.Invocations/Get";
    const WATCH: bool = false;
}
impl Request for harmos_protocol::host::InvocationsCancelRequest {
    type Response = harmos_protocol::host::InvocationsCancelResponse;
    const ID: &'static str = "harmos.plugin.v1.Invocations/Cancel";
    const WATCH: bool = false;
}
impl Request for harmos_protocol::host::InvocationsWatchRequest {
    type Response = harmos_protocol::host::InvocationsWatchResponse;
    const ID: &'static str = "harmos.plugin.v1.Invocations/Watch";
    const WATCH: bool = true;
}
impl Request for harmos_protocol::host::OperationsListRequest {
    type Response = harmos_protocol::host::OperationsListResponse;
    const ID: &'static str = "harmos.plugin.v1.Operations/List";
    const WATCH: bool = false;
}
impl Request for harmos_protocol::host::OperationsGetRequest {
    type Response = harmos_protocol::host::OperationsGetResponse;
    const ID: &'static str = "harmos.plugin.v1.Operations/Get";
    const WATCH: bool = false;
}
impl Request for harmos_protocol::host::plugins::SettingsListRequest {
    type Response = harmos_protocol::host::plugins::SettingsListResponse;
    const ID: &'static str = "harmos.plugin.v1.plugins.Settings/List";
    const WATCH: bool = false;
}
impl Request for harmos_protocol::host::TimeGetRequest {
    type Response = harmos_protocol::host::TimeGetResponse;
    const ID: &'static str = "harmos.plugin.v1.Time/Get";
    const WATCH: bool = false;
}
impl Request for harmos_protocol::host::TimeSleepRequest {
    type Response = harmos_protocol::host::TimeSleepResponse;
    const ID: &'static str = "harmos.plugin.v1.Time/Sleep";
    const WATCH: bool = false;
}
