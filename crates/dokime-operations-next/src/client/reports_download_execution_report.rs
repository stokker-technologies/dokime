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

// Local Imports
use crate::{Error, Result, client::Context};
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
) -> Result<super::Output<v1::ReportsDownloadExecutionReportOutput>> {
    let context = context.clone();
    super::complete(ID, context.settings.read_timeout, async move {
        tokio::task::spawn_blocking(move || {
            let context = &context;

            let cut = context.host.store().metadata(&request.execution_id)?;
            let published = context.host.store().reports(&request.execution_id)?.into_iter().next();
            let (file, metadata) = if let Some(report) = published {
                use std::io::Write;
                let mut file = tempfile::tempfile().map_err(|error| Error::Invalid(error.to_string()))?;
                file.write_all(&report.html).map_err(|error| Error::Invalid(error.to_string()))?;
                (
                    file,
                    v1::ReportMetadata {
                        test_cycle_id: cut.execution.cycle,
                        test_execution_id: Some(request.execution_id.clone()),
                        identifier: report.identifier,
                        metadata: report.document.and_then(|document| document.metadata),
                        origin: v1::ReportOrigin::Sealed as i32,
                        ..Default::default()
                    },
                )
            } else {
                context.host.render_report(
                    &v1::ReportSubject {
                        kind: Some(v1::report_subject::Kind::Execution(request.execution_id)),
                    },
                    None,
                )?
            };
            let (file, length, digest) = super::content_transfer::temporary(file)?;
            super::content_transfer::download(
                context,
                super::content_transfer::Download {
                    source: super::content_transfer::Source::Temporary(file),
                    length,
                    expected_digest: Some(digest),
                },
                v1::ReportsDownloadExecutionReportOutput {
                    value: Some(v1::reports_download_execution_report_output::Value::Item(v1::ReportHtmlFrame {
                        payload: Some(v1::ReportHtmlFramePayload {
                            kind: Some(v1::report_html_frame_payload::Kind::Report(metadata)),
                        }),
                    })),
                },
                |chunk| v1::ReportsDownloadExecutionReportOutput {
                    value: Some(v1::reports_download_execution_report_output::Value::Item(v1::ReportHtmlFrame {
                        payload: Some(v1::ReportHtmlFramePayload {
                            kind: Some(v1::report_html_frame_payload::Kind::Chunk(v1::ReportHtmlChunk {
                                offset: chunk.offset,
                                data: chunk.data,
                            })),
                        }),
                    })),
                },
                |complete| v1::ReportsDownloadExecutionReportOutput {
                    value: Some(v1::reports_download_execution_report_output::Value::Item(v1::ReportHtmlFrame {
                        payload: Some(v1::ReportHtmlFramePayload {
                            kind: Some(v1::report_html_frame_payload::Kind::Complete(v1::ReportHtmlCompletion {
                                size_bytes: complete.size_bytes,
                                sha256: complete.sha256,
                            })),
                        }),
                    })),
                },
                || v1::ReportsDownloadExecutionReportOutput {
                    value: Some(v1::reports_download_execution_report_output::Value::Response(v1::ClientEmpty {})),
                },
            )
        })
        .await
        .map_err(|error| Error::Invalid(format!("Render Report: Retry After the Rendering Worker Failed: {error}")))?
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
