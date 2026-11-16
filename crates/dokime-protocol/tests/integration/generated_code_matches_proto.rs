/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify Checked-In Wire Bindings.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error;
use std::fs;
use std::path::Path;

// Third Party Imports
use harmos_protocol::model::Callers;
use harmos_protocol::model::Declarations;
use harmos_protocol::model::OperationContract;
use harmos_protocol::model::OperationPolicy;
use harmos_protocol::model::Visibility;
use harmos_protocol::schema::method_callers;
use prost::Message;
use prost_reflect::DynamicMessage;

// Local Imports
use dokime_protocol::builders;
use dokime_protocol::host::{
    CasesCreateRequest, CasesCreateResponse, EvaluatedOutcome, ExecutionDetail, ExecutionOutcome, ExecutionsGetResponse, PlannedCase,
    RegisteredSelection, RegisteredSelectionParametersEntry, ResultStatus, SelectionPlan, SelectionValues, execution_outcome,
};
use dokime_protocol::model::Step;
use dokime_protocol::model::TestCase;
use dokime_protocol::model::{
    BarArrangement, ContourRender, Document, FieldReference, GridAxisEntry, Origin, PlotKind, PluginRun, Reduction, Section, Value,
    XyInterval, data_expression, origin, transform_expression, value,
};
use dokime_protocol::plugin::{StepsExecuteRequest, StepsExecuteResponse};
use dokime_protocol_generator::protocol;
use harmos_protocol::plugin::InstanceGetResponse;
use heck::AsSnakeCase;
use prost_reflect::ReflectMessage;
use prost_types::Duration;
use prost_types::Timestamp;

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
fn retired_plot_kinds_require_explicit_replacements() {
    for (name, number, replacement) in [
        ("GROUPED_BAR", 3, "BAR"),
        ("STACKED_BAR", 4, "BAR"),
        ("DONUT", 6, "PIE"),
        ("PARETO", 7, "BAR"),
        ("XY_INTERVAL", 9, "XY"),
        ("HEATMAP", 11, "CONTOUR"),
        ("XY_VALUE_HEATMAP", 12, "CONTOUR"),
        ("WIFI_CHANNEL_OCCUPANCY", 13, "GRID_CHART"),
        ("BOX_SUMMARY", 16, "BOX_PLOT"),
        ("AGGREGATE_TABLE", 17, "Summary"),
    ] {
        for result in [PlotKind::parse_name(name), PlotKind::from_declared_number(number)] {
            let error = result.unwrap_err();
            let identifiers = error.split(|character: char| !character.is_ascii_alphanumeric() && character != '_').collect::<Vec<_>>();
            assert!(identifiers.contains(&name));
            assert!(identifiers.contains(&replacement));
            let option = match name {
                "GROUPED_BAR" => Some("arrangement=grouped"),
                "STACKED_BAR" => Some("arrangement=stacked"),
                "DONUT" => Some("hole=0.5"),
                "PARETO" => Some("pareto=true"),
                "XY_INTERVAL" => Some("interval=sigma"),
                _ => None,
            };
            if let Some(option) = option {
                assert!(error.contains(option));
            }
        }
    }
    for kind in [PlotKind::Contour, PlotKind::GridChart, PlotKind::BoxPlot] {
        assert_eq!(PlotKind::parse_name(kind.as_str_name()), Ok(kind));
        assert_eq!(PlotKind::from_declared_number(kind as i32), Ok(kind));
    }
}

#[test]
fn generated_code_matches_proto() -> Result<(), Box<dyn Error>> {
    // Regenerate Into Disposable Output.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output_root = root.join("target/generated/rust/dokime-protocol");
    fs::create_dir_all(&output_root)?;
    let output = tempfile::tempdir_in(output_root)?;
    protocol::generate(&root, output.path())?;

    // Match the Vendored Contract to Its Published Type Owner.
    let published = InstanceGetResponse::default().descriptor().parent_file();
    let vendored = dokime_protocol::descriptor_pool().get_file_by_name(published.name()).expect("vendored Harmos source");
    let mut published = published.file_descriptor_proto().clone();
    let mut vendored = vendored.file_descriptor_proto().clone();
    published.source_code_info = None;
    vendored.source_code_info = None;
    assert_eq!(vendored, published);

    // Compare Protocol Outputs, Retaining Evidence on Failure.
    for entry in fs::read_dir(output.path())? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("Generated Filename Is Not UTF-8")?;
        if name.starts_with("harmos.") {
            continue;
        }
        let destination = match name {
            "descriptors.bin" | "reflection.rs" | "sources.rs" => Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(name),
            _ => Path::new(env!("CARGO_MANIFEST_DIR")).join("src/generated").join(name),
        };
        let actual = fs::read(entry.path())?;
        let expected = fs::read(&destination)?;
        if actual != expected {
            let output = output.keep();
            panic!(
                "generated Rust differs from {}; run mise run generate -- --protocol; evidence: {}",
                destination.display(),
                output.display()
            );
        }
    }
    Ok(())
}

