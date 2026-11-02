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
use dokime::harmos::operations::StreamOutput;
use dokime::harmos::{Context, Response};
use tonic::Status;

// Local Imports
use super::chunked;
use super::value::canonical;
use dokime::OpenedReport;
use dokime::ReportOrigin;
use dokime::app::LiveRuntime;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::model::identity::Identifier;
use dokime::domain::model::identity::TestCycleId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::reports as wire;

/* === Constants === */

const EXECUTION_REPORT_IDENTIFIER: &str = "execution.report";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    pub(super) runtime: LiveRuntime,
    pub(super) max_message_bytes: usize,
}

struct HtmlFrames(Response<dokime::domain::model::reports::wire::ReportHtmlFrame>);

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum PreparedOperations {
    Operation0(wire::DownloadCycleReport),
    Operation1(wire::DownloadExecutionReport),
}

/* === Traits === */

/* === Implementations === */

impl Serve<wire::DownloadCycleReport> for Resources {
    fn serve<'a>(&'a self, request: wire::DownloadCycleReport, ctx: Context<'a, wire::DownloadCycleReport>) -> Served<'a, ()> {
        Box::pin(async move {
            let call_context = ctx.call();
            let output = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                let cycle = TestCycleId::from_uuid(request.cycle_id.clone()).map_err(|_| Status::invalid_argument("invalid cycle UUID"))?;
                let identifier = report_identifier(&request.identifier)?;
                let report = dokime::app::live_runtime(&self.runtime).await?.open_cycle_report(&cycle).await.map_err(report_failure)?;
                deliver(self, report, &identifier, output).await
            })
            .await
        })
    }
}

impl Serve<wire::DownloadExecutionReport> for Resources {
    fn serve<'a>(&'a self, request: wire::DownloadExecutionReport, ctx: Context<'a, wire::DownloadExecutionReport>) -> Served<'a, ()> {
        Box::pin(async move {
            let call_context = ctx.call();
            let output = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                let execution = TestExecutionId::from_uuid(request.execution_id.clone())
                    .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
                let identifier =
                    Identifier::new(EXECUTION_REPORT_IDENTIFIER).map_err(|_| Status::internal("execution report identifier is invalid"))?;
                let report =
                    dokime::app::live_runtime(&self.runtime).await?.open_execution_report(&execution).await.map_err(report_failure)?;
                deliver(self, report, &identifier, output).await
            })
            .await
        })
    }
}

impl chunked::Frames for HtmlFrames {
    type Frame = dokime::domain::model::reports::wire::ReportHtmlFrame;

    fn metadata(&self) -> Self::Frame {
        dokime::domain::model::reports::wire::ReportHtmlFrame { payload: self.0.payload.clone() }
    }

    fn metadata_owned(&self) -> Option<Response<Self::Frame>> {
        Some(self.0.clone())
    }

    fn chunk(&self, offset: u64, data: Vec<u8>) -> Self::Frame {
        dokime::domain::model::reports::wire::ReportHtmlFrame {
            payload: Some(dokime::domain::model::reports::wire::ReportHtmlFramePayload::Chunk(
                dokime::domain::model::reports::wire::ReportHtmlChunk { offset, data },
            )),
        }
    }

    fn complete(&self, size_bytes: u64, sha256: String) -> Self::Frame {
        dokime::domain::model::reports::wire::ReportHtmlFrame {
            payload: Some(dokime::domain::model::reports::wire::ReportHtmlFramePayload::Complete(
                dokime::domain::model::reports::wire::ReportHtmlCompletion { size_bytes, sha256 },
            )),
        }
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, max_message_bytes: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        max_message_bytes,
    });

    setup.resource::<Host<wire::DownloadCycleReport>>(Host::<wire::DownloadCycleReport>::new(served.clone()));
    setup.resource::<Host<wire::DownloadExecutionReport>>(Host::<wire::DownloadExecutionReport>::new(served.clone()));

    setup.operations::<PreparedOperations>(dokime::app::Lanes::Open);
    Ok(())
}

/* === Private Functions === */

async fn deliver(
    resources: &Resources, report: OpenedReport, identifier: &Identifier,
    output: &StreamOutput<dokime::domain::model::reports::wire::ReportHtmlFrame>,
) -> Result<Response<()>, Status> {
    let OpenedReport {
        test_cycle_id,
        test_execution_id,
        metadata,
        generated_at,
        origin,
        file,
        reader,
    } = report;
    let frame = dokime::domain::model::reports::wire::ReportHtmlFrame {
        payload: Some(dokime::domain::model::reports::wire::ReportHtmlFramePayload::Report(Box::new(
            dokime::domain::model::reports::wire::ReportMetadata {
                test_cycle_id: test_cycle_id.to_string(),
                test_execution_id: test_execution_id.map(|id| id.to_string()),
                identifier: identifier.to_string(),
                metadata: metadata.as_ref().map(canonical::metadata).transpose()?,
                generated_at: Some(canonical::timestamp(&generated_at)?),
                origin: match origin {
                    ReportOrigin::Sealed => dokime::domain::model::reports::wire::ReportOrigin::Sealed,
                    ReportOrigin::Current => dokime::domain::model::reports::wire::ReportOrigin::Current,
                },
                warnings: Vec::new(),
            },
        ))),
    };
    if super::content::output_frame_len(&frame)? > resources.max_message_bytes {
        return Err(Status::resource_exhausted("report metadata exceeds the message limit"));
    }
    let size_bytes = file.length;
    let sha256: String = file.sha256.iter().map(|byte| format!("{byte:02x}")).collect();
    chunked::Download {
        frames: HtmlFrames(frame.into()),
        source: chunked::Source::opened(reader),
        size_bytes,
        sha256: Some(sha256),
        noun: "report HTML",
        read_failure: "report HTML could not be read",
        chunk_bytes: 1_048_576,
        message_limit: resources.max_message_bytes,
    }
    .run(output)
    .await?;
    Ok(().into())
}

