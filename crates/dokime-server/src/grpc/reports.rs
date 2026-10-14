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
use harmos::Protobuf;
use harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;
// Local Imports
use super::chunked;
use super::value::canonical;
use super::{error::detailed, value};
use dokime_app::{
    RuntimeHandle,
    core::{Identifier, TestCycleId},
    execution::CycleStatus,
};
use dokime_contract::operations::{checkpoint, reports as wire};

/* === Constants === */
const HTML_CHUNK_BYTES: usize = 64 * 1024;
const MAX_REPORT_HTML_BYTES: usize = 16 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

struct HtmlFrames(Response<wire::ReportHtmlFrame>);
struct Download {
    resources: Arc<Resources>,
}
struct Render;

/* === Enums === */
#[harmos::catalog]
enum Operations {
    Download(Download),
    Render(Render),
}

/* === Traits === */

/* === Implementations === */
impl Operation for Download {
    type Declaration = wire::DownloadCycleReport;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<wire::ReportHtmlFrame>;
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::DownloadCycleReportRequest,
        output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        value::text(&request.cycle_id)?;
        let cycle = TestCycleId::from_uuid(request.cycle_id.clone())
            .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
        let identifier = report_identifier(&request.identifier)?;
        require_settled(&self.resources.runtime, &cycle, "final report")?;
        let storage = self.resources.runtime.storage();
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
        // The boxed report payload owns its heap pointee beside the inline frame.
        let required = std::mem::size_of::<wire::ReportHtmlFrame>()
            .saturating_add(std::mem::size_of::<wire::ReportMetadata>())
            .saturating_add(cycle.as_str().len())
            .saturating_add(identifier.as_str().len())
            .saturating_add(canonical::metadata_memory(&selected.value.metadata)?);
        if required
            > selection
                .decoded_bytes()
                .get()
                .min(self.resources.max_message_bytes)
        {
            return Err(Status::resource_exhausted(
                "report metadata exceeds its projection budget",
            ));
        }
        let allowance = std::num::NonZeroUsize::new(required)
            .ok_or_else(|| Status::resource_exhausted("invalid report metadata allowance"))?;
        let frame = selected
            .try_project(allowance, |selected, _| {
                let metadata = &selected.value.metadata;
                let frame = wire::ReportHtmlFrame {
                    payload: Some(wire::ReportHtmlFramePayload::Report(Box::new(
                        wire::ReportMetadata {
                            test_cycle_id: cycle.to_string(),
                            identifier: identifier.to_string(),
                            metadata: Some(
                                canonical::metadata(metadata).map_err(projection_failure)?,
                            ),
                            generated_at: Some(
                                canonical::timestamp(&seal.sealed_at)
                                    .map_err(projection_failure)?,
                            ),
                            warnings: Vec::new(),
                        },
                    ))),
                };
                if frame.encoded_len() > self.resources.max_message_bytes {
                    return Err(harmos::Error::ResourceExhausted {
                        resource: "report metadata encoding",
                    });
                }
                Ok(frame)
            })
            .map_err(|source| report_failure(source.into()))?;
        let owner = frame
            .try_into_response()
            .map_err(|source| report_failure(source.into()))?;
        chunked::Download {
            frames: HtmlFrames(owner),
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
            chunk_bytes: HTML_CHUNK_BYTES.min(self.resources.max_message_bytes.saturating_sub(256)),
            message_limit: self.resources.max_message_bytes,
        }
        .run(output)
        .await?;
        Ok(().into())
    }
}

impl Operation for Render {
    type Declaration = wire::RenderCheckpointReport;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(_: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self)
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::RenderCheckpointReportRequest,
        _: &mut (),
    ) -> Result<Response<wire::RenderedReport>, Status> {
        let (context, checkpoints) = inputs(
            request.context.take(),
            std::mem::take(&mut request.checkpoints),
        )?;
        let identifier = required_identifier(&request.identifier)?;
        let _ = (context, checkpoints, identifier);
        Err(Status::unimplemented(
            "sealed checkpoint reports return stored bytes and cannot be rendered again",
        ))
    }
}

impl chunked::Frames for HtmlFrames {
    type Frame = wire::ReportHtmlFrame;

    fn metadata(&self) -> Self::Frame {
        wire::ReportHtmlFrame {
            payload: self.0.payload.clone(),
        }
    }

    fn metadata_owned(&self) -> Option<Response<Self::Frame>> {
        Some(self.0.clone())
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        wire::ReportHtmlFrame {
            payload: Some(wire::ReportHtmlFramePayload::Chunk(wire::ReportHtmlChunk {
                offset,
                data,
            })),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        wire::ReportHtmlFrame {
            payload: Some(wire::ReportHtmlFramePayload::Complete(
                wire::ReportHtmlCompletion { size_bytes, sha256 },
            )),
        }
    }
}

/* === Public Functions === */
pub(super) fn register(
    runtime: &RuntimeHandle,
    max_message_bytes: usize,
) -> Result<(), super::Error> {
    runtime
        .operations()
        .register_wire_operations::<Operations, Resources>(
            "dokime",
            dokime_app::app::Lanes::Open,
            Arc::new(Resources {
                runtime: runtime.clone(),
                max_message_bytes,
            }),
        )?;
    Ok(())
}

/// Refuse a cycle-wide evidence product while the cycle is still producing evidence.
///
/// The final report and the evidence bundle are cut from the sealed cycle, and a
/// running, paused or stopping cycle is still adding evidence no seal covers.
/// Its absence from the live
/// state means nothing is running, so a stored cycle passes through to the
/// product's own checks.
pub(super) fn require_settled(
    runtime: &RuntimeHandle,
    cycle: &TestCycleId,
    product: &str,
) -> Result<(), Status> {
    let latest = runtime.watch_state().latest();
    let Some(lifecycle) = latest
        .state()
        .cycles
        .get(cycle)
        .map(|cycle| cycle.lifecycle)
    else {
        return Ok(());
    };
    let phase = match lifecycle {
        CycleStatus::Running => "running",
        CycleStatus::Paused => "paused",
        CycleStatus::Stopping => "stopping",
        _ => return Ok(()),
    };
    Err(Status::failed_precondition(format!(
        "The {product} becomes available when the cycle is terminal; this cycle is still {phase}."
    )))
}

/* === Private Functions === */
fn projection_failure(source: Status) -> harmos::Error {
    if source.code() == tonic::Code::ResourceExhausted {
        return harmos::Error::ResourceExhausted {
            resource: "report metadata projection",
        };
    }
    harmos::Error::backend(
        "project saved report metadata",
        harmos::BackendFailure::Corrupt,
        source,
    )
}
fn report_failure(error: dokime_app::Error) -> Status {
    let status = if matches!(
        &error,
        dokime_app::Error::Harmos(harmos::Error::ResourceExhausted { .. })
    ) {
        Status::resource_exhausted(format!("report operation failed: {error}"))
    } else {
        Status::failed_precondition(format!("report operation failed: {error}"))
    };
    detailed(status, &error)
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
    context: Option<checkpoint::ExecutionCheckpoint>,
    checkpoints: Vec<checkpoint::ExecutionCheckpoint>,
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
