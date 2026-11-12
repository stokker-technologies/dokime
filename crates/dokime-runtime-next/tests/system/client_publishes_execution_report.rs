/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Report Publication Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime_protocol::{builders::Plot, v1};
use dokime_runtime_next::Host;
use dokime_storage::Store;
use sha2::Digest;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn client_publishes_execution_report() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare Sealed Evidence and Its Selected Document
        let root = tempfile::tempdir()?;
        let host = Host::open(root.path(), super::support::settings())?;
        let entry = host.add_managed(&super::support::execution_fixture()?).await?;
        let document = host.catalog(&entry)?.suites[0].templates[0].documents[0].clone();
        let origin = host.materialize(&entry, "execution.capture", super::support::parameters(23.0, 0, true)).await?;
        let cycle = host.start_cycle(vec![origin]).await?;
        let identifier = cycle.executions[0].clone();
        let prompt = host.next_prompt(&identifier).await?;
        super::support::answer(&host, &identifier, prompt)?;
        cycle.wait().await?;
        let before = host.execution(&identifier)?;
        host.shutdown().await?;
        drop(host);
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut reports = v1::reports_client::ReportsClient::new(channel.clone());
        let mut execution = v1::execution_client::ExecutionClient::new(channel);

        // Render and Download Without Publishing or Mutating Evidence
        let mut rendered = reports
            .download_execution_report(v1::ReportsDownloadExecutionReportRequest { execution_id: identifier.clone() })
            .await?
            .into_inner();
        let mut current = false;
        let mut terminal = false;
        while let Some(frame) = rendered.message().await? {
            match frame.value.unwrap() {
                v1::reports_download_execution_report_output::Value::Item(frame) => {
                    if let v1::report_html_frame_payload::Kind::Report(metadata) = frame.payload.unwrap().kind.unwrap() {
                        current = metadata.origin == v1::ReportOrigin::Current as i32;
                    }
                }
                v1::reports_download_execution_report_output::Value::Response(_) => terminal = true,
            }
        }
        assert!(current && terminal);
        let store = Store::open(root.path())?;
        assert!(store.reports(&identifier)?.is_empty());
        assert_eq!(store.read(&identifier)?, before);

        // Refuse an Invalid Document Before Any Publication
        let invalid = v1::Document::new("Missing Field").section(
            v1::Section::new("Measurements").plot(Plot::time_series(v1::FieldReference::new("execution.reading").field("absent"))),
        );
        let refused = reports
            .publish_execution_report(v1::ReportsPublishExecutionReportRequest {
                execution_id: identifier.clone(),
                document: Some(invalid),
            })
            .await
            .unwrap_err();
        assert_eq!(refused.code(), tonic::Code::InvalidArgument);
        assert!(store.reports(&identifier)?.is_empty());
        let detail = execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner();
        assert_eq!(detail.report.unwrap().state, v1::ReportState::Pending as i32);

        // Publish Explicitly and Freeze the Document, HTML and Execution Seal
        let metadata = reports
            .publish_execution_report(v1::ReportsPublishExecutionReportRequest {
                execution_id: identifier.clone(),
                document: Some(document.clone()),
            })
            .await?
            .into_inner();
        assert_eq!(metadata.origin, v1::ReportOrigin::Sealed as i32);
        let published = store.reports(&identifier)?;
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].document.as_ref(), Some(&document));
        assert_eq!(published[0].identifier, metadata.identifier);
        assert_eq!(published[0].execution_seal, before.seal);
        assert!(std::str::from_utf8(&published[0].html)?.contains("<svg"));
        assert_eq!(store.read(&identifier)?, before);
        let detail = execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner();
        assert_eq!(detail.report.unwrap().state, v1::ReportState::Ready as i32);
        drop(store);
        process.close().await?;

        // Restart and Download Exactly the Published Bytes Without Republishing
        let process = HostProcess::start(root.path()).await?;
        let mut reports = v1::reports_client::ReportsClient::connect(process.url.clone()).await?;
        let mut stream = reports
            .download_execution_report(v1::ReportsDownloadExecutionReportRequest { execution_id: identifier.clone() })
            .await?
            .into_inner();
        let mut bytes = Vec::new();
        let mut completion = None;
        let mut terminal = false;
        while let Some(frame) = stream.message().await? {
            match frame.value.unwrap() {
                v1::reports_download_execution_report_output::Value::Item(frame) => match frame.payload.unwrap().kind.unwrap() {
                    v1::report_html_frame_payload::Kind::Report(value) => assert_eq!(value, metadata),
                    v1::report_html_frame_payload::Kind::Chunk(chunk) => {
                        assert_eq!(chunk.offset, bytes.len() as u64);
                        bytes.extend(chunk.data);
                    }
                    v1::report_html_frame_payload::Kind::Complete(value) => completion = Some(value),
                },
                v1::reports_download_execution_report_output::Value::Response(_) => terminal = true,
            }
        }
        assert!(terminal);
        assert_eq!(bytes, published[0].html);
        let completion = completion.unwrap();
        assert_eq!(completion.size_bytes, bytes.len() as u64);
        assert_eq!(completion.sha256, sha2::Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
        assert_eq!(Store::open(root.path())?.reports(&identifier)?, published);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
