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
use std::{error::Error, fs};

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD};
use prost::Message;
use sha2::{Digest, Sha256};

// Local Imports
use dokime_operations::host::plots::snapshots::list;
use dokime_protocol::host::documents_client::DocumentsClient;
use dokime_protocol::host::files_client::FilesClient;
use dokime_protocol::host::plots::settings_client::SettingsClient;
use dokime_protocol::host::plots::snapshots_client::SnapshotsClient;
use dokime_protocol::host::plots::{PlotRequest, PlotSettingsValue, SettingsGetRequest, SettingsSetRequest, SnapshotsListRequest};
use dokime_protocol::host::reports_client::ReportsClient;
use dokime_protocol::host::{
    DocumentFrame, DocumentScope, DocumentsGetRequest, FilesReadRequest, ReportCapture, ReportSubject, ReportsCreateRequest,
    document_binding_result, document_frame_frame, report_subject,
};
use dokime_protocol::model::{Downsample, PlotHistogramOptions, PlotKind, card};
use dokime_runtime::{Host, HostSettings, services};
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
            let origin = host.materialize(&entry, "execution.execution.capture", super::support::parameters(23.0, 0, true)).await?;
            let cycle = host.start_cycle(vec![origin]).await?;
            let execution = cycle.executions[0].clone();
            let cycle_id = cycle.identifier.clone();
            let prompt = host.next_prompt(&execution).await?;
            super::support::answer(&host, &execution, prompt)?;
            let result = cycle.wait().await?;
            let before = host.execution(&execution)?;
            // Sealing Froze the Default Document's Report Before the Cycle Completed.
            let frozen = host.reports(&execution)?;
            assert_eq!(frozen.len(), 1);
            assert_eq!(frozen[0].document.as_ref(), Some(&document));
            host.shutdown().await?;
            drop(host);

            // Preview the Named Document Through a Host Whose Installation Brands Its Reports.
            let installation = super::support::tempdir()?;
            let pack = installation.path().join("organization");
            fs::create_dir_all(pack.join("theme"))?;
            fs::write(pack.join("organization.toml"), "[app]\nlabel = 'Example Lab'\n[app.logos.transparent]\non_light = 'mark.svg'\n")?;
            fs::write(pack.join("mark.svg"), "<svg xmlns='http://www.w3.org/2000/svg'/>")?;
            fs::write(pack.join("sans.woff2"), [0, 1, 2])?;
            fs::write(pack.join("theme/typography.css"), "@font-face { font-family: Example; src: url(sans.woff2) }")?;
            let settings = HostSettings {
                installation: Some(installation.path().to_path_buf()),
                ..super::support::settings()
            };
            let server = services::serve(([127, 0, 0, 1], 0).into(), root.path(), settings, super::support::service_settings()).await?;
            let endpoint = format!("http://{}", server.endpoint());
            let mut reports = ReportsClient::connect(endpoint.clone()).await?;
            let mut files = FilesClient::connect(endpoint.clone()).await?;
            let mut documents = DocumentsClient::connect(endpoint.clone()).await?;
            let mut settings = SettingsClient::connect(endpoint.clone()).await?;
            let mut plots = SnapshotsClient::connect(endpoint).await?;

            // Save Native Plot Preferences Before Capturing Either Reader.
            let document_id = document.metadata.as_ref().unwrap().identifier.clone();
            let mut sections = document.sections.iter().collect::<Vec<_>>();
            let mut plot_id = None;
            while let Some(section) = sections.pop() {
                for card in &section.cards {
                    if let Some(card::Body::Plot(plot)) = card.body.as_ref() {
                        plot_id = Some(plot.metadata.as_ref().unwrap().identifier.clone());
                        break;
                    }
                }
                if plot_id.is_some() {
                    break;
                }
                sections.extend(section.sections.iter());
            }
            let plot_id = plot_id.ok_or("Fixture Has No Plot")?;
            let scope = DocumentScope {
                cycle_id: cycle_id.clone(),
                execution_id: execution.clone(),
            };
            let current = settings
                .get(SettingsGetRequest {
                    scope: Some(scope.clone()),
                    document_id: document_id.clone(),
                    plot_id: plot_id.clone(),
                    at: None,
                })
                .await?
                .into_inner();
            let preferences = PlotSettingsValue {
                display_points: Some(3),
                downsample: Downsample::MinMax as i32,
            };
            let saved = settings
                .set(SettingsSetRequest {
                    scope: Some(scope.clone()),
                    document_id: document_id.clone(),
                    plot_id: plot_id.clone(),
                    value: Some(preferences),
                    expected_revision: current.revision,
                    request_id: "report-plot-preferences".into(),
                })
                .await?
                .into_inner();
            assert_eq!(saved.value, Some(preferences));
            let plot_request = |frame: &DocumentFrame| -> Option<PlotRequest> {
                let document_frame_frame::Kind::Binding(binding) = frame.frame.as_ref()?.kind.as_ref()? else {
                    return None;
                };
                let document_binding_result::Kind::Plot(plot) = binding.result.as_ref()?.kind.as_ref()? else {
                    return None;
                };
                (plot.metadata.as_ref()?.identifier == plot_id).then(|| plot.request.clone()).flatten()
            };
            let mut live_plot = None;
            let mut live_frames = Vec::new();
            let mut page_token = String::new();
            let mut at = None;
            loop {
                let reply = documents
                    .get(DocumentsGetRequest {
                        scope: Some(scope.clone()),
                        document_id: document_id.clone(),
                        page_token,
                        at,
                        ..Default::default()
                    })
                    .await?
                    .into_inner();
                live_plot = live_plot.or_else(|| reply.frames.iter().find_map(&plot_request));
                live_frames.extend(reply.frames);
                if reply.next_page_token.is_empty() {
                    break;
                }
                page_token = reply.next_page_token;
                at = reply.resolved_at;
            }
            let live_plot = live_plot.ok_or("Document Has No Plot Binding")?;
            assert_eq!(live_plot.max_points, preferences.display_points.map(u64::from));
            assert_eq!(live_plot.technique, preferences.downsample);
            let subject = ReportSubject {
                kind: Some(report_subject::Kind::Execution(execution.clone())),
            };
            let prepared = reports
                .create(ReportsCreateRequest {
                    subject: Some(subject.clone()),
                    document_id: document_id.clone(),
                    request_id: "document-report".into(),
                })
                .await?
                .into_inner();
            let file = prepared.file.unwrap();
            let store = Store::open(root.path())?;
            assert_eq!(store.reports(&execution)?, frozen);
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
            let appearance = capture.appearance.as_ref().and_then(|reply| reply.appearance.as_ref()).ok_or("Report Has No Appearance")?;
            assert_eq!(appearance.app_name, "Example Lab");
            assert!(appearance.themes[0].stylesheet().contains("url(\"data:font/woff2;base64,AAEC\")"));
            assert!(appearance.logos[0].data_uri.starts_with("data:image/svg+xml;base64,"));
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
            let captured_document = captured.document.as_ref().unwrap();
            assert!(!captured_document.frames.is_empty());
            assert_eq!(
                captured_document.frames.iter().map(Message::encode_to_vec).collect::<Vec<_>>(),
                live_frames.iter().map(Message::encode_to_vec).collect::<Vec<_>>()
            );
            let captured_plot = captured_document.frames.iter().find_map(&plot_request).ok_or("Report Has No Plot Binding")?;
            assert_eq!(captured_plot, live_plot);
            assert_eq!(
                captured.inputs.as_ref().unwrap().resolved_at.as_ref().unwrap().sequence,
                prepared.resolved_at.as_ref().unwrap().positions[0].sequence
            );

            // Compare Embedded Bytes With a Separate Batch at the Fixed Fixture Cut.
            let fixed = store.cut(&execution)?;
            let (reader, _cancellation) = store.read_session()?;
            let envelope = &captured.plots.as_ref().unwrap().envelopes[0];
            let expected = list::snapshots_at(
                &reader,
                &fixed,
                vec![captured_plot.clone()],
                super::support::service_settings().page_items,
                0,
                &envelope.epoch,
            )
            .await?;
            assert_eq!(envelope.encode_to_vec(), expected[0].encode_to_vec());

            // Duplicate and Independent Selections Preserve Bytes and Request Order in the Shared Batch Path.
            let mut duplicate = captured_plot.clone();
            duplicate.plot_id = Some("duplicate".into());
            let mut histogram = captured_plot.clone();
            histogram.plot_id = Some("histogram".into());
            histogram.plot_type = PlotKind::Histogram as i32;
            histogram.histogram = Some(PlotHistogramOptions { bins: 3, ..Default::default() });
            let requests = vec![histogram, captured_plot.clone(), duplicate];
            let mut expected = Vec::new();
            for request in &requests {
                let reader = reader.independent_projection();
                expected.extend(
                    list::snapshots_at(
                        &reader,
                        &fixed,
                        vec![request.clone()],
                        super::support::service_settings().page_items,
                        0,
                        &envelope.epoch,
                    )
                    .await?,
                );
            }
            let batch = plots.list(SnapshotsListRequest { requests, ..Default::default() }).await?.into_inner();
            assert_eq!(
                batch.envelopes.iter().map(Message::encode_to_vec).collect::<Vec<_>>(),
                expected.iter().map(Message::encode_to_vec).collect::<Vec<_>>()
            );

            // Download Delivers the Report Frozen at Seal, Unchanged Across Restart.
            let downloaded = reports
                .create(ReportsCreateRequest {
                    subject: Some(subject),
                    document_id: String::new(),
                    request_id: "download-report".into(),
                })
                .await?
                .into_inner();
            assert_eq!(downloaded.report.unwrap().identifier, frozen[0].identifier);
            assert_eq!(downloaded.file.unwrap().sha256, Sha256::digest(&frozen[0].html).as_slice());
            assert_eq!(store.reports(&execution)?, frozen);
            assert_eq!(store.read(&execution)?, before);
            server.close().await?;
            drop(store);
            let host = Host::open(root.path(), super::support::settings(), super::support::service_settings()).await?;
            assert_eq!(host.reports(&execution)?, frozen);
            assert_eq!(host.execution(&execution)?, before);
            host.shutdown().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
