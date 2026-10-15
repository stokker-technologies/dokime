/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use super::*;
use crate::support::reports::{parse, payload_text, render};
use dokime::core::{TraceabilityId, Value};
use dokime::definition::TestCase;
use dokime_app::document::capture::{Capture, SectionBinding};
use dokime_app::document::{
    ArtifactListDescriptor, DocumentCardBodyPayload, DocumentCardLayout, DocumentCardPayload,
    DocumentCardPlacementPayload, DocumentLayoutPayload, DocumentNodePayload, DocumentPayload,
    DocumentSectionNodePayload, DocumentSelection, DocumentWarning, PlotAxisKind, PlotDescriptor,
    PlotInput, PlotStyle, PlotType, TelemetrySummaryFormat, TelemetrySummaryGroupPayload,
    TelemetrySummaryPresentation, TelemetrySummaryStat, TelemetrySummaryTilePayload,
};
use dokime_app::execution::{CycleStatus, CycleView, TestExecution};
use dokime_app::records::TestRecordSet;
use dokime_app::report::{CycleReportInput, ReportCaseDocument};
use dokime_app::traceability::{TraceabilityEntry, TraceabilitySnapshot};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn fixture_input(cycle_label: &str) -> CycleReportInput {
    let test_cycle_id = dokime::core::TestCycleId::new();
    let case_id = dokime::core::CaseId::new();
    let template_id = TemplateId::new();
    let cycle_metadata = metadata("cycle.fixture", cycle_label);

    let case = TestCase {
        suite_id: "fixture.registration".to_owned(),
        test_case_id: case_id,
        test_case_template_id: template_id,
        suite: None,
        metadata: metadata("case.fixture", "Fixture Case"),
        parameters: BTreeMap::from([("fixture.result".to_owned(), Value::from("passed"))]),
        generation_index: 0,
        label_number: 0,
    };
    let test_execution_id = dokime::core::TestExecutionId::new();
    let mut execution = fixture_test_execution(
        test_execution_id.clone(),
        case.test_case_id.clone(),
        RecordScope::default()
            .with_cycle(test_cycle_id.clone())
            .with_test_case(case.test_case_id.clone())
            .with_test_execution(test_execution_id),
        ExecutionStatus::Skipped,
    );
    execution.started_at = None;
    execution.snapshot = dokime_app::execution::ExecutionSnapshot::materialize(
        &case,
        None,
        Default::default(),
        None,
        Default::default(),
    )
    .unwrap();

    CycleReportInput {
        session_id: dokime::core::SessionId::new(),
        cycle: CycleView {
            saved: None,
            test_cycle_id: test_cycle_id.clone(),
            session_id: None,
            metadata: cycle_metadata,
            lifecycle: CycleStatus::Ready,
            created_at: Timestamp::now_utc(),
            started_at: None,
            updated_at: Timestamp::now_utc(),
            test_executions: vec![execution],
            records: TestRecordSet::default(),
            continue_on_failure: false,
            result: None,
        },
        documents: BTreeMap::new(),
        plots: BTreeMap::new(),
        case_documents: Vec::new(),
        records: Vec::new(),
        telemetry: Vec::new(),
        traceability: Vec::new(),
        artifact_links: Vec::new(),
        appearance: None,
        generated_at: Timestamp::now_utc(),
        omissions: Vec::new(),
    }
}

fn insert_report_document(
    input: &mut CycleReportInput,
    document_id_value: &str,
    document: DocumentPayload,
) {
    input
        .documents
        .insert(document_id(document_id_value), report_document(document));
}

fn document_id(value: &str) -> dokime::core::DocumentId {
    dokime::core::DocumentId::from_identifier(value).unwrap()
}

