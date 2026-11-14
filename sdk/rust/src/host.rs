/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host

/* === Modules === */

/* === Re-Exports === */

pub use crate::message::Wire as Call;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost_reflect::ReflectMessage;

// Local Imports
use crate::message::Wire;
use dokime_protocol::context::BackendEnvironmentRequest;
use dokime_protocol::context::BackendEnvironmentResponse;
use dokime_protocol::context::DocumentsAppendPartsRequest;
use dokime_protocol::context::DocumentsAppendPartsResponse;
use dokime_protocol::context::EffectsCreateRequest;
use dokime_protocol::context::EffectsCreateResponse;
use dokime_protocol::context::EffectsGetRequest;
use dokime_protocol::context::EffectsGetResponse;
use dokime_protocol::context::EffectsSetRequest;
use dokime_protocol::context::EffectsSetResponse;
use dokime_protocol::context::EvidenceStatisticsRequest;
use dokime_protocol::context::EvidenceStatisticsResponse;
use dokime_protocol::context::FilesCaptureRequest;
use dokime_protocol::context::FilesCaptureResponse;
use dokime_protocol::context::FilesDeleteRequest;
use dokime_protocol::context::FilesDeleteResponse;
use dokime_protocol::context::FilesGetRequest;
use dokime_protocol::context::FilesGetResponse;
use dokime_protocol::context::FilesReadRequest;
use dokime_protocol::context::FilesReadResponse;
use dokime_protocol::context::PromptsCreateRequest;
use dokime_protocol::context::PromptsCreateResponse;
use dokime_protocol::context::ReadSecret;
use dokime_protocol::context::ReadSettings;
use dokime_protocol::context::SavedSettings;
use dokime_protocol::context::Secret;
use dokime_protocol::context::content::ArtifactsCreateRequest;
use dokime_protocol::context::content::ArtifactsCreateResponse;
use dokime_protocol::context::evidence::LogsAppendRequest;
use dokime_protocol::context::evidence::LogsAppendResponse;
use dokime_protocol::context::evidence::RecordsAppendRequest;
use dokime_protocol::context::evidence::RecordsAppendResponse;
use dokime_protocol::context::evidence::RecordsListRequest;
use dokime_protocol::context::evidence::RecordsListResponse;
use dokime_protocol::context::evidence::TraceabilityGetRequest;
use dokime_protocol::context::evidence::TraceabilityGetResponse;
use dokime_protocol::context::evidence::TraceabilitySetRequest;
use dokime_protocol::context::evidence::TraceabilitySetResponse;
use dokime_protocol::context::executions::ProgressGetRequest;
use dokime_protocol::context::executions::ProgressGetResponse;
use dokime_protocol::context::executions::ProgressSetRequest;
use dokime_protocol::context::executions::ProgressSetResponse;
use dokime_protocol::context::files::CapturesStartRequest;
use dokime_protocol::context::files::CapturesStartResponse;
use dokime_protocol::context::files::CapturesStopRequest;
use dokime_protocol::context::files::CapturesStopResponse;
use dokime_protocol::context::files::UploadsAppendRequest;
use dokime_protocol::context::files::UploadsAppendResponse;
use dokime_protocol::context::files::UploadsCreateRequest;
use dokime_protocol::context::files::UploadsCreateResponse;
use dokime_protocol::context::files::UploadsDeleteRequest;
use dokime_protocol::context::files::UploadsDeleteResponse;
use dokime_protocol::context::files::UploadsGetRequest;
use dokime_protocol::context::files::UploadsGetResponse;
use dokime_protocol::context::files::UploadsPublishRequest;
use dokime_protocol::context::files::UploadsPublishResponse;
use dokime_protocol::context::files::UploadsResumeRequest;
use dokime_protocol::context::files::UploadsResumeResponse;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

// Authoritative Dokime child-call binding. Requests and responses remain generated messages.
pub trait Request: ReflectMessage + Default + 'static {
    type Response: ReflectMessage + Default + 'static;
    const ID: &'static str;
}

/* === Implementations === */

impl<T: Request> harmos::OperationRequest for Wire<T> {
    type Response = Wire<T::Response>;
    const ID: &'static str = T::ID;
}

impl Request for RecordsAppendRequest {
    type Response = RecordsAppendResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Records/Append";
}

impl Request for RecordsListRequest {
    type Response = RecordsListResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Records/List";
}

impl Request for EvidenceStatisticsRequest {
    type Response = EvidenceStatisticsResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Evidence/Statistics";
}

impl Request for ProgressGetRequest {
    type Response = ProgressGetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.executions.Progress/Get";
}

impl Request for ProgressSetRequest {
    type Response = ProgressSetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.executions.Progress/Set";
}

impl Request for LogsAppendRequest {
    type Response = LogsAppendResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Logs/Append";
}

impl Request for ReadSettings {
    type Response = SavedSettings;
    const ID: &'static str = "dokime.settings.read";
}

impl Request for ReadSecret {
    type Response = Secret;
    const ID: &'static str = "dokime.secrets.read";
}

impl Request for PromptsCreateRequest {
    type Response = PromptsCreateResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Prompts/Create";
}

impl Request for TraceabilityGetRequest {
    type Response = TraceabilityGetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Traceability/Get";
}
impl Request for TraceabilitySetRequest {
    type Response = TraceabilitySetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Traceability/Set";
}

impl Request for DocumentsAppendPartsRequest {
    type Response = DocumentsAppendPartsResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Documents/AppendParts";
}

impl Request for FilesGetRequest {
    type Response = FilesGetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Files/Get";
}

impl Request for FilesReadRequest {
    type Response = FilesReadResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Files/Read";
}

impl Request for FilesDeleteRequest {
    type Response = FilesDeleteResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Files/Delete";
}

impl Request for UploadsCreateRequest {
    type Response = UploadsCreateResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Uploads/Create";
}

impl Request for UploadsGetRequest {
    type Response = UploadsGetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Uploads/Get";
}

impl Request for UploadsResumeRequest {
    type Response = UploadsResumeResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Uploads/Resume";
}

impl Request for UploadsAppendRequest {
    type Response = UploadsAppendResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Uploads/Append";
}

impl Request for UploadsPublishRequest {
    type Response = UploadsPublishResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Uploads/Publish";
}

impl Request for UploadsDeleteRequest {
    type Response = UploadsDeleteResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Uploads/Delete";
}

impl Request for ArtifactsCreateRequest {
    type Response = ArtifactsCreateResponse;
    const ID: &'static str = "dokime.plugin.host.v1.content.Artifacts/Create";
}

impl Request for FilesCaptureRequest {
    type Response = FilesCaptureResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Files/Capture";
}

impl Request for CapturesStartRequest {
    type Response = CapturesStartResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Captures/Start";
}

impl Request for CapturesStopRequest {
    type Response = CapturesStopResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Captures/Stop";
}

impl Request for BackendEnvironmentRequest {
    type Response = BackendEnvironmentResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Backend/Environment";
}

impl Request for EffectsCreateRequest {
    type Response = EffectsCreateResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Effects/Create";
}

impl Request for EffectsGetRequest {
    type Response = EffectsGetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Effects/Get";
}

impl Request for EffectsSetRequest {
    type Response = EffectsSetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Effects/Set";
}



/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
