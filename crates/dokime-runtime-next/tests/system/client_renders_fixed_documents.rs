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
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
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
                row: "dokime.client.v1.plots.PlotDomain".into(),
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
                    message: "dokime.client.v1.plots.PlotDomain".into(),
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
                    row: dokime_protocol::client::plots::PlotDomain {
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
                        payload: dokime_protocol::context::evidence::RecordsAppendRequest {
                            execution: id.into(),
                            kind: "measurement".into(),
                            payload: dokime_protocol::client::plots::PlotDomain { min: value, max: value + 7.0 }.encode_to_vec(),
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
        let mut documents = dokime_protocol::client::documents_client::DocumentsClient::new(channel.clone());

        let mut preferences = dokime_protocol::client::plots::settings_client::SettingsClient::new(channel.clone());
        let mut reports = dokime_protocol::client::reports_client::ReportsClient::new(channel);
        let scope = v1::DocumentScope {
            cycle_id: "documents".into(),
            execution_id: "documents-a".into(),
        };
        let current = preferences
            .get(dokime_protocol::client::plots::SettingsGetRequest {
                scope: Some(scope.clone()),
                document_id: "measurements".into(),
                plot_id: "trend".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        preferences
            .set(dokime_protocol::client::plots::SettingsSetRequest {
                scope: Some(scope.clone()),
                document_id: "measurements".into(),
                plot_id: "trend".into(),
                value: Some(dokime_protocol::client::plots::PlotSettingsValue {
                    display_points: Some(32),
                    downsample: v1::Downsample::MinMax as i32,
                }),
                expected_revision: current.revision,
                request_id: "plot-settings".into(),
            })
            .await?;

        // Read Bounded Pages at One Cut and Validate the Complete Frame Protocol.
        let mut request = dokime_protocol::client::DocumentsGetRequest {
            scope: Some(scope.clone()),
            document_id: "measurements".into(),
            ..Default::default()
        };
        let first = documents.get(request.clone()).await?.into_inner();
        let mut page = first.clone();
        let mut frames = Vec::new();
        let mut pages = 0;
        loop {
            pages += 1;
            frames.extend(page.frames);
            if page.next_page_token.is_empty() {
                break;
            }
            request.page_token = page.next_page_token;
            request.at = page.resolved_at;
            page = documents.get(request.clone()).await?.into_inner();
        }
        assert!(pages > 1);
        let mut structures = false;
        let mut evidence_cut = false;
        let mut completed = false;
        let mut binding = None;
        let mut cards = std::collections::BTreeSet::new();
        let mut bindings = std::collections::BTreeSet::new();
        let mut finished = std::collections::BTreeSet::new();
        let mut sections = std::collections::BTreeSet::new();
        let mut repeats = std::collections::BTreeSet::new();
        let mut layout_count = 0;
        let mut table_rows = 0;
        let mut summaries = std::collections::BTreeMap::new();
        let mut grouped_counts = Vec::new();
        for (index, frame) in frames.iter().enumerate() {
            match frame.frame.as_ref().unwrap().kind.as_ref().unwrap() {
                dokime_protocol::client::document_frame_frame::Kind::Card(card) => {
                    assert!(!structures);
                    assert!(cards.insert(card.metadata.as_ref().unwrap().identifier.clone()));
                }
                dokime_protocol::client::document_frame_frame::Kind::GroupDefinition(_)
                | dokime_protocol::client::document_frame_frame::Kind::TileDefinition(_) => {
                    assert!(!structures)
                }
                dokime_protocol::client::document_frame_frame::Kind::Binding(value) => {
                    assert_eq!(value.metadata.as_ref().unwrap().identifier, value.source_id);
                    if !structures {
                        bindings.insert(value.source_id.clone());
                    } else {
                        assert!(evidence_cut);
                        assert!(bindings.contains(&value.source_id));
                    }
                    if let Some(dokime_protocol::client::document_binding_result::Kind::Plot(plot)) =
                        value.result.as_ref().and_then(|result| result.kind.as_ref())
                        && value.source_id == "trend-card"
                    {
                        binding = plot.request.clone();
                    }
                }
                dokime_protocol::client::document_frame_frame::Kind::LayoutNode(node) => {
                    assert!(!structures);
                    layout_count += 1;
                    if let Some(dokime_protocol::client::document_layout_node_node::Kind::Section(section)) =
                        node.node.as_ref().and_then(|node| node.kind.as_ref())
                    {
                        assert!(sections.insert(section.node_id.clone()));
                    }
                }
                dokime_protocol::client::document_frame_frame::Kind::RepeatSelection(value) => {
                    assert!(!structures);
                    repeats.insert(value.node_id.clone());
                }
                dokime_protocol::client::document_frame_frame::Kind::StructureComplete(counts) => {
                    assert!(!structures);
                    structures = true;
                    assert_eq!(counts.frame_count, index as u64 + 1);
                    assert_eq!(counts.card_count, cards.len() as u64);
                    assert_eq!(counts.binding_count, bindings.len() as u64);
                    assert_eq!(counts.layout_node_count, layout_count);
                    assert_eq!(counts.section_count, sections.len() as u64);
                    assert_eq!(counts.repeat_count, repeats.len() as u64);
                }
                dokime_protocol::client::document_frame_frame::Kind::EvidenceCut(_) => {
                    assert!(structures);
                    assert!(!evidence_cut);
                    evidence_cut = true;
                }
                dokime_protocol::client::document_frame_frame::Kind::SummaryTile(tile) => {
                    assert!(evidence_cut);
                    assert!(tile.unavailable.is_none(), "{:?}", tile.unavailable);
                    if tile.card_id.starts_with("group-count.") {
                        grouped_counts.push(tile.value.clone().unwrap());
                    } else {
                        summaries.insert(tile.metadata.as_ref().unwrap().identifier.clone(), tile.value.clone().unwrap());
                    }
                }
                dokime_protocol::client::document_frame_frame::Kind::TableRow(_) => {
                    assert!(evidence_cut);
                    table_rows += 1;
                }
                dokime_protocol::client::document_frame_frame::Kind::CardComplete(id) => {
                    assert!(evidence_cut);
                    assert!(bindings.contains(id));
                    assert!(finished.insert(id.clone()));
                }
                dokime_protocol::client::document_frame_frame::Kind::Completion(counts) => {
                    assert!(evidence_cut);
                    assert_eq!(counts.frame_count, index as u64 + 1);
                    assert_eq!(counts.binding_count, bindings.len() as u64);
                    assert_eq!(counts.card_count, cards.len() as u64);
                    completed = true;
                }
                _ => (),
            }
        }
        assert!(completed);
        assert_eq!(cards, bindings);
        assert_eq!(finished, bindings);
        assert_eq!(sections.len(), 1033);
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
            10003
        );
        let plot = binding.unwrap();
        assert_eq!(plot.max_points, Some(32));
        assert!(plot.at.is_some());
        let reply = dokime_protocol::client::plots::snapshots_client::SnapshotsClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::client::plots::SnapshotsListRequest {
                requests: vec![plot],
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(reply.envelopes[0].diagnostics.as_ref().unwrap().examined_rows, 10003);
        assert!(reply.envelopes[0].diagnostics.as_ref().unwrap().point_count <= 32);
        assert!(store.reports("documents-a")?.is_empty());

        // Replaying a Cut Preserves Structure and Its Page Boundary.
        let replay = documents
            .get(dokime_protocol::client::DocumentsGetRequest {
                scope: Some(scope.clone()),
                document_id: "measurements".into(),
                at: first.resolved_at.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(replay, first);
        let mut structure = dokime_protocol::client::DocumentsGetRequest {
            scope: Some(scope),
            document_id: "measurements".into(),
            structure_only: true,
            ..Default::default()
        };
        let mut cut_seen = false;
        loop {
            let page = documents.get(structure.clone()).await?.into_inner();
            for frame in page.frames {
                match frame.frame.unwrap().kind.unwrap() {
                    dokime_protocol::client::document_frame_frame::Kind::EvidenceCut(_) => cut_seen = true,
                    dokime_protocol::client::document_frame_frame::Kind::CardComplete(_)
                    | dokime_protocol::client::document_frame_frame::Kind::SummaryTile(_)
                    | dokime_protocol::client::document_frame_frame::Kind::TableRow(_) => panic!("Structure-Only Capture Evaluated a Card"),
                    _ => (),
                }
            }
            if page.next_page_token.is_empty() {
                break;
            }
            structure.at = page.resolved_at;
            structure.page_token = page.next_page_token;
        }
        assert!(cut_seen);

        // Preparing Never Publishes; Publication Requires the Exact Prepared Revision.
        let subject = dokime_protocol::client::ReportSubject {
            kind: Some(dokime_protocol::client::report_subject::Kind::Execution("documents-a".into())),
        };
        let prepared = reports
            .create(dokime_protocol::client::ReportsCreateRequest {
                subject: Some(subject.clone()),
                document_id: "measurements".into(),
                request_id: "preview".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let reference = prepared.file.unwrap();
        let name = reference.metadata.as_ref().unwrap().id.clone();
        let frozen: dokime_storage::Report = store.administration(&name)?.unwrap();
        use sha2::Digest;
        assert_eq!(sha2::Sha256::digest(&frozen.html).as_slice(), reference.sha256);
        assert!(std::str::from_utf8(&frozen.html)?.contains("<svg"));
        assert!(store.reports("documents-a")?.is_empty());
        let mut publish = dokime_protocol::client::ReportsPublishRequest {
            name,
            subject: Some(subject),
            expected_revision: vec![0],
            request_id: "publish".into(),
        };
        assert!(reports.publish(publish.clone()).await.is_err());
        assert!(store.reports("documents-a")?.is_empty());
        publish.expected_revision = reference.sha256;
        reports.publish(publish).await?;
        assert_eq!(store.reports("documents-a")?.len(), 1);
        let cycle = reports
            .create(dokime_protocol::client::ReportsCreateRequest {
                subject: Some(dokime_protocol::client::ReportSubject {
                    kind: Some(dokime_protocol::client::report_subject::Kind::Cycle("documents".into())),
                }),
                request_id: "cycle-preview".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(cycle.file.unwrap().length > 0);
        assert!(store.reports("documents-b")?.is_empty());
        store.create(&Execution {
            identifier: "documents-open".into(),
            cycle: "documents".into(),
            catalog: catalog.encode_to_vec(),
            origin: Some(v1::Origin {
                kind: Some(v1::origin::Kind::Import(v1::Import::default())),
            }),
            ..Default::default()
        })?;
        let mut cycle_request = dokime_protocol::client::DocumentsGetRequest {
            scope: Some(v1::DocumentScope {
                cycle_id: "documents".into(),
                execution_id: String::new(),
            }),
            document_id: "default".into(),
            ..Default::default()
        };
        let mut cacheable = None;
        loop {
            let page = documents.get(cycle_request.clone()).await?.into_inner();
            for frame in page.frames {
                if let Some(dokime_protocol::client::document_frame_frame::Kind::EvidenceCut(cut)) =
                    frame.frame.and_then(|frame| frame.kind)
                {
                    cacheable = Some(cut.cacheable);
                }
            }
            if page.next_page_token.is_empty() {
                break;
            }
            cycle_request.at = page.resolved_at;
            cycle_request.page_token = page.next_page_token;
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