fn report_document(document: DocumentPayload) -> Capture {
    use dokime_app::document::capture::{CardBinding, CardBindingResult};
    use dokime_app::document::{
        DocumentCardBody, DocumentCardDefinition, DocumentDefinition, DocumentLayout, DocumentNode,
        DocumentSectionNode,
    };
    let mut definitions = Vec::new();
    let mut bindings = Vec::new();
    for card in document.cards {
        let (body, result) = match card.body {
            DocumentCardBodyPayload::TelemetrySummary { groups } => {
                let definitions = groups
                    .iter()
                    .map(
                        |group| dokime_app::document::TelemetrySummaryGroupDefinition {
                            metadata: group.metadata.clone(),
                            tiles: group
                                .tiles
                                .iter()
                                .map(
                                    |tile| dokime_app::document::TelemetrySummaryTileDefinition {
                                        metadata: tile.metadata.clone(),
                                        value: dokime::expressions::telemetry(
                                            "progress", "distance",
                                        ),
                                        filters: Vec::new(),
                                        stat: tile.stat.clone(),
                                        format: tile.format.clone(),
                                        presentation: tile.presentation.clone(),
                                        unit: tile.unit.clone(),
                                        conditions: Vec::new(),
                                    },
                                )
                                .collect(),
                        },
                    )
                    .collect();
                (
                    DocumentCardBody::TelemetrySummary {
                        summary: dokime_app::document::TelemetrySummaryDefinition {
                            groups: definitions,
                        },
                    },
                    CardBindingResult::Summary { groups },
                )
            }
            DocumentCardBodyPayload::Plot { descriptor } => {
                let mut value = serde_json::to_value(&descriptor.input).unwrap();
                value["metadata"] = serde_json::to_value(&descriptor.metadata).unwrap();
                value["y"] = value["series_fields"].clone();
                value["x"] = if descriptor.plot_type == PlotType::TimeSeries {
                    serde_json::Value::Null
                } else {
                    value["x_field"].clone()
                };
                let plot = serde_json::from_value(value).unwrap();
                (
                    DocumentCardBody::Plot {
                        plot: Box::new(plot),
                    },
                    CardBindingResult::Plot { descriptor },
                )
            }
            body => {
                let body = match body {
                    DocumentCardBodyPayload::Text { markdown } => {
                        DocumentCardBody::Text { markdown }
                    }
                    DocumentCardBodyPayload::Table { rows } => DocumentCardBody::Table { rows },
                    DocumentCardBodyPayload::Steps => DocumentCardBody::Steps,
                    DocumentCardBodyPayload::Criteria => DocumentCardBody::Criteria,
                    DocumentCardBodyPayload::Traceability => DocumentCardBody::Traceability,
                    DocumentCardBodyPayload::Notes => DocumentCardBody::Notes,
                    DocumentCardBodyPayload::Records => DocumentCardBody::Records,
                    DocumentCardBodyPayload::Artifacts { .. } => DocumentCardBody::Artifacts,
                    DocumentCardBodyPayload::Media { .. } => DocumentCardBody::Media,
                    DocumentCardBodyPayload::Health => DocumentCardBody::Health,
                    DocumentCardBodyPayload::TelemetryContext => DocumentCardBody::TelemetryContext,
                    _ => unreachable!(),
                };
                (body, CardBindingResult::Static)
            }
        };
        definitions.push(DocumentCardDefinition {
            metadata: card.metadata.clone(),
            filters: Vec::new(),
            body,
        });
        bindings.push(CardBinding {
            source_id: card.metadata.identifier.clone(),
            metadata: card.metadata,
            result,
        });
    }
    fn layout(
        nodes: Vec<DocumentNodePayload>,
        sections: &mut BTreeMap<Identifier, SectionBinding>,
    ) -> Vec<DocumentNode> {
        nodes
            .into_iter()
            .map(|node| match node {
                DocumentNodePayload::Card(card) => {
                    DocumentNode::Card(dokime_app::document::DocumentCardPlacement {
                        card_id: card.card_id,
                        layout: card.layout,
                    })
                }
                DocumentNodePayload::Section(section) => {
                    sections.insert(
                        section.node_id.clone(),
                        SectionBinding {
                            metadata: section.metadata.clone(),
                            unavailable: None,
                        },
                    );
                    DocumentNode::Section(Box::new(DocumentSectionNode {
                        node_id: section.node_id,
                        metadata: section.metadata,
                        repeat: None,
                        children: layout(section.children, sections),
                    }))
                }
            })
            .collect()
    }
    let mut sections = BTreeMap::new();
    let layout = DocumentLayout {
        nodes: layout(document.layout.nodes, &mut sections),
    };
    Capture {
        definition: DocumentDefinition {
            document_id: document.document_id,
            metadata: document.metadata,
            cards: definitions,
            layout,
            sections: Vec::new(),
        },
        selection: document.selection,
        cards: bindings,
        repeats: BTreeMap::new(),
        sections,
        warnings: document.warnings,
    }
}

fn materialization_shape_is_raw(value: &serde_json::Value) -> bool {
    value.get("selected").is_none()
        && value.get("layout").is_none()
        && value["definition"].get("layout").is_some()
}

fn fixture_plot_payload(
    stream: &str,
    series_key: &str,
    y_value: f64,
    raw_points: usize,
) -> dokime_app::report::value::Value {
    dokime_app::report::value::Value::try_from(serde_json::json!({
        "stream": stream,
        "x_key": "timestamp",
        "series_by": serde_json::Value::Null,
        "context": {},
        "series": [{
            "key": series_key,
            "label": series_key,
            "unit": serde_json::Value::Null,
            "points": [{"x": "2026-01-01T00:00:00Z", "y": y_value}],
            "raw_points": raw_points,
            "downsampled": raw_points > 1,
        }],
        "cartesian": serde_json::Value::Null,
        "markers": [],
        "max_points": 2048,
        "window_s": serde_json::Value::Null,
        "generated_at": "2026-01-01T00:00:00Z",
    }))
    .unwrap()
}

fn fixture_test_execution(
    test_execution_id: dokime::core::TestExecutionId,
    test_case_id: dokime::core::CaseId,
    scope: RecordScope,
    status: ExecutionStatus,
) -> TestExecution {
    let case = dokime::definition::TestCase {
        suite_id: "fixture.registration".to_owned(),
        test_case_id: test_case_id.clone(),
        test_case_template_id: dokime::core::TemplateId::from_seed("fixture.template"),
        metadata: dokime::core::Metadata::new(
            dokime::core::Identifier::new("fixture.case").unwrap(),
            "Fixture Case",
            None,
        ),
        suite: None,
        parameters: Default::default(),
        generation_index: 0,
        label_number: 0,
    };

    TestExecution {
        saved: None,
        mode: Default::default(),
        active_step_attempt: None,
        capture_phase: Default::default(),
        step_attempts: Vec::new(),
        test_cycle_id: scope
            .test_cycle_id
            .clone()
            .expect("fixture execution has an owning cycle"),
        test_execution_id,
        test_case_id,
        suite: None,
        status,
        scope,
        snapshot: dokime::execution::ExecutionSnapshot::materialize(
            &case,
            None,
            Default::default(),
            None,
            Default::default(),
        )
        .unwrap(),
        invalidation: None,
        output_parameters: BTreeMap::new(),
        documents: BTreeMap::new(),
        records: TestRecordSet::default(),
        started_at: Some(Timestamp::new("2026-01-01T00:00:00Z")),
        finished_at: None,
        steps: Vec::new(),
        traceability: Vec::new(),
        result: None,
    }
}

