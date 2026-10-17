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
use std::sync::Arc;
// Third Party Imports
use crate::grpc::transcode::Transcode;
use harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;
// Local Imports
use super::chunked;
use super::value::canonical;
use super::{error::detailed, value};
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::operations::reports as wire;
use dokime_runtime::OpenedReport;
use dokime_runtime::ReportOrigin;
use dokime_runtime::RuntimeHandle;

/* === Constants === */
const HTML_CHUNK_BYTES: usize = 64 * 1024;
const EXECUTION_REPORT_IDENTIFIER: &str = "execution.report";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

struct HtmlFrames(Response<dokime_domain::model::reports::wire::ReportHtmlFrame>);
struct Download {
    resources: Arc<Resources>,
}
struct Execution {
    resources: Arc<Resources>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    Download(Download),
    Execution(Execution),
}

/* === Traits === */

/* === Implementations === */
impl Operation for Download {
    type Declaration = wire::DownloadCycleReport;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime_domain::model::reports::wire::ReportHtmlFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::DownloadCycleReport, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        value::text(&request.cycle_id)?;
        let cycle = TestCycleId::from_uuid(request.cycle_id.clone()).map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
        let identifier = report_identifier(&request.identifier)?;
        let report = self.resources.runtime.open_cycle_report(&cycle).await.map_err(report_failure)?;
        deliver(&self.resources, report, &identifier, output).await
    }
}

impl Operation for Execution {
    type Declaration = wire::DownloadExecutionReport;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime_domain::model::reports::wire::ReportHtmlFrame>;
    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut wire::DownloadExecutionReport, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        value::text(&request.execution_id)?;
        let execution =
            TestExecutionId::from_uuid(request.execution_id.clone()).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
        let identifier =
            Identifier::new(EXECUTION_REPORT_IDENTIFIER).map_err(|_| Status::internal("execution report identifier is invalid"))?;
        let report = self.resources.runtime.open_execution_report(&execution).await.map_err(report_failure)?;
        deliver(&self.resources, report, &identifier, output).await
    }
}

impl chunked::Frames for HtmlFrames {
    type Frame = dokime_domain::model::reports::wire::ReportHtmlFrame;

    fn metadata(&self) -> Self::Frame {
        dokime_domain::model::reports::wire::ReportHtmlFrame { payload: self.0.payload.clone() }
    }

    fn metadata_owned(&self) -> Option<Response<Self::Frame>> {
        Some(self.0.clone())
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        dokime_domain::model::reports::wire::ReportHtmlFrame {
            payload: Some(dokime_domain::model::reports::wire::ReportHtmlFramePayload::Chunk(
                dokime_domain::model::reports::wire::ReportHtmlChunk { offset, data },
            )),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        dokime_domain::model::reports::wire::ReportHtmlFrame {
            payload: Some(dokime_domain::model::reports::wire::ReportHtmlFramePayload::Complete(
                dokime_domain::model::reports::wire::ReportHtmlCompletion { size_bytes, sha256 },
            )),
        }
    }
}

/* === Public Functions === */
pub(super) fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), super::Error> {
    runtime.operations().register_wire_operations::<Operations, Resources>(
        "dokime",
        dokime_runtime::app::Lanes::Open,
        Arc::new(Resources { runtime: runtime.clone(), max_message_bytes }),
    )?;
    Ok(())
}

/* === Private Functions === */
async fn deliver(
    resources: &Resources, report: OpenedReport, identifier: &Identifier,
    output: &mut StreamOutput<dokime_domain::model::reports::wire::ReportHtmlFrame>,
) -> Result<Response<()>, Status> {
    let OpenedReport { test_cycle_id, test_execution_id, metadata, generated_at, origin, html } = report;
    let frame = dokime_domain::model::reports::wire::ReportHtmlFrame {
        payload: Some(dokime_domain::model::reports::wire::ReportHtmlFramePayload::Report(Box::new(
            dokime_domain::model::reports::wire::ReportMetadata {
                test_cycle_id: test_cycle_id.to_string(),
                test_execution_id: test_execution_id.map(|id| id.to_string()),
                identifier: identifier.to_string(),
                metadata: metadata.as_ref().map(canonical::metadata).transpose()?,
                generated_at: Some(canonical::timestamp(&generated_at)?),
                origin: match origin {
                    ReportOrigin::Sealed => dokime_domain::model::reports::wire::ReportOrigin::Sealed,
                    ReportOrigin::Current => dokime_domain::model::reports::wire::ReportOrigin::Current,
                },
                warnings: Vec::new(),
            },
        ))),
    };
    if frame.wire_len()? > resources.max_message_bytes {
        return Err(Status::resource_exhausted("report metadata exceeds the message limit"));
    }
    let size_bytes = html.len() as u64;
    chunked::Download {
        frames: HtmlFrames(frame.into()),
        source: chunked::Source::bytes(html),
        size_bytes,
        sha256: None,
        noun: "report HTML",
        read_failure: "report HTML could not be read",
        chunk_bytes: HTML_CHUNK_BYTES.min(resources.max_message_bytes.saturating_sub(256)),
        message_limit: resources.max_message_bytes,
    }
    .run(output)
    .await?;
    Ok(().into())
}

fn report_failure(error: dokime_runtime::runtime::Error) -> Status {
    if let dokime_runtime::runtime::Error::Admission(AdmissionFailure::NotFound { message, .. }) = &error {
        return Status::not_found(message.clone());
    }
    let status = if matches!(&error, dokime_runtime::runtime::Error::Harmos(harmos::Error::ResourceExhausted { .. })) {
        Status::resource_exhausted(format!("report operation failed: {error}"))
    } else {
        Status::failed_precondition(format!("report operation failed: {error}"))
    };
    detailed(status, &error)
}

fn report_identifier(value: &str) -> Result<Identifier, Status> {
    if value.is_empty() {
        Identifier::new("cycle.summary").map_err(|_| Status::internal("default report identifier is invalid"))
    } else {
        required_identifier(value)
    }
}

fn required_identifier(value: &str) -> Result<Identifier, Status> {
    value::text(value)?;
    Identifier::new(value.to_owned()).map_err(|_| Status::invalid_argument("invalid report identifier"))
}

/* === Tests === */
