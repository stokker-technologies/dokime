/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Renders Fixed Documents

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    error::Error,
    str,
};

// Third Party Imports
use harmos_protocol::model::{Declarations, StreamDeclaration};
use prost_types::FileDescriptorSet;
use sha2::{Digest, Sha256};
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::Aggregate;
use dokime::AggregateExpression;
use dokime::Card;
use dokime::Client;
use dokime::ContourRender;
use dokime::DataExpression;
use dokime::Document;
use dokime::Downsample;
use dokime::FieldReference;
use dokime::HarmosSection;
use dokime::Import;
use dokime::Metadata;
use dokime::Origin;
use dokime::Plot;
use dokime::PlotKind;
use dokime::RecordContract;
use dokime::RecordContracts;
use dokime::Section;
use dokime::SectionRepeat;
use dokime::Suite;
use dokime::Suites;
use dokime::Summary;
use dokime::SummaryTile;
use dokime::Table;
use dokime::TableRow;
use dokime::Termination;
use dokime::TestCaseTemplate;
use dokime::Value;
use dokime::client::DocumentScope;
use dokime::client::DocumentsGetOptions;
use dokime::client::ExecutionOutcome;
use dokime::client::FilesReadOptions;
use dokime::client::PlotsPlotDomain;
use dokime::client::PlotsPlotSettingsValue;
use dokime::client::PlotsSettingsSetOptions;
use dokime::client::ReportSubject;
use dokime::client::ReportsCreateOptions;
use dokime::client::ReportsPublishOptions;
use dokime::encoding::Message;
use dokime_protocol::FILE_DESCRIPTOR_SET;
use dokime_protocol::builders;
use dokime_protocol::host::DocumentsGetRequest;
use dokime_protocol::host::ReportsPublishRequest;
use dokime_protocol::host::document_binding_result;
use dokime_protocol::host::document_frame_frame;
use dokime_protocol::host::document_layout_node_node;
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::host::report_subject;
use dokime_protocol::model::card::Body;
use dokime_protocol::model::data_expression;
use dokime_protocol::model::origin;
use dokime_protocol::model::value;
use dokime_storage::Evidence;
use dokime_storage::Execution;
use dokime_storage::Sample;
use dokime_storage::Store;

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