fn fixture_plot_input(plot_id: Identifier, scope: RecordScope, plot_type: PlotType) -> PlotInput {
    PlotInput {
        plot_id,
        scope,
        plot_type,
        x_field: Some("timestamp".to_owned()),
        series_fields: Vec::new(),
        z_stream: None,
        z_field: None,
        stream: None,
        series_by: None,
        x_expression: None,
        y_expressions: Vec::new(),
        z_expression: None,
        series_by_expression: None,
        filters: Vec::new(),
        unit: None,
        style: PlotStyle::Line,
        x_scale: Default::default(),
        y_scale: Default::default(),
        analysis: None,
        fft: Default::default(),
        interval: Default::default(),
        histogram: Default::default(),
        marker_record_types: Vec::new(),
        reference_lines: Vec::new(),
        reference_bands: Vec::new(),
        live_window_s: Some(120.0),
        display_points: None,
        downsample: Default::default(),
        x_format: None,
        y_format: None,
        x_axis: None,
        available_streams: Vec::new(),
        marker_count: 0,
    }
}

fn fixture_plot_document(
    document_identifier: &str,
    document_label: &str,
    section_identifier: &str,
    section_label: &str,
    card_label: &str,
    descriptor: PlotDescriptor,
    scope: RecordScope,
) -> DocumentPayload {
    let cards = vec![document_card(
        &format!("{}.card", descriptor.plot_id),
        card_label,
        DocumentCardBodyPayload::Plot {
            descriptor: Box::new(descriptor),
        },
    )];

    DocumentPayload {
        document_id: dokime::core::DocumentId::new(),
        metadata: metadata(document_identifier, document_label),
        selection: DocumentSelection { scope },
        cards: cards.clone(),
        layout: document_layout(section_identifier, section_label, &cards),
        plot_descriptors: Vec::new(),
        plot_inputs: Vec::new(),
        record_markers: Vec::new(),
        artifact_lists: Vec::new(),
        warnings: Vec::new(),
    }
}

fn fixture_document(test_cycle_id: dokime::core::TestCycleId) -> DocumentPayload {
    let scope = RecordScope::default().with_cycle(test_cycle_id);
    let plot_id = Identifier::new("cycle.telemetry").unwrap();
    let artifact_descriptor = ArtifactListDescriptor {
        scope: scope.clone(),
        artifact_count: 3,
        completed_count: 2,
    };
    let cards = vec![
        document_card(
            "summary.text",
            "Summary",
            DocumentCardBodyPayload::Text {
                markdown: "Cycle summary".to_owned(),
            },
        ),
        document_card(
            "summary.telemetry",
            "Telemetry Summary",
            DocumentCardBodyPayload::TelemetrySummary {
                groups: vec![TelemetrySummaryGroupPayload {
                    metadata: metadata("summary.telemetry", "Telemetry Summary"),
                    tiles: vec![TelemetrySummaryTilePayload {
                        metadata: metadata("distance.max", "Max Distance"),
                        value: Some(Value::from(12.5)),
                        unavailable: None,
                        stat: TelemetrySummaryStat::Latest,
                        format: TelemetrySummaryFormat::Number,
                        presentation: TelemetrySummaryPresentation::Value,
                        unit: Some("m".to_owned()),
                        state: None,
                    }],
                }],
            },
        ),
        document_card(
            "cycle.telemetry.card",
            "Cycle Telemetry",
            DocumentCardBodyPayload::Plot {
                descriptor: Box::new(PlotDescriptor {
                    plot_id: plot_id.clone(),
                    metadata: metadata("cycle.telemetry", "Cycle Telemetry"),
                    plot_type: PlotType::TimeSeries,
                    input: PlotInput {
                        series_fields: vec!["distance".to_owned()],
                        available_streams: vec![Identifier::new("progress").unwrap()],
                        marker_count: 1,
                        // A duration x column: what the frozen report has to
                        // carry for its axis to read as elapsed seconds.
                        x_axis: Some(PlotAxisKind::Elapsed {
                            unit: "s".to_owned(),
                        }),
                        ..fixture_plot_input(plot_id, scope.clone(), PlotType::TimeSeries)
                    },
                }),
            },
        ),
        document_card(
            "artifacts",
            "Artifacts",
            DocumentCardBodyPayload::Artifacts {
                descriptor: Box::new(artifact_descriptor.clone()),
            },
        ),
    ];

    DocumentPayload {
        document_id: dokime::core::DocumentId::new(),
        metadata: metadata("document.fixture", "Fixture Document"),
        selection: DocumentSelection {
            scope: scope.clone(),
        },
        cards: cards.clone(),
        layout: document_layout("result", "Summary", &cards),
        plot_descriptors: Vec::new(),
        plot_inputs: Vec::new(),
        record_markers: Vec::new(),
        artifact_lists: vec![artifact_descriptor],
        warnings: vec![DocumentWarning {
            code: Identifier::new("document.plot.no_telemetry").unwrap(),
            message: "Telemetry is not available yet.".to_owned(),
        }],
    }
}

