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

pub const ID: &str = "dokime.reports.download_cycle_report";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ReportsDownloadCycleReportRequest, v1::ReportsDownloadCycleReportOutput>(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: v1::ReportsDownloadCycleReportRequest,
) -> Result<super::Output<v1::ReportsDownloadCycleReportOutput>> {
    super::complete(ID, context.settings.read_timeout, async {
        if !request.identifier.is_empty() && request.identifier != "cycle" {
            return Err(Error::Invalid("Download Cycle Report: Select the Cycle Report Identifier".into()));
        }
        let (file, metadata) = context.host.render_report(
            &v1::ReportSubject {
                kind: Some(v1::report_subject::Kind::Cycle(request.cycle_id)),
            },
            None,
        )?;
        let (file, length, digest) = super::content_transfer::temporary(file)?;
        super::content_transfer::download(
            context,
            super::content_transfer::Download {
                source: super::content_transfer::Source::Temporary(file),
                length,
                expected_digest: Some(digest),
            },
            v1::ReportsDownloadCycleReportOutput {
                value: Some(v1::reports_download_cycle_report_output::Value::Item(v1::ReportHtmlFrame {
                    payload: Some(v1::ReportHtmlFramePayload {
                        kind: Some(v1::report_html_frame_payload::Kind::Report(metadata)),
                    }),
                })),
            },
            |chunk| v1::ReportsDownloadCycleReportOutput {
                value: Some(v1::reports_download_cycle_report_output::Value::Item(v1::ReportHtmlFrame {
                    payload: Some(v1::ReportHtmlFramePayload {
                        kind: Some(v1::report_html_frame_payload::Kind::Chunk(v1::ReportHtmlChunk {
                            offset: chunk.offset,
                            data: chunk.data,
                        })),
                    }),
                })),
            },
            |complete| v1::ReportsDownloadCycleReportOutput {
                value: Some(v1::reports_download_cycle_report_output::Value::Item(v1::ReportHtmlFrame {
                    payload: Some(v1::ReportHtmlFramePayload {
                        kind: Some(v1::report_html_frame_payload::Kind::Complete(v1::ReportHtmlCompletion {
                            size_bytes: complete.size_bytes,
                            sha256: complete.sha256,
                        })),
                    }),
                })),
            },
            || v1::ReportsDownloadCycleReportOutput {
                value: Some(v1::reports_download_cycle_report_output::Value::Response(v1::ClientEmpty {})),
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
