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

impl Request for dokime_protocol::context::evidence::RecordsAppendRequest {
    type Response = dokime_protocol::context::evidence::RecordsAppendResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Records/Append";
}

impl Request for dokime_protocol::context::evidence::RecordsListRequest {
    type Response = dokime_protocol::context::evidence::RecordsListResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Records/List";
}

impl Request for dokime_protocol::context::EvidenceStatisticsRequest {
    type Response = dokime_protocol::context::EvidenceStatisticsResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Evidence/Statistics";
}

impl Request for dokime_protocol::context::executions::ProgressGetRequest {
    type Response = dokime_protocol::context::executions::ProgressGetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.executions.Progress/Get";
}

impl Request for dokime_protocol::context::executions::ProgressSetRequest {
    type Response = dokime_protocol::context::executions::ProgressSetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.executions.Progress/Set";
}

impl Request for dokime_protocol::context::evidence::LogsAppendRequest {
    type Response = dokime_protocol::context::evidence::LogsAppendResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Logs/Append";
}

impl Request for dokime_protocol::context::ReadSettings {
    type Response = dokime_protocol::context::SavedSettings;
    const ID: &'static str = "dokime.settings.read";
}

impl Request for dokime_protocol::context::ReadSecret {
    type Response = dokime_protocol::context::Secret;
    const ID: &'static str = "dokime.secrets.read";
}

impl Request for dokime_protocol::context::PromptsCreateRequest {
    type Response = dokime_protocol::context::PromptsCreateResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Prompts/Create";
}

impl Request for dokime_protocol::context::evidence::TraceabilityGetRequest {
    type Response = dokime_protocol::context::evidence::TraceabilityGetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Traceability/Get";
}
impl Request for dokime_protocol::context::evidence::TraceabilitySetRequest {
    type Response = dokime_protocol::context::evidence::TraceabilitySetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.evidence.Traceability/Set";
}

impl Request for dokime_protocol::context::DocumentsAppendPartsRequest {
    type Response = dokime_protocol::context::DocumentsAppendPartsResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Documents/AppendParts";
}

impl Request for dokime_protocol::context::content::PublishArtifact {
    type Response = dokime_protocol::context::content::ArtifactReceipt;
    const ID: &'static str = "dokime.execution.artifact";
}

impl Request for dokime_protocol::context::FilesCaptureRequest {
    type Response = dokime_protocol::context::FilesCaptureResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Files/Capture";
}

impl Request for dokime_protocol::context::files::CapturesStartRequest {
    type Response = dokime_protocol::context::files::CapturesStartResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Captures/Start";
}

impl Request for dokime_protocol::context::files::CapturesStopRequest {
    type Response = dokime_protocol::context::files::CapturesStopResponse;
    const ID: &'static str = "dokime.plugin.host.v1.files.Captures/Stop";
}

impl Request for dokime_protocol::context::BackendEnvironmentRequest {
    type Response = dokime_protocol::context::BackendEnvironmentResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Backend/Environment";
}

impl Request for dokime_protocol::context::EffectsCreateRequest {
    type Response = dokime_protocol::context::EffectsCreateResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Effects/Create";
}

impl Request for dokime_protocol::context::EffectsGetRequest {
    type Response = dokime_protocol::context::EffectsGetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Effects/Get";
}

impl Request for dokime_protocol::context::EffectsSetRequest {
    type Response = dokime_protocol::context::EffectsSetResponse;
    const ID: &'static str = "dokime.plugin.host.v1.Effects/Set";
}

impl Request for dokime_protocol::context::content::ArtifactsCreateRequest {
    type Response = dokime_protocol::context::content::ArtifactsCreateResponse;
    const ID: &'static str = "dokime.plugin.host.v1.content.Artifacts/Create";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