fn fixture_wifi_document(test_cycle_id: dokime::core::TestCycleId) -> DocumentPayload {
    let scope = RecordScope::default().with_cycle(test_cycle_id);
    let plot_id = Identifier::new("wifi.channel_occupancy").unwrap();

    fixture_plot_document(
        "document.wifi",
        "WiFi Document",
        "wifi",
        "WiFi",
        "WiFi Channel Occupancy",
        PlotDescriptor {
            plot_id: plot_id.clone(),
            metadata: metadata("wifi.channel_occupancy", "WiFi Channel Occupancy"),
            plot_type: PlotType::WifiChannelOccupancy,
            input: PlotInput {
                series_fields: vec!["network_count".to_owned()],
                stream: Some("host.wifi.channel_occupancy".to_owned()),
                series_by: Some("wifi_channel".to_owned()),
                available_streams: vec![Identifier::new("host.wifi.channel_occupancy").unwrap()],
                ..fixture_plot_input(plot_id, scope.clone(), PlotType::WifiChannelOccupancy)
            },
        },
        scope,
    )
}

fn fixture_tracking_case_document(scope: RecordScope) -> DocumentPayload {
    let plot_id = Identifier::new("tracking.distance").unwrap();

    fixture_plot_document(
        "document.tracking_case",
        "Tracking Case Document",
        "position",
        "Position",
        "Tracking Distance",
        PlotDescriptor {
            plot_id: plot_id.clone(),
            metadata: metadata("tracking.distance", "Tracking Distance"),
            plot_type: PlotType::TimeSeries,
            input: PlotInput {
                series_fields: vec!["distance_m".to_owned()],
                stream: Some("plt400.ract.tracking.tracking".to_owned()),
                unit: Some("m".to_owned()),
                available_streams: vec![Identifier::new("plt400.ract.tracking.tracking").unwrap()],
                ..fixture_plot_input(plot_id, scope.clone(), PlotType::TimeSeries)
            },
        },
        scope,
    )
}

fn fixture_evidence_case_document(scope: RecordScope) -> DocumentPayload {
    let records = document_card(
        "events.timeline",
        "Records",
        DocumentCardBodyPayload::Records,
    );
    let media = document_card(
        "traceability.media",
        "Media",
        DocumentCardBodyPayload::Media {
            descriptor: Box::new(ArtifactListDescriptor {
                scope: scope.clone(),
                artifact_count: 1,
                completed_count: 1,
            }),
        },
    );

    DocumentPayload {
        document_id: document_id("default"),
        metadata: metadata("document.evidence", "Evidence"),
        selection: DocumentSelection {
            scope: scope.clone(),
        },
        cards: vec![records.clone(), media.clone()],
        layout: DocumentLayoutPayload {
            nodes: vec![
                document_section_node("execution.events", "Records", &[records]),
                document_section_node("execution.traceability.media", "Media", &[media]),
            ],
        },
        plot_descriptors: Vec::new(),
        plot_inputs: Vec::new(),
        record_markers: Vec::new(),
        artifact_lists: Vec::new(),
        warnings: Vec::new(),
    }
}

fn metadata(identifier: &str, label: &str) -> Metadata {
    Metadata::new(Identifier::new(identifier).unwrap(), label, None)
}

fn document_card(
    identifier: &str,
    label: &str,
    body: DocumentCardBodyPayload,
) -> DocumentCardPayload {
    DocumentCardPayload {
        metadata: metadata(identifier, label),
        body,
    }
}

fn document_layout(
    section_identifier: &str,
    section_label: &str,
    cards: &[DocumentCardPayload],
) -> DocumentLayoutPayload {
    DocumentLayoutPayload {
        nodes: vec![document_section_node(
            section_identifier,
            section_label,
            cards,
        )],
    }
}

fn document_section_node(
    section_identifier: &str,
    section_label: &str,
    cards: &[DocumentCardPayload],
) -> DocumentNodePayload {
    DocumentNodePayload::Section(DocumentSectionNodePayload {
        node_id: Identifier::new(section_identifier).unwrap(),
        metadata: metadata(section_identifier, section_label),
        children: cards
            .iter()
            .map(|card| {
                DocumentNodePayload::Card(DocumentCardPlacementPayload {
                    card_id: card.metadata.identifier.clone(),
                    layout: DocumentCardLayout::default(),
                })
            })
            .collect(),
    })
}

/* === Tests === */

