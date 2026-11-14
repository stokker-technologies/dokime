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
use std::error::Error;

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
async fn client_renders_fixed_documents() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare a Frozen Catalog with an Authored Document
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let store = Store::open(root.path())?;
        let mut plot =
            dokime_protocol::builders::Plot::time_series(dokime_protocol::model::FieldReference::new("reading").field("min")).message;
        plot.metadata = Some(dokime_protocol::model::Metadata {
            identifier: "trend".into(),
            label: "Trend".into(),
            ..Default::default()
        });
        let aggregate = |input: dokime_protocol::model::DataExpression, kind: dokime_protocol::model::Aggregate, percentile: f64| {
            dokime_protocol::model::DataExpression {
                kind: Some(model::data_expression::Kind::Aggregate(Box::new(dokime_protocol::model::AggregateExpression {
                    input: Some(Box::new(input)),
                    kind: kind as i32,
                    percentile,
                }))),
                ..Default::default()
            }
        };
        let record = dokime_protocol::model::DataExpression {
            kind: Some(model::data_expression::Kind::Record(dokime_protocol::model::FieldReference {
                identifier: "measurement".into(),
                field: "min".into(),
            })),
            ..Default::default()
        };
        let summary = dokime_protocol::model::Summary {
            tiles: vec![
                dokime_protocol::model::SummaryTile {
                    metadata: Some(dokime_protocol::model::Metadata {
                        identifier: "percentile".into(),
                        ..Default::default()
                    }),
                    value: Some(aggregate(
                        dokime_protocol::model::FieldReference::new("reading").field("max"),
                        dokime_protocol::model::Aggregate::Percentile,
                        25.0,
                    )),
                    ..Default::default()
                },
                dokime_protocol::model::SummaryTile {
                    metadata: Some(dokime_protocol::model::Metadata {
                        identifier: "record-mean".into(),
                        ..Default::default()
                    }),
                    value: Some(aggregate(record.clone(), dokime_protocol::model::Aggregate::Mean, 0.0)),
                    ..Default::default()
                },
                dokime_protocol::model::SummaryTile {
                    metadata: Some(dokime_protocol::model::Metadata {
                        identifier: "record-std".into(),
                        ..Default::default()
                    }),
                    value: Some(aggregate(record.clone(), dokime_protocol::model::Aggregate::Stddev, 0.0)),
                    ..Default::default()
                },
                dokime_protocol::model::SummaryTile {
                    metadata: Some(dokime_protocol::model::Metadata {
                        identifier: "record-rms".into(),
                        ..Default::default()
                    }),
                    value: Some(aggregate(record, dokime_protocol::model::Aggregate::Rms, 0.0)),
                    ..Default::default()
                },
            ],
        };
        let mut document = dokime_protocol::model::Document::new("Measurements").id("measurements").section(
            dokime_protocol::model::Section::new("Results")
                .card(dokime_protocol::model::Card {
                    metadata: Some(dokime_protocol::model::Metadata {
                        identifier: "summary".into(),
                        ..Default::default()
                    }),
                    body: Some(model::card::Body::Summary(summary)),
                    ..Default::default()
                })
                .card(dokime_protocol::model::Card {
                    metadata: Some(dokime_protocol::model::Metadata {
                        identifier: "trend-card".into(),
                        ..Default::default()
                    }),
                    body: Some(model::card::Body::Plot(plot)),
                    ..Default::default()
                })
                .card(dokime_protocol::model::Card {
                    metadata: Some(dokime_protocol::model::Metadata {
                        identifier: "table".into(),
                        ..Default::default()
                    }),
                    body: Some(model::card::Body::Table(dokime_protocol::model::Table {
                        rows: vec![
                            dokime_protocol::model::TableRow {
                                cells: std::collections::HashMap::from([(
                                    "value".into(),
                                    dokime_protocol::model::Value {
                                        kind: Some(model::value::Kind::Number(23.0))
                                    }
                                )])
                            };
                            7
                        ],
                    })),
                    ..Default::default()
                }),
        );
        document.sections[0].cards.push(dokime_protocol::model::Card {
            metadata: Some(dokime_protocol::model::Metadata {
                identifier: "contour-card".into(),
                ..Default::default()
            }),
            body: Some(model::card::Body::Plot(dokime_protocol::model::Plot {
                metadata: Some(dokime_protocol::model::Metadata {
                    identifier: "contour".into(),
                    ..Default::default()
                }),
                kind: dokime_protocol::model::PlotKind::Contour as i32,
                x: Some(dokime_protocol::model::FieldReference::new("reading").field("min")),
                y: vec![dokime_protocol::model::FieldReference::new("reading").field("max")],
                z: Some(dokime_protocol::model::FieldReference::new("reading").field("min")),
                render: dokime_protocol::model::ContourRender::Bands as i32,
                display_points: Some(32),
                ..Default::default()
            })),
            ..Default::default()
        });
        document.sections.push(dokime_protocol::model::Section {
            metadata: Some(dokime_protocol::model::Metadata {
                identifier: "groups".into(),
                ..Default::default()
            }),
            repeat: Some(dokime_protocol::model::SectionRepeat {
                value: Some(dokime_protocol::model::FieldReference::new("reading").field("min")),
                label_template: "Group {value}".into(),
                order_by: None,
            }),
            cards: vec![dokime_protocol::model::Card {
                metadata: Some(dokime_protocol::model::Metadata {
                    identifier: "group-count".into(),
                    ..Default::default()
                }),
                body: Some(model::card::Body::Summary(dokime_protocol::model::Summary {
                    tiles: vec![dokime_protocol::model::SummaryTile {
                        value: Some(aggregate(
                            dokime_protocol::model::FieldReference::new("reading").field("min"),
                            dokime_protocol::model::Aggregate::Count,
                            0.0,
                        )),
                        ..Default::default()
                    }],
                })),
                ..Default::default()
            }],
            ..Default::default()
        });
        for index in 0..1_001 {
            document.sections.push(dokime_protocol::model::Section {
                metadata: Some(dokime_protocol::model::Metadata {
                    identifier: format!("appendix-{index}"),
                    ..Default::default()
                }),
                ..Default::default()
            });
        }
        document.metadata.as_mut().unwrap().attributes = std::collections::HashMap::from([
            (
                "first".into(),
                dokime_protocol::model::Value {
                    kind: Some(model::value::Kind::Integer(1)),
                },
            ),
            (
                "second".into(),
                dokime_protocol::model::Value {
                    kind: Some(model::value::Kind::Integer(2)),
                },
            ),
        ]);
        let suites = dokime_protocol::model::Suites {
            suites: vec![dokime_protocol::model::Suite {
                templates: vec![dokime_protocol::model::TestCaseTemplate {
                    documents: vec![document.clone()],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };
        let mut catalog = harmos_protocol::model::Declarations {
            types: Some(prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?),
            streams: vec![harmos_protocol::model::StreamDeclaration {
                id: "reading".into(),
                row: "dokime.host.v1.plots.PlotDomain".into(),
                ..Default::default()
            }],
            sections: vec![harmos_protocol::model::Section {
                r#type: "dokime.model.v1.Suites".into(),
                payload: suites.encode_to_vec(),
            }],
            ..Default::default()
        };
        catalog.sections.push(harmos_protocol::model::Section {
            r#type: "dokime.model.v1.RecordContracts".into(),
            payload: dokime_protocol::model::RecordContracts {
                records: vec![dokime_protocol::model::RecordContract {
                    metadata: Some(dokime_protocol::model::Metadata {
                        identifier: "measurement".into(),
                        ..Default::default()
                    }),
                    message: "dokime.host.v1.plots.PlotDomain".into(),
                }],
            }
            .encode_to_vec(),
        });
        for id in ["documents-a", "documents-b"] {
            store.create(&Execution {
                identifier: id.into(),
                cycle: "documents".into(),
                catalog: catalog.encode_to_vec(),
                origin: Some(dokime_protocol::model::Origin {
                    kind: Some(model::origin::Kind::Import(dokime_protocol::model::Import::default())),
                }),
                ..Default::default()
            })?;
            store.open_stream(id, "reading")?;
            let samples = (0..10_003)
                .map(|index| Sample {
                    stream: "reading".into(),
                    source_ns: index * 1_000_000_000,
                    row: dokime_protocol::host::plots::PlotDomain {
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
                        payload: dokime_protocol::host::evidence::RecordsAppendRequest {
                            execution: id.into(),
                            kind: "measurement".into(),
                            payload: dokime_protocol::host::plots::PlotDomain { min: value, max: value + 7.0 }.encode_to_vec(),
                            ..Default::default()
                        }
                        .encode_to_vec(),
                        ..Default::default()
                    },
                )?;
            }
            store.close_streams(id)?;
            store.close(id, dokime_protocol::model::Termination::Completed, dokime_protocol::host::ExecutionOutcome::default())?;
            store.seal(id)?;
        }
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut documents = dokime_protocol::host::documents_client::DocumentsClient::new(channel.clone());

        let mut preferences = dokime_protocol::host::plots::settings_client::SettingsClient::new(channel.clone());
        let mut reports = dokime_protocol::host::reports_client::ReportsClient::new(channel);
        let scope = dokime_protocol::host::DocumentScope {
            cycle_id: "documents".into(),
            execution_id: "documents-a".into(),
        };
        let current = preferences
            .get(dokime_protocol::host::plots::SettingsGetRequest {
                scope: Some(scope.clone()),
                document_id: "measurements".into(),
                plot_id: "trend".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        preferences
            .set(dokime_protocol::host::plots::SettingsSetRequest {
                scope: Some(scope.clone()),
                document_id: "measurements".into(),
                plot_id: "trend".into(),
                value: Some(dokime_protocol::host::plots::PlotSettingsValue {
                    display_points: Some(32),
                    downsample: dokime_protocol::model::Downsample::MinMax as i32,
                }),
                expected_revision: current.revision,
                request_id: "plot-settings".into(),
            })
            .await?;

        // Read Bounded Pages at One Cut and Validate the Complete Frame Protocol.
        let mut request = dokime_protocol::host::DocumentsGetRequest {
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
        let mut section_identities = std::collections::BTreeSet::new();
        let mut repeats = std::collections::BTreeSet::new();
        let mut layout_count = 0;
        let mut table_rows = 0;
        let mut summaries = std::collections::BTreeMap::new();
        let mut grouped_counts = Vec::new();
        let mut ready = std::collections::BTreeSet::new();
        for (index, frame) in frames.iter().enumerate() {
            match frame.frame.as_ref().unwrap().kind.as_ref().unwrap() {
                dokime_protocol::host::document_frame_frame::Kind::Card(card) => {
                    assert!(!structures);
                    assert!(cards.insert(card.metadata.as_ref().unwrap().identifier.clone()));
                }
                dokime_protocol::host::document_frame_frame::Kind::GroupDefinition(_)
                | dokime_protocol::host::document_frame_frame::Kind::TileDefinition(_) => {
                    assert!(!structures)
                }
                dokime_protocol::host::document_frame_frame::Kind::Binding(value) => {
                    assert_eq!(value.metadata.as_ref().unwrap().identifier, value.source_id);
                    if !structures {
                        assert!(matches!(
                            value.result.as_ref().and_then(|result| result.kind.as_ref()),
                            Some(dokime_protocol::host::document_binding_result::Kind::Pending(true))
                        ));
                        bindings.insert(value.source_id.clone());
                    } else {
                        assert!(evidence_cut);
                        assert!(bindings.contains(&value.source_id));
                        assert!(ready.insert(value.source_id.clone()));
                    }
                    if let Some(dokime_protocol::host::document_binding_result::Kind::Plot(plot)) =
                        value.result.as_ref().and_then(|result| result.kind.as_ref())
                        && value.source_id == "trend-card"
                    {
                        binding = plot.request.clone();
                    }
                }
                dokime_protocol::host::document_frame_frame::Kind::SectionIdentity(section) => {
                    assert!(!structures);
                    assert!(!section.node_id.is_empty());
                    assert_eq!(section.metadata.as_ref().unwrap().identifier, section.node_id);
                    assert!(section_identities.insert(section.node_id.clone()));
                }
                dokime_protocol::host::document_frame_frame::Kind::LayoutNode(node) => {
                    assert!(!structures);
                    layout_count += 1;
                    if let Some(dokime_protocol::host::document_layout_node_node::Kind::Section(section)) =
                        node.node.as_ref().and_then(|node| node.kind.as_ref())
                    {
                        assert!(sections.insert(section.node_id.clone()));
                    }
                }
                dokime_protocol::host::document_frame_frame::Kind::RepeatSelection(value) => {
                    assert!(!structures);
                    repeats.insert(value.node_id.clone());
                }
                dokime_protocol::host::document_frame_frame::Kind::StructureComplete(counts) => {
                    assert!(!structures);
                    structures = true;
                    assert_eq!(counts.frame_count, index as u64 + 1);
                    assert_eq!(counts.card_count, cards.len() as u64);
                    assert_eq!(counts.binding_count, bindings.len() as u64);
                    assert_eq!(counts.layout_node_count, layout_count);
                    assert_eq!(counts.section_count, sections.len() as u64);
                    assert_eq!(section_identities, sections);
                    assert_eq!(counts.repeat_count, repeats.len() as u64);
                }
                dokime_protocol::host::document_frame_frame::Kind::EvidenceCut(_) => {
                    assert!(structures);
                    assert!(!evidence_cut);
                    evidence_cut = true;
                }
                dokime_protocol::host::document_frame_frame::Kind::SummaryGroup(group) => {
                    assert!(evidence_cut);
                    assert!(ready.contains(&group.card_id));
                }
                dokime_protocol::host::document_frame_frame::Kind::SummaryTile(tile) => {
                    assert!(evidence_cut);
                    assert!(tile.unavailable.is_none(), "{:?}", tile.unavailable);
                    if tile.card_id.starts_with("group-count.") {
                        grouped_counts.push(tile.value.clone().unwrap());
                    } else {
                        summaries.insert(tile.metadata.as_ref().unwrap().identifier.clone(), tile.value.clone().unwrap());
                    }
                }
                dokime_protocol::host::document_frame_frame::Kind::TableRow(_) => {
                    assert!(evidence_cut);
                    table_rows += 1;
                }
                dokime_protocol::host::document_frame_frame::Kind::CardComplete(id) => {
                    assert!(evidence_cut);
                    assert!(bindings.contains(id));
                    assert!(ready.contains(id));
                    assert!(finished.insert(id.clone()));
                }
                dokime_protocol::host::document_frame_frame::Kind::Completion(counts) => {
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
        assert_eq!(summaries["percentile"].kind, Some(model::value::Kind::Number(2500.5)));
        assert_eq!(summaries["record-mean"].kind, Some(model::value::Kind::Number(3.0)));
        assert_eq!(summaries["record-std"].kind, Some(model::value::Kind::Number(1.0)));
        assert_eq!(summaries["record-rms"].kind, Some(model::value::Kind::Number(10.0f64.sqrt())));
        assert_eq!(grouped_counts.len(), 31);
        assert_eq!(
            grouped_counts
                .iter()
                .map(|value| match value.kind {
                    Some(model::value::Kind::Integer(value)) => value,
                    _ => panic!("Expected Count"),
                })
                .sum::<i64>(),
            10003
        );
        let plot = binding.unwrap();
        assert_eq!(plot.max_points, Some(32));
        assert!(plot.at.is_some());
        let reply = dokime_protocol::host::plots::snapshots_client::SnapshotsClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::host::plots::SnapshotsListRequest {
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
            .get(dokime_protocol::host::DocumentsGetRequest {
                scope: Some(scope.clone()),
                document_id: "measurements".into(),
                at: first.resolved_at.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(replay, first);
        let mut structure = dokime_protocol::host::DocumentsGetRequest {
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
                    dokime_protocol::host::document_frame_frame::Kind::EvidenceCut(_) => cut_seen = true,
                    dokime_protocol::host::document_frame_frame::Kind::CardComplete(_)
                    | dokime_protocol::host::document_frame_frame::Kind::SummaryTile(_)
                    | dokime_protocol::host::document_frame_frame::Kind::TableRow(_) => panic!("Structure-Only Capture Evaluated a Card"),
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
        let subject = dokime_protocol::host::ReportSubject {
            kind: Some(dokime_protocol::host::report_subject::Kind::Execution("documents-a".into())),
        };
        let prepared = reports
            .create(dokime_protocol::host::ReportsCreateRequest {
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
        let mut publish = dokime_protocol::host::ReportsPublishRequest {
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
            .create(dokime_protocol::host::ReportsCreateRequest {
                subject: Some(dokime_protocol::host::ReportSubject {
                    kind: Some(dokime_protocol::host::report_subject::Kind::Cycle("documents".into())),
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
            origin: Some(dokime_protocol::model::Origin {
                kind: Some(model::origin::Kind::Import(dokime_protocol::model::Import::default())),
            }),
            ..Default::default()
        })?;
        let mut cycle_request = dokime_protocol::host::DocumentsGetRequest {
            scope: Some(dokime_protocol::host::DocumentScope {
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
                if let Some(dokime_protocol::host::document_frame_frame::Kind::EvidenceCut(cut)) = frame.frame.and_then(|frame| frame.kind)
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
        Ok::<_, Box<dyn Error>>(())
    })
    .await?
}
