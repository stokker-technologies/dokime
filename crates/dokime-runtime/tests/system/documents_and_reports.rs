/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Document Rendering and Publication Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD};
use prost::Message;

// Local Imports
use dokime_protocol::host::files_client::FilesClient;
use dokime_protocol::host::reports_client::ReportsClient;
use dokime_protocol::host::{FilesReadRequest, ReportCapture, ReportSubject, ReportsCreateRequest, ReportsPublishRequest, report_subject};
use dokime_runtime::{Host, services};
use dokime_storage::Store;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn decode_report(html: &[u8]) -> Result<ReportCapture, Box<dyn Error>> {
    let html = str::from_utf8(html)?;
    let start = html.find("id=\"hub-report-data\"").or_else(|| html.find("id=hub-report-data")).ok_or("Report Data Element Is Missing")?;
    let element = &html[start..];
    let data = element.split_once('>').ok_or("Report Data Is Missing")?.1.split_once("</script>").ok_or("Report Data Is Incomplete")?.0;
    let bytes = STANDARD.decode(data.trim())?;
    Ok(ReportCapture::decode(bytes.as_slice())?)
}

/* === Private Functions === */

/* === Tests === */

#[test]
fn documents_and_reports() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Seal Evidence and Take a Fixed Snapshot
            let root = super::support::tempdir()?;
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            let entry = host.add_managed("execution", &super::support::execution_fixture()?, &[]).await?;
            let document = host.catalog(&entry)?.suites[0].templates[0].documents[0].clone();
            let origin = host.materialize(&entry, "execution.capture", super::support::parameters(23.0, 0, true)).await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            let execution = cycle.executions[0].clone();
            let prompt = host.next_prompt(&execution).await?;
            super::support::answer(&host, &execution, prompt)?;
            let result = cycle.wait().await?;
            let before = host.execution(&execution)?;
            host.shutdown().await?;
            drop(host);

            // Create Through the Active Host Operation Without Publication.
            let server =
                services::serve(([127, 0, 0, 1], 0).into(), root.path(), super::support::settings(), super::support::service_settings())
                    .await?;
            let endpoint = format!("http://{}", server.endpoint());
            let mut reports = ReportsClient::connect(endpoint.clone()).await?;
            let mut files = FilesClient::connect(endpoint).await?;
            let subject = ReportSubject {
                kind: Some(report_subject::Kind::Execution(execution.clone())),
            };
            let prepared = reports
                .create(ReportsCreateRequest {
                    subject: Some(subject.clone()),
                    document_id: document.metadata.as_ref().unwrap().identifier.clone(),
                    request_id: "document-report".into(),
                })
                .await?
                .into_inner();
            let file = prepared.file.unwrap();
            let store = Store::open(root.path())?;
            assert!(store.reports(&execution)?.is_empty());
            assert_eq!(store.read(&execution)?, before);
            let mut html = Vec::new();
            let mut at = None;
            while (html.len() as u64) < file.length {
                let reply = files
                    .read(FilesReadRequest {
                        file: Some(file.clone()),
                        offset: html.len() as u64,
                        max_bytes: 0,
                        at,
                    })
                    .await?
                    .into_inner();
                assert!(!reply.data.is_empty());
                at = reply.resolved_at;
                html.extend(reply.data);
            }
            let capture = decode_report(&html)?;
            assert_eq!(capture.executions.len(), 1);
            let captured = &capture.executions[0];
            assert_eq!(captured.detail.as_ref().unwrap().execution.as_ref().unwrap().summary.as_ref().unwrap().execution_id, execution);
            assert!(
                captured
                    .plots
                    .as_ref()
                    .unwrap()
                    .envelopes
                    .iter()
                    .any(|plot| plot.diagnostics.as_ref().unwrap().raw_point_count == result.executions[0].samples.len() as u64)
            );
            assert!(!captured.document.as_ref().unwrap().frames.is_empty());
            assert_eq!(
                captured.inputs.as_ref().unwrap().resolved_at.as_ref().unwrap().sequence,
                prepared.resolved_at.as_ref().unwrap().positions[0].sequence
            );

            // Publication Retains the Same Inputs and File Across Restart.
            reports
                .publish(ReportsPublishRequest {
                    name: file.metadata.unwrap().id,
                    subject: Some(subject),
                    expected_revision: file.sha256,
                    request_id: "publish-document-report".into(),
                })
                .await?;
            let published = store.reports(&execution)?.remove(0);
            assert_eq!(published.document, Some(document));
            assert_eq!(published.html, html);
            assert_eq!(store.read(&execution)?, before);
            server.close().await?;
            drop(store);
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            assert_eq!(host.reports(&execution)?, vec![published]);
            assert_eq!(host.execution(&execution)?, before);
            host.shutdown().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
