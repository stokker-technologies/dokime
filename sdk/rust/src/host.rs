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

impl Request for crate::v1::PublishRecord {
    type Response = crate::v1::RecordReceipt;
    const ID: &'static str = "dokime.execution.record";
}

impl Request for crate::v1::RecordsRequest {
    type Response = crate::v1::Records;
    const ID: &'static str = "dokime.execution.records";
}

impl Request for crate::v1::StatisticsRequest {
    type Response = crate::v1::Statistics;
    const ID: &'static str = "dokime.execution.statistics";
}

impl Request for crate::v1::ReportProgress {
    type Response = crate::v1::ReportProgressResponse;
    const ID: &'static str = "dokime.execution.progress";
}

impl Request for crate::v1::WriteLog {
    type Response = crate::v1::WriteLogResponse;
    const ID: &'static str = "dokime.execution.log";
}

impl Request for crate::v1::ReadSettings {
    type Response = crate::v1::SavedSettings;
    const ID: &'static str = "dokime.settings.read";
}

impl Request for crate::v1::ReadSecret {
    type Response = crate::v1::Secret;
    const ID: &'static str = "dokime.secrets.read";
}

impl Request for crate::v1::OperatorQuestion {
    type Response = crate::v1::OperatorAnswer;
    const ID: &'static str = "dokime.execution.prompt";
}

impl Request for crate::v1::SetTraceability {
    type Response = crate::v1::SetTraceabilityResponse;
    const ID: &'static str = "dokime.execution.traceability";
}

impl Request for crate::v1::AddDocumentPart {
    type Response = crate::v1::AddDocumentPartResponse;
    const ID: &'static str = "dokime.execution.document";
}

impl Request for crate::v1::PublishArtifact {
    type Response = crate::v1::ArtifactReceipt;
    const ID: &'static str = "dokime.execution.artifact";
}

impl Request for crate::v1::CopyFiles {
    type Response = crate::v1::CapturedFiles;
    const ID: &'static str = "dokime.files.copy_files";
}

impl Request for crate::v1::StartCapture {
    type Response = crate::v1::CaptureStarted;
    const ID: &'static str = "dokime.files.start_capture";
}

impl Request for crate::v1::StopCapture {
    type Response = crate::v1::CapturedFiles;
    const ID: &'static str = "dokime.files.stop_capture";
}

impl Request for crate::v1::ReadEnvironment {
    type Response = crate::v1::Environment;
    const ID: &'static str = "dokime.execution.environment";
}

impl Request for crate::v1::PrepareEffect {
    type Response = crate::v1::Ticket;
    const ID: &'static str = "dokime.execution.effect.prepare";
}

impl Request for crate::v1::ConfirmEffect {
    type Response = crate::v1::ConfirmEffectResponse;
    const ID: &'static str = "dokime.execution.effect.applied";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
