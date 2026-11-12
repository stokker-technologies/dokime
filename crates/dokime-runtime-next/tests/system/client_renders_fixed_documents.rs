/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Queries Every Plot Kind

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{encoding::Message, v1};
use dokime_storage::{Execution, Sample, Store};

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
async fn client_renders_fixed_documents() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare a Frozen Catalog with an Authored Document
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let mut plot = dokime_protocol::builders::Plot::time_series(v1::FieldReference::new("reading").field("min")).message;
        plot.metadata = Some(v1::Metadata {
            identifier: "trend".into(),
            label: "Trend".into(),
            ..Default::default()
        });
        let document = v1::Document::new("Measurements").id("measurements").section(
            v1::Section::new("Results")
                .card(v1::Card {
                    metadata: Some(v1::Metadata {
                        identifier: "trend-card".into(),
                        ..Default::default()
                    }),
                    body: Some(v1::card::Body::Plot(plot)),
                    ..Default::default()
                })
                .card(v1::Card {
                    metadata: Some(v1::Metadata {
                        identifier: "table".into(),
                        ..Default::default()
                    }),
                    body: Some(v1::card::Body::Table(v1::Table {
                        rows: vec![
                            v1::TableRow {
                                cells: std::collections::HashMap::from([(
                                    "value".into(),
                                    v1::Value {
                                        kind: Some(v1::value::Kind::Number(23.0))
                                    }
                                )])
                            };
                            7
                        ],
                    })),
                    ..Default::default()
                }),
        );
        let suites = v1::Suites {
            suites: vec![v1::Suite {
                templates: vec![v1::TestCaseTemplate {
                    documents: vec![document.clone()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let catalog = harmos_protocol::v1::Catalog {
            types: Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            streams: vec![harmos_protocol::v1::StreamDeclaration {
                id: "reading".into(),
                row: "dokime.v1.PlotDomain".into(),
                ..Default::default()
            }],
            sections: vec![harmos_protocol::v1::Section {
                r#type: "dokime.v1.Suites".into(),
                payload: suites.encode_to_vec(),
            }],
            ..Default::default()
        };
        for id in ["documents-a", "documents-b"] {
            store.create(&Execution {
                identifier: id.into(),
                cycle: "documents".into(),
                catalog: catalog.encode_to_vec(),
                origin: Some(v1::Origin {
                    kind: Some(v1::origin::Kind::Import(v1::Import::default())),
                }),
                ..Default::default()
            })?;
            store.open_stream(id, "reading")?;
            let samples = (0..10_003)
                .map(|index| Sample {
                    stream: "reading".into(),
                    source_ns: index * 1_000_000_000,
                    row: v1::PlotDomain {
                        min: (index % 31) as f64,
                        max: index as f64,
                    }
                    .encode_to_vec(),
                    ..Default::default()
                })
                .collect::<Vec<_>>();
            store.sample_batch(id, &samples)?;
            store.close_streams(id)?;
            store.close(id, v1::Termination::Completed, v1::ExecutionOutcome::default())?;
            store.seal(id)?;
        }
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut documents = v1::documents_client::DocumentsClient::new(channel.clone());
        let mut plots = v1::plots_client::PlotsClient::new(channel.clone());
        let mut reports = v1::reports_client::ReportsClient::new(channel.clone());
        let mut content = v1::content_client::ContentClient::new(channel.clone());
        let scope = v1::DocumentScope {
            cycle_id: "documents".into(),
            execution_id: "documents-a".into(),
        };

        // Persist Plot Preferences and Capture Structure Before Data
        documents
            .update_plot_settings(v1::DocumentsUpdatePlotSettingsRequest {
                scope: Some(scope.clone()),
                document_id: "measurements".into(),
                plot_id: "trend".into(),
                display_points: Some(32),
                downsample: v1::Downsample::MinMax as i32,
            })
            .await?;
        let mut capture = documents
            .capture_current_document(v1::DocumentsCaptureCurrentDocumentRequest {
                scope: Some(scope.clone()),
                document_id: "measurements".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let mut binding = None;
        let mut table_rows = 0;
        let mut completed = false;
        let mut terminal = false;
        while let Some(frame) = capture.message().await? {
            match frame.value.unwrap() {
                v1::documents_capture_current_document_output::Value::Item(frame) => match frame.frame.unwrap().kind.unwrap() {
                    v1::document_frame_frame::Kind::Binding(value) => {
                        if let Some(v1::document_binding_result::Kind::Plot(plot)) = value.result.and_then(|result| result.kind) {
                            binding = plot.request;
                        }
                    }
                    v1::document_frame_frame::Kind::TableRow(_) => table_rows += 1,
                    v1::document_frame_frame::Kind::Completion(_) => completed = true,
                    _ => (),
                },
                v1::documents_capture_current_document_output::Value::Response(_) => {
                    assert!(completed);
                    terminal = true;
                }
            }
        }
        assert!(terminal);
        assert_eq!(table_rows, 7);
        let binding = binding.unwrap();
        assert_eq!(binding.max_points, Some(32));
        assert!(binding.at.is_some());
        let reply = plots.query_snapshots(v1::PlotsQuerySnapshotsRequest { requests: vec![binding] }).await?.into_inner();
        assert_eq!(reply.envelopes[0].diagnostics.as_ref().unwrap().examined_rows, 10_003);
        assert!(reply.envelopes[0].diagnostics.as_ref().unwrap().point_count <= 32);
        assert!(store.reports("documents-a")?.is_empty());

        // Materialize Temporary Output and Download It Without Publishing
        let rendered = reports
            .materialize_report(v1::ReportsMaterializeReportRequest {
                subject: Some(v1::ReportSubject {
                    kind: Some(v1::report_subject::Kind::Execution("documents-a".into())),
                }),
                at: None,
            })
            .await?
            .into_inner()
            .report
            .unwrap();
        let mut download = content
            .download_artifact(v1::ContentDownloadArtifactRequest {
                artifact_id: rendered.metadata.unwrap().id,
            })
            .await?
            .into_inner();
        let mut html = Vec::new();
        while let Some(frame) = download.message().await? {
            if let Some(v1::content_download_artifact_output::Value::Item(frame)) = frame.value
                && let Some(v1::artifact_frame_payload::Kind::Chunk(chunk)) = frame.payload.and_then(|payload| payload.kind)
            {
                html.extend(chunk.data);
            }
        }
        use sha2::Digest;
        assert_eq!(sha2::Sha256::digest(&html).as_slice(), rendered.sha256);
        assert!(std::str::from_utf8(&html)?.contains("<svg"));
        assert!(store.reports("documents-a")?.is_empty());

        // Publish Explicitly and Download an Independent Cycle Report
        reports
            .publish_execution_report(v1::ReportsPublishExecutionReportRequest {
                execution_id: "documents-a".into(),
                document: Some(document),
            })
            .await?;
        assert_eq!(store.reports("documents-a")?.len(), 1);
        let mut cycle = reports
            .download_cycle_report(v1::ReportsDownloadCycleReportRequest {
                cycle_id: "documents".into(),
                identifier: "cycle".into(),
            })
            .await?
            .into_inner();
        let mut completion = false;
        let mut terminal = false;
        while let Some(frame) = cycle.message().await? {
            match frame.value.unwrap() {
                v1::reports_download_cycle_report_output::Value::Item(frame) => {
                    if matches!(frame.payload.and_then(|payload| payload.kind), Some(v1::report_html_frame_payload::Kind::Complete(_))) {
                        completion = true;
                    }
                }
                v1::reports_download_cycle_report_output::Value::Response(_) => {
                    assert!(completion);
                    terminal = true;
                }
            }
        }
        assert!(terminal);
        assert!(store.reports("documents-b")?.is_empty());
        process.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
