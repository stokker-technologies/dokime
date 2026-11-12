/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use sha2::Digest;

// Local Imports
use crate::{
    Result,
    client::{Context, Output},
};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.reports.download_execution_report";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ReportsDownloadExecutionReportRequest, v1::ReportsDownloadExecutionReportOutput>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: v1::ReportsDownloadExecutionReportRequest,
) -> Result<Output<v1::ReportsDownloadExecutionReportOutput>> {
    super::complete(ID, context.settings.read_timeout, async {
        let snapshot = context.host.store().read(&request.execution_id)?;
        let html = super::projection::report(&snapshot);
        let bytes = html.into_bytes();
        let digest = sha2::Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let size_bytes = bytes.len() as u64;
        let metadata = v1::ReportMetadata {
            test_cycle_id: snapshot.execution.cycle.clone(),
            test_execution_id: Some(request.execution_id.clone()),
            identifier: request.execution_id,
            origin: v1::ReportOrigin::Current as i32,
            ..Default::default()
        };
        let (sender, output) = tokio::sync::mpsc::channel(context.settings.stream_buffer);
        let chunk_bytes = context.settings.message_bytes / 2;
        let timeout = context.settings.read_timeout;
        tokio::spawn(async move {
            let work = async {
                let frame = super::projection::report_frame(v1::report_html_frame_payload::Kind::Report(metadata));
                if sender.send(Ok(frame)).await.is_err() {
                    return;
                }
                for (index, data) in bytes.chunks(chunk_bytes).enumerate() {
                    let chunk = v1::ReportHtmlChunk {
                        offset: (index * chunk_bytes) as u64,
                        data: data.to_vec(),
                    };
                    let frame = super::projection::report_frame(v1::report_html_frame_payload::Kind::Chunk(chunk));
                    if sender.send(Ok(frame)).await.is_err() {
                        return;
                    }
                }
                let complete = v1::ReportHtmlCompletion { size_bytes, sha256: digest };
                let frame = super::projection::report_frame(v1::report_html_frame_payload::Kind::Complete(complete));
                if sender.send(Ok(frame)).await.is_err() {
                    return;
                }
                let frame = v1::ReportsDownloadExecutionReportOutput {
                    value: Some(v1::reports_download_execution_report_output::Value::Response(v1::ClientEmpty {})),
                };
                let _ = sender.send(Ok(frame)).await;
            };
            if tokio::time::timeout(timeout, work).await.is_err() {
                let _ = sender.try_send(Err(tonic::Status::deadline_exceeded("DownloadExecutionReport: Operation Deadline Expired")));
            }
        });
        Ok(tokio_stream::wrappers::ReceiverStream::new(output))
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