#[test]
fn client_renders_fixed_documents() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Prepare a Frozen Catalog with an Authored Document
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let store = Store::open(root.path())?;
            let mut plot = builders::Plot::time_series(FieldReference::new("reading").field("min")).message;
            plot.metadata = Some(Metadata {
                identifier: "trend".into(),
                label: "Trend".into(),
                ..Default::default()
            });
            let aggregate = |input: DataExpression, kind: Aggregate, percentile: f64| DataExpression {
                kind: Some(data_expression::Kind::Aggregate(Box::new(AggregateExpression {
                    input: Some(Box::new(input)),
                    kind: kind as i32,
                    percentile,
                }))),
                ..Default::default()
            };
            let record = DataExpression {
                kind: Some(data_expression::Kind::Record(FieldReference {
                    identifier: "measurement".into(),
                    field: "min".into(),
                })),
                ..Default::default()
            };
            let summary = Summary {
                tiles: vec![
                    SummaryTile {
                        metadata: Some(Metadata {
                            identifier: "percentile".into(),
                            ..Default::default()
                        }),
                        value: Some(aggregate(FieldReference::new("reading").field("max"), Aggregate::Percentile, 25.0)),
                        ..Default::default()
                    },
                    SummaryTile {
                        metadata: Some(Metadata {
                            identifier: "record-mean".into(),
                            ..Default::default()
                        }),
                        value: Some(aggregate(record.clone(), Aggregate::Mean, 0.0)),
                        ..Default::default()
                    },
                    SummaryTile {
                        metadata: Some(Metadata {
                            identifier: "record-std".into(),
                            ..Default::default()
                        }),
                        value: Some(aggregate(record.clone(), Aggregate::Stddev, 0.0)),
                        ..Default::default()
                    },
                    SummaryTile {
                        metadata: Some(Metadata {
                            identifier: "record-rms".into(),
                            ..Default::default()
                        }),
                        value: Some(aggregate(record, Aggregate::Rms, 0.0)),
                        ..Default::default()
                    },
                ],
            };
            let mut document = Document::new("Measurements").id("measurements").section(
                Section::new("Results")
                    .card(Card {
                        metadata: Some(Metadata {
                            identifier: "summary".into(),
                            ..Default::default()
                        }),
                        body: Some(Body::Summary(summary)),
                        ..Default::default()
                    })
                    .card(Card {
                        metadata: Some(Metadata {
                            identifier: "trend-card".into(),
                            ..Default::default()
                        }),
                        body: Some(Body::Plot(plot)),
                        ..Default::default()
                    })
                    .card(Card {
                        metadata: Some(Metadata {
                            identifier: "table".into(),
                            ..Default::default()
                        }),
                        layout: Some(dokime_protocol::model::CardLayout { columns: 6, rows: 2, order: 0 }),
                        body: Some(Body::Table(Table {
                            rows: vec![
                                TableRow {
                                    cells: HashMap::from([("value".into(), Value { kind: Some(value::Kind::Number(23.0)) })])
                                };
                                7
                            ],
                        })),
                        ..Default::default()
                    }),
            );
            document.sections[0].cards.push(Card {
                metadata: Some(Metadata {
                    identifier: "contour-card".into(),
                    ..Default::default()
                }),
                body: Some(Body::Plot(Plot {
                    metadata: Some(Metadata {
                        identifier: "contour".into(),
                        ..Default::default()
                    }),
                    kind: PlotKind::Contour as i32,
                    x: Some(FieldReference::new("reading").field("min")),
                    y: vec![FieldReference::new("reading").field("max")],
                    z: Some(FieldReference::new("reading").field("min")),
                    render: ContourRender::Bands as i32,
                    display_points: Some(32),
                    ..Default::default()
                })),
                ..Default::default()
            });
            document.sections.push(Section {
                metadata: Some(Metadata {
                    identifier: "groups".into(),
                    ..Default::default()
                }),
                repeat: Some(SectionRepeat {
                    value: Some(FieldReference::new("reading").field("min")),
                    label_template: "Group {value}".into(),
                    order_by: None,
                }),
                cards: vec![Card {
                    metadata: Some(Metadata {
                        identifier: "group-count".into(),
                        ..Default::default()
                    }),
                    body: Some(Body::Summary(Summary {
                        tiles: vec![SummaryTile {
                            value: Some(aggregate(FieldReference::new("reading").field("min"), Aggregate::Count, 0.0)),
                            ..Default::default()
                        }],
                    })),
                    ..Default::default()
                }],
                ..Default::default()
            });
            for index in 0..1_001 {
                document.sections.push(Section {
                    metadata: Some(Metadata {
                        identifier: format!("appendix-{index}"),
                        ..Default::default()
                    }),
                    ..Default::default()
                });
            }
            document.metadata.as_mut().unwrap().attributes = HashMap::from([
                ("first".into(), Value { kind: Some(value::Kind::Integer(1)) }),
                ("second".into(), Value { kind: Some(value::Kind::Integer(2)) }),
            ]);
            let suites = Suites {
                suites: vec![Suite {
                    templates: vec![TestCaseTemplate {
                        documents: vec![document.clone()],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let mut catalog = Declarations {
                types: Some(FileDescriptorSet::decode(FILE_DESCRIPTOR_SET)?),
                streams: vec![StreamDeclaration {
                    id: "reading".into(),
                    row: "dokime.host.v1.plots.PlotDomain".into(),
                    ..Default::default()
                }],
                sections: vec![HarmosSection {
                    r#type: "dokime.model.v1.Suites".into(),
                    payload: suites.encode_to_vec(),
                }],
                ..Default::default()
            };
            catalog.sections.push(HarmosSection {
                r#type: "dokime.model.v1.RecordContracts".into(),
                payload: RecordContracts {
                    records: vec![RecordContract {
                        metadata: Some(Metadata {
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
                    origin: Some(Origin {
                        kind: Some(origin::Kind::Import(Import::default())),
                    }),
                    ..Default::default()
                })?;
                store.open_stream(id, "reading")?;
                let samples = (0..10_003)
                    .map(|index| Sample {
                        stream: "reading".into(),
                        source_ns: index * 1_000_000_000,
                        row: PlotsPlotDomain {
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
                        Evidence {
                            kind: "record:measurement".into(),
                            source_ns: 0,
                            payload: RecordsAppendRequest {
                                execution: id.into(),
                                kind: "measurement".into(),
                                payload: PlotsPlotDomain { min: value, max: value + 7.0 }.encode_to_vec(),
                                ..Default::default()
                            }
                            .encode_to_vec(),
                            ..Default::default()
                        },
                    )?;
                }
                store.close_streams(id)?;
                store.close(id, Termination::Completed, ExecutionOutcome::default())?;
                store.seal(id)?;
            }
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let documents = &client.documents;
            let preferences = &client.plots.settings;
            let reports = &client.reports;
            let files = &client.files;
            let scope = DocumentScope {
                cycle_id: "documents".into(),
                execution_id: "documents-a".into(),
            };
            let current = preferences.get(scope.clone(), "measurements", "trend").await?;
            preferences
                .set_with(
                    scope.clone(),
                    "measurements",
                    "trend",
                    PlotsPlotSettingsValue {
                        display_points: Some(32),
                        downsample: Downsample::MinMax as i32,
                    },
                    current.revision,
                    PlotsSettingsSetOptions {
                        request_id: Some("plot-settings".into()),
                        ..Default::default()
                    },
                )
                .await?;

            // Read Bounded Pages at One Cut and Validate the Complete Frame Protocol.
            let mut request = DocumentsGetRequest {
                scope: Some(scope.clone()),
                document_id: "measurements".into(),
                max_frames: 4,
                max_bytes: 1_000_000,
                ..Default::default()
            };
            let first = documents
                .get_with(
                    request.scope.clone().unwrap_or_default(),
                    request.document_id.clone(),
                    DocumentsGetOptions {
                        structure_only: Some(request.structure_only),
                        card_ids: Some(request.card_ids.clone()),
                        page_token: Some(request.page_token.clone()),
                        max_bytes: Some(request.max_bytes),
                        max_frames: Some(request.max_frames),
                        at: request.at.clone(),
                        ..Default::default()
                    },
                )
                .await?;
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
                page = documents
                    .get_with(
                        request.scope.clone().unwrap_or_default(),
                        request.document_id.clone(),
                        DocumentsGetOptions {
                            structure_only: Some(request.structure_only),
                            card_ids: Some(request.card_ids.clone()),
                            page_token: Some(request.page_token.clone()),
                            max_bytes: Some(request.max_bytes),
                            max_frames: Some(request.max_frames),
                            at: request.at.clone(),
                            ..Default::default()
                        },
                    )
                    .await?;
            }
            assert!(pages > 1);
            let mut structures = false;
            let mut evidence_cut = false;
            let mut completed = false;
            let mut binding = None;
            let mut cards = BTreeSet::new();
            let mut bindings = BTreeSet::new();
            let mut finished = BTreeSet::new();
            let mut sections = BTreeSet::new();
            let mut section_identities = BTreeSet::new();
            let mut repeats = BTreeSet::new();
            let mut layout_count = 0;
            let mut spans_checked = 0;
            let mut table_rows = 0;
            let mut summaries = BTreeMap::new();
            let mut grouped_counts = Vec::new();
            let mut ready = BTreeSet::new();
            for (index, frame) in frames.iter().enumerate() {
                match frame.frame.as_ref().unwrap().kind.as_ref().unwrap() {
                    document_frame_frame::Kind::Card(card) => {
                        assert!(!structures);
                        assert!(cards.insert(card.metadata.as_ref().unwrap().identifier.clone()));
                    }
                    document_frame_frame::Kind::GroupDefinition(_) | document_frame_frame::Kind::TileDefinition(_) => {
                        assert!(!structures)
                    }
                    document_frame_frame::Kind::Binding(value) => {
                        assert_eq!(value.metadata.as_ref().unwrap().identifier, value.source_id);
                        if !structures {
                            assert!(matches!(
                                value.result.as_ref().and_then(|result| result.kind.as_ref()),
                                Some(document_binding_result::Kind::Pending(true))
                            ));
                            bindings.insert(value.source_id.clone());
                        } else {
                            assert!(evidence_cut);
                            assert!(bindings.contains(&value.source_id));
                            assert!(ready.insert(value.source_id.clone()));
                        }
                        if let Some(document_binding_result::Kind::Plot(plot)) =
                            value.result.as_ref().and_then(|result| result.kind.as_ref())
                            && value.source_id == "trend-card"
                        {
                            binding = plot.request.clone();
                        }
                    }
                    document_frame_frame::Kind::SectionIdentity(section) => {
                        assert!(!structures);
                        assert!(!section.node_id.is_empty());
                        assert_eq!(section.metadata.as_ref().unwrap().identifier, section.node_id);
                        assert!(section_identities.insert(section.node_id.clone()));
                    }
                    document_frame_frame::Kind::LayoutNode(node) => {
                        assert!(!structures);
                        layout_count += 1;
                        match node.node.as_ref().and_then(|node| node.kind.as_ref()) {
                            Some(document_layout_node_node::Kind::Section(section)) => assert!(sections.insert(section.node_id.clone())),
                            // An Undeclared Layout Reports No Span; a Declared Span Is Kept.
                            Some(document_layout_node_node::Kind::Placement(placement)) if placement.card_id.ends_with("summary") => {
                                assert_eq!(
                                    (placement.column_span.as_ref(), placement.row_span),
                                    (None, 0),
                                    "Leave Undeclared Card Spans Absent"
                                );
                                spans_checked += 1;
                            }
                            Some(document_layout_node_node::Kind::Placement(placement)) if placement.card_id.ends_with("table") => {
                                assert_eq!(
                                    (placement.column_span.as_ref().map(|span| span.base), placement.row_span),
                                    (Some(6), 2),
                                    "Keep Declared Card Spans"
                                );
                                spans_checked += 1;
                            }
                            _ => {}
                        }
                    }
                    document_frame_frame::Kind::RepeatSelection(value) => {
                        assert!(!structures);
                        repeats.insert(value.node_id.clone());
                    }
                    document_frame_frame::Kind::StructureComplete(counts) => {
                        assert!(!structures);
                        structures = true;
                        assert_eq!(counts.frame_count, index as u64 + 1);
                        assert_eq!(counts.card_count, cards.len() as u64);
                        assert_eq!(counts.binding_count, bindings.len() as u64);
                        assert_eq!(counts.layout_node_count, layout_count);
                        assert_eq!(spans_checked, 2, "Report Both Card Placements");
                        assert_eq!(counts.section_count, sections.len() as u64);
                        assert_eq!(section_identities, sections);
                        assert_eq!(counts.repeat_count, repeats.len() as u64);
                    }
                    document_frame_frame::Kind::EvidenceCut(_) => {
                        assert!(structures);
                        assert!(!evidence_cut);
                        evidence_cut = true;
                    }
                    document_frame_frame::Kind::SummaryGroup(group) => {
                        assert!(evidence_cut);
                        assert!(ready.contains(&group.card_id));
                    }
                    document_frame_frame::Kind::SummaryTile(tile) => {
                        assert!(evidence_cut);
                        assert!(tile.unavailable.is_none(), "{:?}", tile.unavailable);
                        if tile.card_id.starts_with("group-count.") {
                            grouped_counts.push(tile.value.clone().unwrap());
                        } else {
                            summaries.insert(tile.metadata.as_ref().unwrap().identifier.clone(), tile.value.clone().unwrap());
                        }
                    }
                    document_frame_frame::Kind::TableRow(_) => {
                        assert!(evidence_cut);
                        table_rows += 1;
                    }
                    document_frame_frame::Kind::CardComplete(id) => {
                        assert!(evidence_cut);
                        assert!(bindings.contains(id));
                        assert!(ready.contains(id));
                        assert!(finished.insert(id.clone()));
                    }
                    document_frame_frame::Kind::Completion(counts) => {
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
            assert_eq!(summaries["percentile"].kind, Some(value::Kind::Number(2500.5)));
            assert_eq!(summaries["record-mean"].kind, Some(value::Kind::Number(3.0)));
            assert_eq!(summaries["record-std"].kind, Some(value::Kind::Number(1.0)));
            assert_eq!(summaries["record-rms"].kind, Some(value::Kind::Number(10.0f64.sqrt())));
            assert_eq!(grouped_counts.len(), 31);
            assert_eq!(
                grouped_counts
                    .iter()
                    .map(|value| match value.kind {
                        Some(value::Kind::Integer(value)) => value,
                        _ => panic!("Expected Count"),
                    })
                    .sum::<i64>(),
                10003
            );
            let plot = binding.unwrap();
            assert_eq!(plot.max_points, Some(32));
            assert!(plot.at.is_some());
            let reply =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .plots
                    .snapshots
                    .clone()
                    .list(vec![plot])
                    .await?;
            assert_eq!(reply.envelopes[0].diagnostics.as_ref().unwrap().examined_rows, 10003);
            assert!(reply.envelopes[0].diagnostics.as_ref().unwrap().point_count <= 32);
            assert!(store.reports("documents-a")?.is_empty());

            // Replaying a Cut Preserves Structure and Its Page Boundary.
            let replay = documents
                .get_with(
                    scope.clone(),
                    "measurements",
                    DocumentsGetOptions {
                        structure_only: Some(Default::default()),
                        card_ids: Some(Default::default()),
                        page_token: Some(String::new()),
                        max_bytes: Some(1_000_000),
                        max_frames: Some(4),
                        at: first.resolved_at.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(replay, first);
            let mut structure = DocumentsGetRequest {
                scope: Some(scope),
                document_id: "measurements".into(),
                structure_only: true,
                max_frames: 4,
                max_bytes: 1_000_000,
                ..Default::default()
            };
            let mut cut_seen = false;
            loop {
                let page = documents
                    .get_with(
                        structure.scope.clone().unwrap_or_default(),
                        structure.document_id.clone(),
                        DocumentsGetOptions {
                            structure_only: Some(structure.structure_only),
                            card_ids: Some(structure.card_ids.clone()),
                            page_token: Some(structure.page_token.clone()),
                            max_bytes: Some(structure.max_bytes),
                            max_frames: Some(structure.max_frames),
                            at: structure.at.clone(),
                            ..Default::default()
                        },
                    )
                    .await?;
                for frame in page.frames {
                    match frame.frame.unwrap().kind.unwrap() {
                        document_frame_frame::Kind::EvidenceCut(_) => cut_seen = true,
                        document_frame_frame::Kind::CardComplete(_)
                        | document_frame_frame::Kind::SummaryTile(_)
                        | document_frame_frame::Kind::TableRow(_) => {
                            panic!("Structure-Only Capture Evaluated a Card")
                        }
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
            let subject = ReportSubject {
                kind: Some(report_subject::Kind::Execution("documents-a".into())),
            };
            let prepared = reports
                .create_with(
                    subject.clone(),
                    "measurements",
                    ReportsCreateOptions {
                        request_id: Some("preview".into()),
                        ..Default::default()
                    },
                )
                .await?;
            let reference = prepared.file.unwrap();
            let name = reference.metadata.as_ref().unwrap().id.clone();
            let mut html = Vec::new();
            let mut at = None;
            while (html.len() as u64) < reference.length {
                let page = files.read_with(reference.clone(), html.len() as u64, 0, FilesReadOptions { at, ..Default::default() }).await?;
                assert!(!page.data.is_empty());
                at = page.resolved_at;
                html.extend(page.data);
            }
            assert_eq!(Sha256::digest(&html).as_slice(), reference.sha256);
            assert!(str::from_utf8(&html)?.contains("<svg"));
            assert!(store.reports("documents-a")?.is_empty());
            let mut publish = ReportsPublishRequest {
                name,
                subject: Some(subject),
                expected_revision: vec![0],
                request_id: "publish".into(),
            };
            assert!(
                reports
                    .publish_with(
                        publish.name.clone(),
                        publish.subject.clone().unwrap_or_default(),
                        publish.expected_revision.clone(),
                        ReportsPublishOptions {
                            request_id: Some(publish.request_id.clone()),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            assert!(store.reports("documents-a")?.is_empty());
            publish.expected_revision = reference.sha256;
            reports
                .publish_with(
                    publish.name.clone(),
                    publish.subject.clone().unwrap_or_default(),
                    publish.expected_revision.clone(),
                    ReportsPublishOptions {
                        request_id: Some(publish.request_id.clone()),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(store.reports("documents-a")?.len(), 1);
            let cycle = reports
                .create_with(
                    ReportSubject {
                        kind: Some(report_subject::Kind::Cycle("documents".into())),
                    },
                    String::new(),
                    ReportsCreateOptions {
                        request_id: Some("cycle-preview".into()),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(cycle.file.unwrap().length > 0);
            assert!(store.reports("documents-b")?.is_empty());
            store.create(&Execution {
                identifier: "documents-open".into(),
                cycle: "documents".into(),
                catalog: catalog.encode_to_vec(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::Import(Import::default())),
                }),
                ..Default::default()
            })?;
            let mut cycle_request = DocumentsGetRequest {
                scope: Some(DocumentScope {
                    cycle_id: "documents".into(),
                    execution_id: String::new(),
                }),
                document_id: "default".into(),
                max_frames: 4,
                max_bytes: 1_000_000,
                ..Default::default()
            };
            let mut cacheable = None;
            loop {
                let page = documents
                    .get_with(
                        cycle_request.scope.clone().unwrap_or_default(),
                        cycle_request.document_id.clone(),
                        DocumentsGetOptions {
                            structure_only: Some(cycle_request.structure_only),
                            card_ids: Some(cycle_request.card_ids.clone()),
                            page_token: Some(cycle_request.page_token.clone()),
                            max_bytes: Some(cycle_request.max_bytes),
                            max_frames: Some(cycle_request.max_frames),
                            at: cycle_request.at.clone(),
                            ..Default::default()
                        },
                    )
                    .await?;
                for frame in page.frames {
                    if let Some(document_frame_frame::Kind::EvidenceCut(cut)) = frame.frame.and_then(|frame| frame.kind) {
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
    }))
}
