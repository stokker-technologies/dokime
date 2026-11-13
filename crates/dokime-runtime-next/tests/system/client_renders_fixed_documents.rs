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
        let credential_root = tempfile::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        let mut plot = dokime_protocol::builders::Plot::time_series(v1::FieldReference::new("reading").field("min")).message;
        plot.metadata = Some(v1::Metadata {
            identifier: "trend".into(),
            label: "Trend".into(),
            ..Default::default()
        });
        let aggregate = |input: v1::DataExpression, kind: v1::Aggregate, percentile: f64| v1::DataExpression {
            kind: Some(v1::data_expression::Kind::Aggregate(Box::new(v1::AggregateExpression {
                input: Some(Box::new(input)),
                kind: kind as i32,
                percentile,
            }))),
            ..Default::default()
        };
        let record = v1::DataExpression {
            kind: Some(v1::data_expression::Kind::Record(v1::FieldReference {
                identifier: "measurement".into(),
                field: "min".into(),
            })),
            ..Default::default()
        };
        let summary = v1::Summary {
            tiles: vec![
                v1::SummaryTile {
                    metadata: Some(v1::Metadata {
                        identifier: "percentile".into(),
                        ..Default::default()
                    }),
                    value: Some(aggregate(v1::FieldReference::new("reading").field("max"), v1::Aggregate::Percentile, 25.0)),
                    ..Default::default()
                },
                v1::SummaryTile {
                    metadata: Some(v1::Metadata {
                        identifier: "record-mean".into(),
                        ..Default::default()
                    }),
                    value: Some(aggregate(record.clone(), v1::Aggregate::Mean, 0.0)),
                    ..Default::default()
                },
                v1::SummaryTile {
                    metadata: Some(v1::Metadata {
                        identifier: "record-std".into(),
                        ..Default::default()
                    }),
                    value: Some(aggregate(record.clone(), v1::Aggregate::Stddev, 0.0)),
                    ..Default::default()
                },
                v1::SummaryTile {
                    metadata: Some(v1::Metadata {
                        identifier: "record-rms".into(),
                        ..Default::default()
                    }),
                    value: Some(aggregate(record, v1::Aggregate::Rms, 0.0)),
                    ..Default::default()
                },
            ],
        };
        let mut document = v1::Document::new("Measurements").id("measurements").section(
            v1::Section::new("Results")
                .card(v1::Card {
                    metadata: Some(v1::Metadata {
                        identifier: "summary".into(),
                        ..Default::default()
                    }),
                    body: Some(v1::card::Body::Summary(summary)),
                    ..Default::default()
                })
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
        document.sections[0].cards.push(v1::Card {
            metadata: Some(v1::Metadata {
                identifier: "contour-card".into(),
                ..Default::default()
            }),
            body: Some(v1::card::Body::Plot(v1::Plot {
                metadata: Some(v1::Metadata {
                    identifier: "contour".into(),
                    ..Default::default()
                }),
                kind: v1::PlotKind::Contour as i32,
                x: Some(v1::FieldReference::new("reading").field("min")),
                y: vec![v1::FieldReference::new("reading").field("max")],
                z: Some(v1::FieldReference::new("reading").field("min")),
                render: v1::ContourRender::Bands as i32,
                display_points: Some(32),
                ..Default::default()
            })),
            ..Default::default()
        });
        document.sections.push(v1::Section {
            metadata: Some(v1::Metadata {
                identifier: "groups".into(),
                ..Default::default()
            }),
            repeat: Some(v1::SectionRepeat {
                value: Some(v1::FieldReference::new("reading").field("min")),
                label_template: "Group {value}".into(),
                order_by: None,
            }),
            cards: vec![v1::Card {
                metadata: Some(v1::Metadata {
                    identifier: "group-count".into(),
                    ..Default::default()
                }),
                body: Some(v1::card::Body::Summary(v1::Summary {
                    tiles: vec![v1::SummaryTile {
                        value: Some(aggregate(v1::FieldReference::new("reading").field("min"), v1::Aggregate::Count, 0.0)),
                        ..Default::default()
                    }],
                })),
                ..Default::default()
            }],
            ..Default::default()
        });
        for index in 0..1_001 {
            document.sections.push(v1::Section {
                metadata: Some(v1::Metadata {
                    identifier: format!("appendix-{index}"),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
        document.metadata.as_mut().unwrap().attributes = std::collections::HashMap::from([
            ("first".into(), v1::Value { kind: Some(v1::value::Kind::Integer(1)) }),
            ("second".into(), v1::Value { kind: Some(v1::value::Kind::Integer(2)) }),
        ]);
        let suites = v1::Suites {
            suites: vec![v1::Suite {
                templates: vec![v1::TestCaseTemplate {
                    documents: vec![document.clone()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let mut catalog = harmos_protocol::v1::Catalog {
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
        catalog.sections.push(harmos_protocol::v1::Section {
            r#type: "dokime.v1.RecordContracts".into(),
            payload: v1::RecordContracts {
                records: vec![v1::RecordContract {
                    metadata: Some(v1::Metadata {
                        identifier: "measurement".into(),
                        ..Default::default()
                    }),
                    message: "dokime.v1.PlotDomain".into(),
                }],
            }
            .encode_to_vec(),
        });
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
            for value in [2.0, 4.0] {
                store.append(
                    id,
                    dokime_storage::Evidence {
                        kind: "record:measurement".into(),
                        source_ns: 0,
                        payload: v1::PublishRecord {
                            execution: id.into(),
                            kind: "measurement".into(),
                            payload: v1::PlotDomain { min: value, max: value + 7.0 }.encode_to_vec(),
                            ..Default::default()
                        }
                        .encode_to_vec(),
                        ..Default::default()
                    },
                )?;
            }
            store.close_streams(id)?;
            store.close(id, v1::Termination::Completed, v1::ExecutionOutcome::default())?;
            store.seal(id)?;
        }
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
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
        let mut summaries = std::collections::BTreeMap::new();
        let mut grouped_counts = Vec::new();
        let mut next_page = None;
        let mut sections = std::collections::BTreeSet::new();
        let mut table_rows = 0;
        let mut completed = false;
        let mut terminal = false;
        let mut frame_count = 0;
        while let Some(frame) = capture.message().await? {
            match frame.value.unwrap() {
                v1::documents_capture_current_document_output::Value::Item(frame) => {
                    frame_count += 1;
                    match frame.frame.unwrap().kind.unwrap() {
                        v1::document_frame_frame::Kind::Binding(value) => {
                            if let Some(v1::document_binding_result::Kind::Plot(plot)) = value.result.and_then(|result| result.kind)
                                && value.source_id == "trend-card"
                            {
                                binding = plot.request;
                            }
                        }
                        v1::document_frame_frame::Kind::SummaryTile(tile) => {
                            assert!(tile.unavailable.is_none(), "{:?}", tile.unavailable);
                            if tile.card_id.starts_with("group-count.") {
                                grouped_counts.push(tile.value.unwrap());
                            } else {
                                summaries.insert(tile.metadata.unwrap().identifier, tile.value.unwrap());
                            }
                        }
                        v1::document_frame_frame::Kind::TableRow(_) => table_rows += 1,
                        v1::document_frame_frame::Kind::Page(page) => next_page = page.next,
                        v1::document_frame_frame::Kind::LayoutNode(node) => {
                            if let Some(v1::document_layout_node_node::Kind::Section(section)) = node.node.and_then(|node| node.kind) {
                                assert!(sections.insert(section.node_id));
                            }
                        }
                        v1::document_frame_frame::Kind::Completion(completion) => {
                            assert_eq!(completion.frame_count, frame_count);
                            assert_eq!(completion.repeat_count, 1);
                            completed = true;
                        }
                        _ => (),
                    }
                }
                v1::documents_capture_current_document_output::Value::Response(_) => {
                    assert!(completed);
                    terminal = true;
                }
            }
        }
        assert!(terminal);
        assert_eq!(table_rows, 7);
        assert_eq!(summaries["percentile"].kind, Some(v1::value::Kind::Number(2500.5)));
        assert_eq!(summaries["record-mean"].kind, Some(v1::value::Kind::Number(3.0)));
        assert_eq!(summaries["record-std"].kind, Some(v1::value::Kind::Number(1.0)));
        assert_eq!(summaries["record-rms"].kind, Some(v1::value::Kind::Number(10.0f64.sqrt())));
        assert_eq!(grouped_counts.len(), 31);
        assert_eq!(
            grouped_counts
                .iter()
                .map(|value| match value.kind {
                    Some(v1::value::Kind::Integer(value)) => value,
                    _ => panic!("Expected Count"),
                })
                .sum::<i64>(),
            10_003
        );
        let mut pages = 1;
        while let Some(page) = next_page.take() {
            let mut capture = documents
                .capture_current_document(v1::DocumentsCaptureCurrentDocumentRequest {
                    scope: Some(scope.clone()),
                    document_id: "measurements".into(),
                    page: Some(page),
                    ..Default::default()
                })
                .await?
                .into_inner();
            while let Some(frame) = capture.message().await? {
                if let Some(v1::documents_capture_current_document_output::Value::Item(frame)) = frame.value {
                    match frame.frame.unwrap().kind.unwrap() {
                        v1::document_frame_frame::Kind::Page(page) => next_page = page.next,
                        v1::document_frame_frame::Kind::LayoutNode(node) => {
                            if let Some(v1::document_layout_node_node::Kind::Section(section)) = node.node.and_then(|node| node.kind) {
                                assert!(sections.insert(section.node_id));
                            }
                        }
                        _ => (),
                    }
                }
            }
            pages += 1;
        }
        assert!(pages > 1);
        assert_eq!(sections.len(), 1 + 31 + 1_001);
        let mut cycle_document = documents
            .capture_current_document(v1::DocumentsCaptureCurrentDocumentRequest {
                scope: Some(v1::DocumentScope {
                    cycle_id: "documents".into(),
                    execution_id: String::new(),
                }),
                document_id: "default".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let mut cycle_cards = 0;
        while let Some(frame) = cycle_document.message().await? {
            if let Some(v1::documents_capture_current_document_output::Value::Item(frame)) = frame.value {
                match frame.frame.unwrap().kind.unwrap() {
                    v1::document_frame_frame::Kind::Header(header) => assert!(header.scope.unwrap().execution_id.is_none()),
                    v1::document_frame_frame::Kind::Card(_) => cycle_cards += 1,
                    _ => (),
                }
            }
        }
        assert_eq!(cycle_cards, 3);
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
        assert!(std::str::from_utf8(&html)?.contains("data-contour-band="));
        assert!(std::str::from_utf8(&html)?.contains("data-contour-mask"));
        assert!(store.reports("documents-a")?.is_empty());

        // Refuse Unsupported Evidence Filters Without Publishing Misleading Output
        let mut invalid = document.clone();
        invalid.sections[0].cards.push(v1::Card {
            body: Some(v1::card::Body::Evidence(v1::EvidenceCard::Records as i32)),
            filters: vec![v1::DocumentFilter {
                expression: Some(v1::DataExpression {
                    kind: Some(v1::data_expression::Kind::Literal(v1::Value {
                        kind: Some(v1::value::Kind::Boolean(true)),
                    })),
                    ..Default::default()
                }),
                operator: v1::FilterOperator::Eq as i32,
                value: Some(v1::Value {
                    kind: Some(v1::value::Kind::Boolean(true)),
                }),
            }],
            ..Default::default()
        });
        assert_eq!(
            reports
                .publish_execution_report(v1::ReportsPublishExecutionReportRequest {
                    execution_id: "documents-a".into(),
                    document: Some(invalid)
                })
                .await
                .unwrap_err()
                .code(),
            tonic::Code::InvalidArgument
        );
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
        // An Open Member Prevents Reusing a Cycle Capture as Sealed Evidence
        store.create(&Execution {
            identifier: "documents-open".into(),
            cycle: "documents".into(),
            catalog: catalog.encode_to_vec(),
            origin: Some(v1::Origin {
                kind: Some(v1::origin::Kind::Import(v1::Import::default())),
            }),
            ..Default::default()
        })?;
        let mut open_cycle = documents
            .capture_current_document(v1::DocumentsCaptureCurrentDocumentRequest {
                scope: Some(v1::DocumentScope {
                    cycle_id: "documents".into(),
                    execution_id: String::new(),
                }),
                document_id: "default".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let mut cacheable = None;
        while let Some(frame) = open_cycle.message().await? {
            if let Some(v1::documents_capture_current_document_output::Value::Item(frame)) = frame.value
                && let Some(v1::document_frame_frame::Kind::EvidenceCut(cut)) = frame.frame.and_then(|frame| frame.kind)
            {
                cacheable = Some(cut.cacheable);
            }
        }
        assert_eq!(cacheable, Some(false));
        process.close().await?;
        drop(store);
        let reopened = Store::open(root.path())?;
        assert_eq!(reopened.reports("documents-a")?.len(), 1);
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