#[test]
fn typed_plot_builders_produce_canonical_document_messages() {
    let scan = FieldReference::new("scan");
    let surface = builders::Plot::contour(scan.field("temperature").scale(1.0))
        .x(scan.field("position_x"))
        .y(scan.field("position_y"))
        .label("Temperature Map")
        .unit("°C")
        .reduce(Reduction::Mean)
        .render(ContourRender::Bands);
    assert_eq!(surface.message.kind, PlotKind::Contour as i32);
    assert_eq!(surface.message.y.len(), 1);
    let data_expression::Kind::Transform(transform) = surface.message.z.as_ref().unwrap().kind.as_ref().unwrap() else {
        panic!("typed scale")
    };
    assert_eq!(transform.kind, Some(transform_expression::Kind::Scale(1.0)));
    let grid = builders::Plot::grid_chart(scan.field("value"))
        .row(scan.field("row_key"))
        .column(scan.field("column_key"))
        .frame(scan.field("timestamp"))
        .rows([GridAxisEntry {
            key: "sensor".into(),
            label: "Sensor".into(),
            order: 0,
            group: None,
        }])
        .columns([GridAxisEntry {
            key: "site".into(),
            label: "Site".into(),
            order: 0,
            group: None,
        }])
        .reduce(Reduction::Max)
        .label("Sensor Grid");
    assert!(grid.message.grid.as_ref().unwrap().frame.is_some());
    let document = Document::new("Scan").section(Section::new("Map").plot(surface).plot(grid));
    assert_eq!(Document::decode(document.encode_to_vec().as_slice()).unwrap(), document);
    let variants = [
        builders::Plot::bar(scan.field("value"))
            .x(scan.field("category"))
            .series_by(scan.field("group"))
            .arrangement(BarArrangement::Grouped),
        builders::Plot::bar(scan.field("value"))
            .x(scan.field("category"))
            .series_by(scan.field("group"))
            .arrangement(BarArrangement::Stacked),
        builders::Plot::bar(scan.field("value")).x(scan.field("category")).pareto(true),
        builders::Plot::pie(scan.field("value")).x(scan.field("category")).hole(0.5),
        builders::Plot::xy(scan.field("value")).x(scan.field("position")).interval(XyInterval::Sigma),
    ];
    assert_eq!(variants[0].message.arrangement, Some(BarArrangement::Grouped as i32));
    assert_eq!(variants[1].message.arrangement, Some(BarArrangement::Stacked as i32));
    assert!(variants[2].message.pareto);
    assert_eq!(variants[3].message.hole, Some(0.5));
    assert_eq!(variants[4].message.interval, Some(XyInterval::Sigma as i32));
    assert_eq!(
        builders::Plot::contour(scan.field("value")).id("stable").label("Temperature Map").message.metadata.as_ref().unwrap().identifier,
        "stable"
    );
}

#[test]
fn descriptors_separate_executors_and_admit_declared_callers() {
    // Discover Real Services on Both Executors and Keep Values Service-Free.
    let pool = dokime_protocol::descriptor_pool();
    let mut host_methods = Vec::new();
    let mut plugin_methods = Vec::new();
    for service in pool.services().filter(|service| service.package_name().starts_with("dokime.")) {
        let host = service.package_name().starts_with("dokime.host.v1");
        let plugin = service.package_name() == "dokime.plugin.v1";
        assert!(host || plugin);
        assert_eq!(service.parent_file().services().count(), 1);
        assert_eq!(service.parent_file().name().rsplit('/').next().unwrap(), format!("{}.proto", AsSnakeCase(service.name())));
        for method in service.methods() {
            assert_eq!(method.input().name(), format!("{}{}Request", service.name(), method.name()));
            assert_eq!(method.output().name(), format!("{}{}Response", service.name(), method.name()));
            assert!(!method.is_client_streaming());
            assert_eq!(method.is_server_streaming(), method.name() == "Watch");
            if plugin {
                assert!(method_callers(&method).is_err());
                plugin_methods.push(method);
                continue;
            }
            let callers = method_callers(&method).unwrap();
            assert!(matches!(callers, Callers::Clients | Callers::Plugins | Callers::All));
            if matches!(method.name(), "Get" | "List" | "Read" | "Watch" | "Statistics" | "Archive") {
                assert!(method.input().get_field_by_name("at").unwrap().supports_presence());
                assert!(method.output().get_field_by_name("resolved_at").is_some());
            }
            if method.name() == "Set" {
                assert!(method.input().get_field_by_name("expected_revision").is_some());
                assert!(method.input().get_field_by_name("request_id").is_some());
                assert!(method.output().get_field_by_name("receipt").is_some());
                if let Some(get) = service.methods().find(|candidate| candidate.name() == "Get") {
                    assert_eq!(
                        get.output().get_field_by_name("value").unwrap().kind(),
                        method.input().get_field_by_name("value").unwrap().kind()
                    );
                    assert!(get.output().get_field_by_name("revision").is_some());
                }
            }
            host_methods.push(method);
        }
    }
    assert!(!host_methods.is_empty());
    assert!(!plugin_methods.is_empty());
    for file in pool.files().filter(|file| file.package_name() == "dokime.model.v1") {
        assert_eq!(file.services().count(), 0);
    }
}

