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
use std::pin::Pin;
// Third Party Imports
use futures_util::Stream;
use tokio::sync::watch;
use tonic::{Request, Response, Status};
// Local Imports
use super::chunked;
use super::{error::detailed, value};
use dokime_app::{
    RuntimeHandle,
    core::{Identifier, TestCycleId},
};
use dokime_protocol::v1::{self, reports_server::Reports};

/* === Constants === */
const HTML_CHUNK_BYTES: usize = 64 * 1024;
const MAX_REPORT_HTML_BYTES: usize = 16 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) stopping: watch::Receiver<bool>,
    pub(super) max_message_bytes: usize,
}

struct HtmlFrames(v1::ReportMetadata);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Reports for Service {
    type DownloadCycleReportStream =
        Pin<Box<dyn Stream<Item = Result<v1::ReportHtmlFrame, Status>> + Send + 'static>>;

    async fn download_cycle_report(
        &self,
        request: Request<v1::DownloadCycleReportRequest>,
    ) -> Result<Response<Self::DownloadCycleReportStream>, Status> {
        if *self.stopping.borrow() {
            return Err(Status::unavailable("listener is stopping"));
        }
        let request = request.into_inner();
        let cycle = TestCycleId::from_uuid(request.cycle_id)
            .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
        let identifier = report_identifier(&request.identifier)?;
        let storage = self.runtime.storage();
        let mut query =
            harmos::storage::ObjectQuery::<dokime::storage::objects::cycle::Cycle>::new();
        query.item = Some(cycle.as_str());
        let selection = storage
            .query(query)
            .await
            .map_err(|error| report_failure(error.into()))?;
        let view = selection
            .view()
            .ok_or_else(|| Status::internal("cycle query has no saved view"))?
            .clone();
        let selected = dokime::storage::objects::cycle::read_in(
            storage,
            &view,
            &cycle,
            harmos::storage::ObjectVersion::Latest,
            selection.decoded_bytes(),
        )
        .await
        .map_err(|error| report_failure(error.into()))?
        .ok_or_else(|| Status::not_found("cycle not found"))?;
        let seal = selected
            .value
            .saved
            .as_ref()
            .ok_or_else(|| Status::failed_precondition("cycle has no saved final report"))?;
        seal.validate(&selected.value)
            .map_err(|error| Status::failed_precondition(error.to_string()))?;
        if seal.report.length > MAX_REPORT_HTML_BYTES as u64 {
            return Err(Status::resource_exhausted(
                "report HTML exceeds the download limit",
            ));
        }
        let reader = view
            .read_file(&seal.report)
            .await
            .map_err(|error| report_failure(error.into()))?;
        Ok(Response::new(
            chunked::Download {
                frames: HtmlFrames(v1::ReportMetadata {
                    test_cycle_id: cycle.to_string(),
                    identifier: identifier.to_string(),
                    metadata: Some(value::checked_metadata(&selected.value.metadata)?),
                    generated_at: Some(value::timestamp(&seal.sealed_at)?),
                    warnings: Vec::new(),
                }),
                source: chunked::Source::Native(Box::new(reader)),
                size_bytes: seal.report.length,
                sha256: Some(
                    seal.report
                        .sha256
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect(),
                ),
                noun: "report HTML",
                read_failure: "saved report HTML could not be read",
                chunk_bytes: HTML_CHUNK_BYTES.min(self.max_message_bytes.saturating_sub(256)),
                message_limit: self.max_message_bytes,
                stopping: self.stopping.clone(),
            }
            .stream()?,
        ))
    }

    async fn render_checkpoint_report(
        &self,
        request: Request<v1::RenderCheckpointReportRequest>,
    ) -> Result<Response<v1::RenderedReport>, Status> {
        let request = request.into_inner();
        let (context, checkpoints) = inputs(request.context, request.checkpoints)?;
        let identifier = required_identifier(&request.identifier)?;
        let _ = (context, checkpoints, identifier);
        Err(Status::unimplemented(
            "sealed checkpoint reports return stored bytes and cannot be rendered again",
        ))
    }
}

impl chunked::Frames for HtmlFrames {
    type Frame = v1::ReportHtmlFrame;

    fn metadata(&self) -> Self::Frame {
        v1::ReportHtmlFrame {
            payload: Some(v1::report_html_frame::Payload::Report(self.0.clone())),
        }
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        v1::ReportHtmlFrame {
            payload: Some(v1::report_html_frame::Payload::Chunk(v1::ReportHtmlChunk {
                offset,
                data,
            })),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        v1::ReportHtmlFrame {
            payload: Some(v1::report_html_frame::Payload::Complete(
                v1::ReportHtmlCompletion { size_bytes, sha256 },
            )),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn report_failure(error: dokime_app::Error) -> Status {
    detailed(
        Status::failed_precondition(format!("report operation failed: {error}")),
        &error,
    )
}

fn report_identifier(value: &str) -> Result<Identifier, Status> {
    if value.is_empty() {
        Identifier::new("cycle.summary")
            .map_err(|_| Status::internal("default report identifier is invalid"))
    } else {
        required_identifier(value)
    }
}

fn required_identifier(value: &str) -> Result<Identifier, Status> {
    value::text(value)?;
    Identifier::new(value.to_owned())
        .map_err(|_| Status::invalid_argument("invalid report identifier"))
}

fn inputs(
    context: Option<v1::ExecutionCheckpoint>,
    checkpoints: Vec<v1::ExecutionCheckpoint>,
) -> Result<
    (
        dokime_app::runtime::query::execution_history::ExecutionCheckpoint,
        Vec<dokime_app::runtime::query::execution_history::ExecutionCheckpoint>,
    ),
    Status,
> {
    value::collection(checkpoints.len())?;
    let context = super::history::checkpoint(context)?;
    let checkpoints = checkpoints
        .into_iter()
        .map(|checkpoint| super::history::checkpoint(Some(checkpoint)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((context, checkpoints))
}

/* === Tests === */