pub(crate) async fn rendered_html_data() {
    let input = fixture_input("<script>alert('x')</script>");
    let html = render(&input).await;

    assert!(html.starts_with("<!doctype html>"));
    assert!(html.contains("<style "));
    assert!(html.contains("id=\"root\""));
    assert!(html.contains("id=\"hub-report-data\""));
    assert!(!html.contains("<script>alert"));
    assert!(payload_text(&html).contains(r#"\u003cscript\u003ealert"#));
    assert!(!html.contains("src=\"http"));
    assert!(!html.contains("href=\"http"));

    let payload = parse(&html);
    assert_eq!(
        payload["test_cycle_plan"]["test_cycle"]["title"],
        "<script>alert('x')</script>"
    );
    assert_eq!(payload["test_executions"].as_array().unwrap().len(), 1);
}

pub(crate) async fn rendered_html_payload() {
    let mut input = fixture_input("Safe Cycle");
    let test_cycle_id = input.cycle.test_cycle_id.clone();
    let document = fixture_document(input.cycle.test_cycle_id.clone());
    insert_report_document(&mut input, "default", document);
    input.plots.insert(
        "cycle.telemetry".to_owned(),
        fixture_plot_payload("progress", "distance", 12.5, 1),
    );

    let html = render(&input).await;

    let payload_text = payload_text(&html);
    let payload = parse(&html);

    assert_eq!(
        payload["documents"]["default"]["definition"]["metadata"]["label"],
        "Fixture Document"
    );
    assert!(payload_text.contains("Max Distance"));
    assert!(payload_text.contains("Cycle Telemetry"));
    assert!(materialization_shape_is_raw(
        &payload["documents"]["default"]
    ));
    assert!(payload_text.contains("Telemetry is not available yet."));
    assert_eq!(
        payload["plots"]["cycle.telemetry"]["series"][0]["points"][0]["y"],
        12.5
    );
    assert_eq!(payload["warnings"].as_array().unwrap().len(), 2);

    // One document, resolved once, embedded once — and the selection is a scope
    // and nothing else; a view never narrowed it.
    let materialization = &payload["documents"]["default"];
    assert_eq!(
        materialization
            .as_object()
            .expect("a report document materialization is an object")
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([
            "cards",
            "definition",
            "repeats",
            "sections",
            "selection",
            "warnings"
        ])
    );
    let selected = materialization;
    assert_eq!(
        selected["selection"]
            .as_object()
            .expect("a selection is an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["scope"]
    );
    assert_eq!(
        selected["selection"]["scope"]["test_cycle_id"],
        test_cycle_id.to_string()
    );
    assert!(selected.get("views").is_none());
    assert_eq!(
        selected["definition"]["layout"]["nodes"][0]["data"]["node_id"],
        "result"
    );
    assert_eq!(
        payload["plots"]["cycle.telemetry"]["series"][0]["raw_points"],
        1
    );
}

pub(crate) async fn materialized_report_axis() {
    let mut input = fixture_input("Elapsed Cycle");
    let document = fixture_document(input.cycle.test_cycle_id.clone());
    insert_report_document(&mut input, "default", document);

    let html = render(&input).await;
    let payload = parse(&html);

    let card = payload["documents"]["default"]["cards"]
        .as_array()
        .expect("a materialized document carries its cards")
        .iter()
        .find(|card| card["result"]["type"] == "plot")
        .expect("one plot card");
    assert_eq!(
        card["result"]["data"]["descriptor"]["input"]["x_axis"],
        serde_json::json!({"kind": "elapsed", "unit": "s"})
    );
}

// rendered cycle payload keeps cycle document plots out of case runs.

pub(crate) async fn rendered_cycle_out() {
    let mut input = fixture_input("WiFi Cycle");
    let document = fixture_wifi_document(input.cycle.test_cycle_id.clone());
    insert_report_document(&mut input, "default", document);
    input.plots.insert(
        "wifi.channel_occupancy".to_owned(),
        fixture_plot_payload("host.wifi.channel_occupancy", "6", 4.0, 1),
    );

    let html = render(&input).await;
    let payload = parse(&html);
    let cycle_plot = &payload["plots"]["wifi.channel_occupancy"];
    let case_run = &payload["case_runs"][0];

    assert_eq!(cycle_plot["stream"], "host.wifi.channel_occupancy");
    assert_eq!(cycle_plot["series"][0]["key"], "6");
    assert_eq!(cycle_plot["series"][0]["points"][0]["y"], 4.0);
    assert_eq!(case_run["documents"].as_object().unwrap().len(), 0);
    assert!(case_run["plots"]["wifi.channel_occupancy"].is_null());
}

pub(crate) async fn execution_template_cases() {
    let mut input = fixture_input("Frozen Execution");
    let case = input.cycle.cases().next().unwrap().clone();
    let id = dokime::core::TestExecutionId::new();
    let scope = RecordScope::default()
        .with_cycle(input.cycle.test_cycle_id.clone())
        .with_test_case(case.test_case_id.clone())
        .with_test_execution(id.clone());
    let mut execution =
        fixture_test_execution(id, case.test_case_id, scope, ExecutionStatus::Passed);
    execution.steps.push(dokime_app::execution::StepRecord {
        step_id: dokime::core::StepId::new(),
        metadata: metadata("fixture.step", "Fixture Step"),
        status: dokime_app::execution::StepStatus::Passed,
        started_at: None,
        finished_at: None,
        measurements: Vec::new(),
        criteria: vec![dokime_app::execution::CriterionResult {
            criterion_id: dokime::core::CriterionId::new(),
            metadata: metadata("fixture.criterion", "Fixture Criterion"),
            status: dokime_app::execution::CriterionStatus::Passed,
            observed: Some(Value::from("original measurement")),
            message: Some("original criterion reason".into()),
        }],
        detail: None,
    });
    let template_id = execution.snapshot.case().test_case_template_id.to_string();
    input.cycle.test_executions = vec![execution];

    let html = render(&input).await;
    let payload = parse(&html);
    assert_eq!(payload["test_executions"][0]["template_id"], template_id);
    let steps = payload["test_steps"].as_array().unwrap();
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0]["record_type"], "step");
    assert_eq!(steps[1]["record_type"], "criterion");
    assert_eq!(steps[1]["result"]["reason"], "original criterion reason");
    assert_eq!(
        steps[1]["result"]["details"]["observed"],
        "original measurement"
    );
    assert_eq!(steps[0]["started_at"], steps[1]["started_at"]);
    assert_eq!(steps[0]["scope"]["step_id"], steps[1]["scope"]["step_id"]);
    assert!(steps[0]["scope"]["criterion_id"].is_null());
    assert!(steps[1]["scope"]["criterion_id"].is_string());
    for step in steps {
        assert_eq!(step["template_id"], template_id);
        assert_eq!(step["scope"]["template_id"], template_id);
    }
}

pub(crate) async fn rendered_case_plots() {
    let mut input = fixture_input("Tracking Cycle");
    let case_id = input.cycle.cases().next().unwrap().test_case_id.clone();
    let test_execution_id = dokime::core::TestExecutionId::new();
    let scope = RecordScope::default()
        .with_cycle(input.cycle.test_cycle_id.clone())
        .with_test_case(case_id.clone())
        .with_test_execution(test_execution_id.clone());

    let document = fixture_document(input.cycle.test_cycle_id.clone());
    insert_report_document(&mut input, "default", document);
    input.cycle.test_executions = vec![fixture_test_execution(
        test_execution_id.clone(),
        case_id.clone(),
        scope.clone(),
        ExecutionStatus::Passed,
    )];
    input.case_documents = vec![ReportCaseDocument {
        test_case_id: case_id,
        test_execution_id: Some(test_execution_id.clone()),
        documents: BTreeMap::from([(
            document_id("default"),
            report_document(fixture_tracking_case_document(scope.clone())),
        )]),
        plots: BTreeMap::from([(
            "tracking.distance".to_owned(),
            fixture_plot_payload("plt400.ract.tracking.tracking", "distance_m", 7.5, 1),
        )]),
    }];

    let html = render(&input).await;
    let payload = parse(&html);
    let case_run = &payload["case_runs"][0];
    let case_plot = &case_run["plots"]["tracking.distance"];

    assert_eq!(case_run["queue_item_id"], case_run["case_id"]);
    assert_eq!(case_run["test_execution_id"], test_execution_id.to_string());
    assert_eq!(
        case_run["documents"]["default"]["definition"]["metadata"]["label"],
        "Tracking Case Document"
    );
    assert_eq!(case_plot["stream"], "plt400.ract.tracking.tracking");
    assert_eq!(case_plot["series"][0]["points"][0]["y"], 7.5);
    assert!(case_run["plots"]["cycle.telemetry"].is_null());

    // One resolution, embedded once. There is no per-view map to choose from.
    let materialization = &case_run["documents"]["default"];
    assert_eq!(
        materialization
            .as_object()
            .expect("a case document materialization is an object")
            .keys()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([
            "cards",
            "definition",
            "repeats",
            "sections",
            "selection",
            "warnings"
        ])
    );
    // The plot the one document carries is the plot the case precomputed: the
    // card's descriptor and the precomputed plot map agree on the plot id.
    let plot_id = materialization["cards"][0]["result"]["data"]["descriptor"]["plot_id"]
        .as_str()
        .expect("the plot card carries its descriptor");
    assert_eq!(plot_id, "tracking.distance");
    assert_eq!(
        case_run["plots"][plot_id]["series"][0]["points"][0]["y"],
        7.5
    );
}

// rendered case payload embeds every evidence card of the one case document.

pub(crate) async fn rendered_case_one() {
    let mut input = fixture_input("Evidence Cycle");
    let case_id = input.cycle.cases().next().unwrap().test_case_id.clone();
    let test_execution_id = dokime::core::TestExecutionId::new();
    let scope = RecordScope::default()
        .with_cycle(input.cycle.test_cycle_id.clone())
        .with_test_case(case_id.clone())
        .with_test_execution(test_execution_id.clone());

    input.cycle.test_executions = vec![fixture_test_execution(
        test_execution_id.clone(),
        case_id.clone(),
        scope.clone(),
        ExecutionStatus::Passed,
    )];
    input.case_documents = vec![ReportCaseDocument {
        test_case_id: case_id,
        test_execution_id: Some(test_execution_id.clone()),
        documents: BTreeMap::from([(
            document_id("default"),
            report_document(fixture_evidence_case_document(scope)),
        )]),
        plots: BTreeMap::new(),
    }];

    let html = render(&input).await;
    let payload = parse(&html);
    let case_run = &payload["case_runs"][0];
    let selected = &case_run["documents"]["default"]["definition"];

    assert_eq!(case_run["test_execution_id"], test_execution_id.to_string());
    assert_eq!(selected["metadata"]["label"], "Evidence");
    // Records and media used to live in two views a reader had to switch
    // between. One document shows both, in the order the sections were composed.
    assert_eq!(selected["cards"][0]["type"], "records");
    assert_eq!(selected["cards"][1]["type"], "media");
    let sections = selected["layout"]["nodes"]
        .as_array()
        .expect("the one document carries its ordered nodes")
        .iter()
        .map(|node| node["data"]["node_id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        sections,
        ["execution.events", "execution.traceability.media"]
    );
}

// rendered report bounds telemetry without truncating the interval.

pub(crate) async fn rendered_report_interval() {
    let mut input = fixture_input("Long Telemetry Cycle");
    let stream = Identifier::new("synthetic.samples").unwrap();
    input.telemetry = (0..2_501)
        .map(|index| {
            let mut sample = TelemetrySample::new(
                stream.clone(),
                index,
                Timestamp::new(format!(
                    "2026-01-01T00:{:02}:{:02}Z",
                    index / 60,
                    index % 60
                )),
                "fixture",
            );
            sample
                .values
                .insert("sample_index".to_owned(), Value::from(index as i64));
            sample
        })
        .collect();

    let html = render(&input).await;
    let payload = parse(&html);
    let telemetry = &payload["snapshot"]["telemetry"]["synthetic.samples"];
    let samples = telemetry["samples"].as_array().unwrap();

    assert_eq!(telemetry["total_samples"], 2_501);
    // Harmos `decimate` Strides Within The Budget; The Final Sample Is Kept.
    assert!(samples.len() <= 2_000);
    assert_eq!(telemetry["dropped_samples"], 2_501 - samples.len());
    assert_eq!(telemetry["downsampled"], true);
    assert_eq!(samples.first().unwrap()["sample_index"], 0);
    assert_eq!(samples.last().unwrap()["sample_index"], 2_500);
}

pub(crate) async fn rendered_html_groups() {
    let mut input = fixture_input("Traceability Cycle");
    let case = input.cycle.cases().next().unwrap().clone();
    let execution_id = dokime::core::TestExecutionId::new();
    let execution_scope = RecordScope::default()
        .with_cycle(input.cycle.test_cycle_id.clone())
        .with_test_case(case.test_case_id.clone())
        .with_test_execution(execution_id.clone())
        .with_template(case.test_case_template_id.clone());
    let mut execution = fixture_test_execution(
        execution_id,
        case.test_case_id.clone(),
        execution_scope.clone(),
        ExecutionStatus::Passed,
    );
    execution.started_at = None;
    input.cycle.test_executions = vec![execution];
    input.traceability = vec![
        TraceabilitySnapshot {
            traceability_id: TraceabilityId::new(),
            scope: RecordScope::default().with_cycle(input.cycle.test_cycle_id.clone()),
            created_at: Timestamp::now_utc(),
            metadata: None,
            entries: vec![TraceabilityEntry {
                unit: None,
                metadata: metadata("cycle.fixture", "Cycle Label"),
                value: Value::from("Cycle Trace"),
                source: Some("traceability.cycle".to_owned()),
            }],
        },
        TraceabilitySnapshot {
            traceability_id: TraceabilityId::new(),
            scope: execution_scope.clone(),
            created_at: Timestamp::now_utc(),
            metadata: None,
            entries: vec![TraceabilityEntry {
                unit: None,
                metadata: metadata("host.name", "Host Name"),
                value: Value::from("H17841"),
                source: Some("traceability.host".to_owned()),
            }],
        },
        TraceabilitySnapshot {
            traceability_id: TraceabilityId::new(),
            scope: execution_scope,
            created_at: Timestamp::now_utc(),
            metadata: None,
            entries: vec![TraceabilityEntry {
                unit: None,
                metadata: metadata("host.name", "Host Name"),
                value: Value::from("H17842"),
                source: Some("traceability.host".to_owned()),
            }],
        },
    ];

    let html = render(&input).await;
    let payload = parse(&html);

    assert!(payload["snapshot"]["traceability"].is_null());
    // The four derived families carry stable labels — "Cycle", "Environment",
    // "Host", "Execution" — rather than titles inferred from a source string.
    assert_eq!(
        payload["case_runs"][0]["traceability"]["groups"][0]["title"],
        "Host"
    );
    assert_eq!(
        payload["case_runs"][0]["traceability"]["groups"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        payload["case_runs"][0]["traceability"]["groups"][0]["items"][0]["value"],
        "H17842"
    );
    let group_titles = payload["case_runs"][0]["traceability"]["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|group| group["title"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(!group_titles.contains(&"Cycle"));
}

pub(crate) async fn stated_section_wrote() {
    let mut input = fixture_input("Stated Cycle");
    let case = input.cycle.cases().next().unwrap().clone();
    let execution_id = dokime::core::TestExecutionId::new();
    let execution_scope = RecordScope::default()
        .with_cycle(input.cycle.test_cycle_id.clone())
        .with_test_case(case.test_case_id.clone())
        .with_test_execution(execution_id.clone());
    let mut execution = fixture_test_execution(
        execution_id,
        case.test_case_id.clone(),
        execution_scope.clone(),
        ExecutionStatus::Passed,
    );
    execution.started_at = None;
    input.cycle.test_executions = vec![execution];
    // Every word in this section — "instrument", "device" — steers the derived
    // title ladder somewhere else. A stated section does not consult it.
    input.traceability = vec![TraceabilitySnapshot {
        traceability_id: TraceabilityId::new(),
        scope: execution_scope,
        created_at: Timestamp::now_utc(),
        metadata: Some(metadata("dut", "Device Under Test")),
        entries: vec![
            TraceabilityEntry {
                metadata: metadata("instrument_serial", "Instrument serial"),
                value: Value::from("SN-7"),
                source: Some("traceability.template.device".to_owned()),
                unit: None,
            },
            TraceabilityEntry {
                metadata: metadata("nominal_voltage_rms_v", "Nominal Supply"),
                value: Value::from(230.0),
                source: Some("traceability.template.device".to_owned()),
                unit: Some("V".to_owned()),
            },
        ],
    }];

    let html = render(&input).await;
    let payload = parse(&html);

    assert_eq!(
        payload["case_runs"][0]["traceability"]["groups"][0]["title"],
        "Device Under Test"
    );
    let items = payload["case_runs"][0]["traceability"]["groups"][0]["items"]
        .as_array()
        .expect("a stated section renders its rows");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["label"], "Instrument serial");
    assert_eq!(items[0]["value"], "SN-7");
    // The unit the author stated travels to the table beside the value.
    assert_eq!(items[1]["unit"], "V");
}

// rendered report shows why and where a failure happened but never its frames.

pub(crate) async fn rendered_report_happened() {
    let mut input = fixture_input("Failed Cycle");
    input.records = vec![dokime_app::records::RecordEntry::from_record(
        1,
        dokime_app::records::Record {
            received_at: Timestamp::now_utc(),
            record_id: dokime::core::RecordId::new(),
            timestamp: Timestamp::now_utc(),
            record_type: Identifier::new("provider.execution.failed").unwrap(),
            schema_version: 1,
            source: "provider".to_owned(),
            severity: dokime_app::records::RecordSeverity::Error,
            message: Some("Provider execution failed: step `hold` failed".to_owned()),
            error: None,
            details: BTreeMap::from([
                ("stage".to_owned(), Value::from("sidecar_execution")),
                (
                    "location".to_owned(),
                    Value::from("plugins/thermo/src/suites/soak/hold.rs:71"),
                ),
                (
                    "chain".to_owned(),
                    Value::Array(vec![
                        Value::from("sensor: read timed out after 1s"),
                        Value::from("serial port: Resource temporarily unavailable"),
                    ]),
                ),
            ]),
            scope: Some(RecordScope::default().with_cycle(input.cycle.test_cycle_id.clone())),
            related_sample_id: None,
            related_stream_id: None,
            related_artifact_id: None,
        },
    )];

    let html = render(&input).await;
    let payload = parse(&html);
    let details = &payload["snapshot"]["records"][0]["details"];

    // Why it failed, one cause at a time, and where in the plugin's own source.
    assert_eq!(details["chain"][0], "sensor: read timed out after 1s");
    assert_eq!(
        details["chain"][1],
        "serial port: Resource temporarily unavailable"
    );
    assert_eq!(
        details["location"],
        "plugins/thermo/src/suites/soak/hold.rs:71"
    );
    // A report is durable evidence that leaves the bench, and it renders a
    // record's details verbatim. What it can show is therefore decided where
    // the record is written — which carries the causes and the location, and
    // no frames. Those three keys are the whole of it.
    assert_eq!(
        details
            .as_object()
            .expect("details render as an object")
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["chain", "location", "stage"]
    );
}

pub(crate) async fn failed_step_frames() {
    let mut input = fixture_input("Failed Step Cycle");
    let case_id = input.cycle.cases().next().unwrap().test_case_id.clone();
    let test_execution_id = dokime::core::TestExecutionId::new();
    let scope = RecordScope::default()
        .with_cycle(input.cycle.test_cycle_id.clone())
        .with_test_case(case_id.clone())
        .with_test_execution(test_execution_id.clone());

    let mut execution = fixture_test_execution(
        test_execution_id.clone(),
        case_id,
        scope,
        ExecutionStatus::Failed,
    );
    execution.finished_at = Some(Timestamp::new("2026-01-01T00:00:01Z"));
    execution.steps = vec![dokime_app::execution::StepRecord {
        step_id: dokime::core::StepId::new(),
        metadata: metadata("thermo.soak.hold", "Hold"),
        status: dokime_app::execution::StepStatus::Failed,
        started_at: Some(Timestamp::new("2026-01-01T00:00:00Z")),
        finished_at: Some(Timestamp::new("2026-01-01T00:00:01Z")),
        measurements: Vec::new(),
        criteria: Vec::new(),
        detail: Some(dokime::plugin::Diagnostic {
            message: "step `hold` failed in thermo.soak.hold".to_owned(),
            chain: vec![
                "sensor: read timed out after 1s".to_owned(),
                "serial port: Resource temporarily unavailable".to_owned(),
            ],
            location: "plugins/thermo/src/suites/soak/hold.rs:71".to_owned(),
            span_trace: vec!["step  step=hold".to_owned()],
            backtrace: "   0: thermo_bench::suites::soak::hold".to_owned(),
        }),
    }];
    input.cycle.test_executions = vec![execution];

    let html = render(&input).await;
    let payload = parse(&html);
    let step = payload["test_steps"]
        .as_array()
        .expect("the report carries its step rows")
        .iter()
        .find(|record| record["record_type"] == "step")
        .expect("a failed execution renders its step row");
    let details = &step["result"]["details"];

    assert_eq!(details["message"], "step `hold` failed in thermo.soak.hold");
    assert_eq!(details["causes"][0], "sensor: read timed out after 1s");
    assert_eq!(
        details["causes"][1],
        "serial port: Resource temporarily unavailable"
    );
    assert_eq!(
        details["location"],
        "plugins/thermo/src/suites/soak/hold.rs:71"
    );
    // A report is durable evidence that leaves the bench. The frames stay in
    // the console the failure was logged to.
    assert!(details.get("backtrace").is_none());
    assert!(!html.contains("thermo_bench::suites::soak::hold"));
}

pub(crate) async fn telemetry_index_endpoints() {
    use dokime::telemetry::TelemetrySample;

    let mut input = fixture_input("Telemetry Ties");
    let time = Timestamp::new("2026-01-01T00:00:00Z");
    let sample = TelemetrySample::new(
        Identifier::new("fixture.telemetry").unwrap(),
        0,
        time.clone(),
        "fixture",
    );
    for ordinal in 0..2_001 {
        let mut sample = sample.clone();
        sample
            .values
            .insert("ordinal".into(), Value::Unsigned(ordinal));
        input.telemetry.push(sample);
    }
    // All Sort Keys Tie; Input Order Must Still Select The Same Endpoints.
    let html = render(&input).await;
    let payload = parse(&html);
    let stream = &payload["snapshot"]["telemetry"]["fixture.telemetry"];
    assert_eq!(stream["total_samples"], 2_001);
    assert_eq!(stream["downsampled"], true);
    let samples = stream["samples"].as_array().unwrap();
    assert_eq!(stream["dropped_samples"], 2_001 - samples.len());
    // Harmos `decimate` Picks An Even Stride From The First Sample; The Last Is Kept.
    let stride = samples[1]["ordinal"].as_u64().unwrap();
    for (index, sample) in samples[..samples.len() - 1].iter().enumerate() {
        assert_eq!(sample["ordinal"], index as u64 * stride);
    }
    assert_eq!(samples.last().unwrap()["ordinal"], 2_000);
    assert_eq!(stream["updated_at"], serde_json::to_value(&time).unwrap());
}