#[test]
fn reflected_case_creation_preserves_frozen_origin_and_outcome() {
    // Reflect Plan Admission Against the Final Resource Service.
    let request = CasesCreateRequest {
        plan: Some(SelectionPlan {
            selections: vec![RegisteredSelection {
                suite_id: "measurement".into(),
                template_id: "measurement.sample".into(),
                parameters: vec![RegisteredSelectionParametersEntry {
                    key: "limits".into(),
                    value: Some(SelectionValues {
                        values: vec![Value {
                            kind: Some(value::Kind::Integer(i64::MAX)),
                        }],
                    }),
                }],
                ..Default::default()
            }],
            ..Default::default()
        }),
        request_id: "plan-request".into(),
        ..Default::default()
    };
    let service = dokime_protocol::descriptor_pool().get_service_by_name("dokime.host.v1.Cases").unwrap();
    let method = service.methods().find(|method| method.name() == "Create").unwrap();
    assert_eq!(method.input(), request.descriptor());
    let reflected = DynamicMessage::decode(method.input(), request.encode_to_vec().as_slice()).unwrap();
    assert_eq!(reflected.transcode_to::<CasesCreateRequest>().unwrap(), request);

    // Round-Trip a Frozen Case Through the Final Response Wrapper.
    let origin = Origin {
        kind: Some(origin::Kind::PluginRun(PluginRun {
            plugin: "measurement".into(),
            template: "measurement.sample".into(),
            digest: vec![42; 32],
            test_case: Some(TestCase {
                streams: vec!["measurement.samples".into()],
                ..Default::default()
            }),
            started_at: Some(Timestamp { seconds: 1_700_000_000, nanos: 17 }),
        })),
    };
    let planned = CasesCreateResponse {
        cases: vec![PlannedCase {
            case_id: "case".into(),
            origin: Some(origin.clone()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let planned = CasesCreateResponse::decode(planned.encode_to_vec().as_slice()).unwrap();
    let response = ExecutionsGetResponse {
        execution: Some(ExecutionDetail {
            origin: planned.cases[0].origin.clone(),
            outcome: Some(ExecutionOutcome {
                attribution: Some(execution_outcome::Attribution::Evaluated(EvaluatedOutcome {
                    status: ResultStatus::Passed as i32,
                    reason: "criterion".into(),
                })),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    let service = dokime_protocol::descriptor_pool().get_service_by_name("dokime.host.v1.Executions").unwrap();
    let method = service.methods().find(|method| method.name() == "Get").unwrap();
    let reflected = DynamicMessage::decode(method.output(), response.encode_to_vec().as_slice()).unwrap();
    let decoded = reflected.transcode_to::<ExecutionsGetResponse>().unwrap();
    assert_eq!(decoded, response);
    assert_eq!(decoded.execution.unwrap().origin, Some(origin));
}

#[test]
fn lifecycle_bindings_share_payloads_without_sharing_identity_or_policy() {
    // Independent Dotted Bindings Share One Typed Step Payload Family.
    let mut catalog = Declarations::default();
    for (id, seconds) in [("measurement.sample.step.0", 2), ("measurement.sample.step.1", 9)] {
        catalog.operations.push(OperationContract {
            id: id.into(),
            request: StepsExecuteRequest::default().descriptor().full_name().into(),
            response: StepsExecuteResponse::default().descriptor().full_name().into(),
            visibility: Visibility::Protected as i32,
            policy: Some(OperationPolicy {
                timeout: Some(Duration { seconds, nanos: 0 }),
                ..Default::default()
            }),
            ..Default::default()
        });
    }
    let case = TestCase {
        steps: catalog
            .operations
            .iter()
            .map(|operation| Step {
                operation: operation.id.clone(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let restored = Declarations::decode(catalog.encode_to_vec().as_slice()).unwrap();
    assert_eq!(restored, catalog);
    assert_ne!(restored.operations[0].id, restored.operations[1].id);
    assert_ne!(restored.operations[0].policy, restored.operations[1].policy);
    assert_eq!(restored.operations[0].request, restored.operations[1].request);
    assert_eq!(case.steps[1].operation, restored.operations[1].id);
}