fn report_failure(error: dokime::runtime::Error) -> Status {
    if let dokime::runtime::Error::Admission(AdmissionFailure::NotFound { message, .. }) = &error {
        return Status::not_found(message.clone());
    }
    tracing::error!(error = ?error, "report download failed");
    if matches!(&error, dokime::runtime::Error::Harmos(dokime::harmos::Error::ResourceExhausted { .. })) {
        Status::resource_exhausted("There is not enough capacity to create this report. Try again after other report operations finish.")
    } else {
        Status::failed_precondition("The report could not be created or read. Try again; if the problem continues, collect diagnostics.")
    }
}

fn report_identifier(value: &str) -> Result<Identifier, Status> {
    if value.is_empty() {
        Identifier::new("cycle.summary").map_err(|_| Status::internal("default report identifier is invalid"))
    } else {
        required_identifier(value)
    }
}

fn required_identifier(value: &str) -> Result<Identifier, Status> {
    Identifier::new(value.to_owned()).map_err(|_| Status::invalid_argument("invalid report identifier"))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grpc::chunked::Frames as _;

    #[test]
    fn report_error_privacy() {
        let detail = "private.storage.detail".to_owned();
        let error = dokime::runtime::Error::Operation(detail.clone());
        let status = report_failure(error);
        assert_eq!(status.code(), tonic::Code::FailedPrecondition);
        assert!(!status.message().contains(&detail));
        assert!(status.details().is_empty());
    }

    #[test]
    fn report_chunks_fit_the_full_envelope() {
        let frames = HtmlFrames(dokime::domain::model::reports::wire::ReportHtmlFrame { payload: None }.into());
        for offset in [0, 127, 128, 16_384, u64::MAX] {
            let template = frames.chunk(offset, Vec::new());
            let maximum = 1_048_576;
            let length = crate::grpc::content::output_chunk_bytes(&template, maximum, maximum).unwrap();
            let frame = frames.chunk(offset, vec![0; length]);
            assert!(crate::grpc::content::output_frame_len(&frame).unwrap() <= maximum);
            let overflow = frames.chunk(offset, vec![0; length + 1]);
            assert!(crate::grpc::content::output_frame_len(&overflow).unwrap() > maximum);
        }
    }

    #[tokio::test]
    async fn download_delivers_contiguous_chunks_and_verified_digest() {
        use dokime::domain::model::reports::wire::ReportHtmlFramePayload;
        use dokime::harmos::operations::{StreamSettings, stream};
        use std::num::NonZeroUsize;
        let (output, input) = stream(StreamSettings {
            items: NonZeroUsize::new(4).unwrap(),
            bytes: NonZeroUsize::new(4 * 1_048_576).unwrap(),
        })
        .unwrap();
        let length = 2 * 1_048_576 + 13;
        let digest = "ab".repeat(32);
        let expected = digest.clone();
        let download = chunked::Download {
            frames: HtmlFrames(dokime::domain::model::reports::wire::ReportHtmlFrame { payload: None }.into()),
            source: chunked::Source::bytes(vec![42; length].into()),
            size_bytes: length as u64,
            sha256: Some(digest),
            noun: "report HTML",
            read_failure: "report read failed",
            chunk_bytes: 1_048_576,
            message_limit: 1_048_576,
        };
        let producer = async move { download.run(&output).await };
        let consumer = async move {
            let mut offset = 0;
            let mut complete = false;
            while let Some(frame) = input.recv().await {
                match &frame.payload {
                    Some(ReportHtmlFramePayload::Chunk(chunk)) => {
                        assert!(!complete);
                        assert_eq!(chunk.offset, offset);
                        assert!(chunk.data.iter().all(|byte| *byte == 42));
                        offset += chunk.data.len() as u64;
                        assert!(crate::grpc::content::output_frame_len(&*frame).unwrap() <= 1_048_576);
                    }
                    Some(ReportHtmlFramePayload::Complete(completion)) => {
                        assert!(!complete);
                        assert_eq!(completion.size_bytes, offset);
                        assert_eq!(completion.sha256, expected);
                        complete = true;
                    }
                    _ => {}
                }
            }
            assert!(complete);
            assert_eq!(offset, length as u64);
        };
        let (result, ()) = tokio::join!(producer, consumer);
        result.unwrap();
    }
}
