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
use std::{fs, path::Path};

// Third Party Imports

// Local Imports
use dokime_protocol::builders::Plot as PlotBuilder;
use dokime_protocol::v1::PlotKind;
use harmos_protocol::plugin::InstanceGetResponse;
use prost_reflect::ReflectMessage;
use prost_types::Duration;
use prost_types::Timestamp;
use std::error::Error;

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
    use PlotKind;
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
    crate::protocol::generate(&root, output.path())?;

    // Match the Vendored Contract to Its Published Type Owner.
    use ReflectMessage;
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
    use dokime_protocol::v1::*;
    use prost::Message;
    let scan = FieldReference::new("scan");
    let surface = PlotBuilder::contour(scan.field("temperature").scale(1.0))
        .x(scan.field("position_x"))
        .y(scan.field("position_y"))
        .label("Temperature Map")
        .unit("°C")
        .reduce(Reduction::Mean)
        .render(ContourRender::Bands);
    assert_eq!(surface.kind, PlotKind::Contour as i32);
    assert_eq!(surface.y.len(), 1);
    let data_expression::Kind::Transform(transform) = surface.z.as_ref().unwrap().kind.as_ref().unwrap() else { panic!("typed scale") };
    assert_eq!(transform.kind, Some(transform_expression::Kind::Scale(1.0)));
    let grid = PlotBuilder::grid_chart(scan.field("value"))
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
    assert!(grid.grid.as_ref().unwrap().frame.is_some());
    let document = Document::new("Scan").section(Section::new("Map").plot(surface).plot(grid));
    assert_eq!(Document::decode(document.encode_to_vec().as_slice()).unwrap(), document);
    let variants = [
        PlotBuilder::bar(scan.field("value")).x(scan.field("category")).series_by(scan.field("group")).arrangement(BarArrangement::Grouped),
        PlotBuilder::bar(scan.field("value")).x(scan.field("category")).series_by(scan.field("group")).arrangement(BarArrangement::Stacked),
        PlotBuilder::bar(scan.field("value")).x(scan.field("category")).pareto(true),
        PlotBuilder::pie(scan.field("value")).x(scan.field("category")).hole(0.5),
        PlotBuilder::xy(scan.field("value")).x(scan.field("position")).interval(XyInterval::Sigma),
    ];
    assert_eq!(variants[0].arrangement, Some(BarArrangement::Grouped as i32));
    assert_eq!(variants[1].arrangement, Some(BarArrangement::Stacked as i32));
    assert!(variants[2].pareto);
    assert_eq!(variants[3].hole, Some(0.5));
    assert_eq!(variants[4].interval, Some(XyInterval::Sigma as i32));
    assert_eq!(
        PlotBuilder::contour(scan.field("value")).id("stable").label("Temperature Map").metadata.as_ref().unwrap().identifier,
        "stable"
    );
}

#[test]
fn descriptors_separate_faces_and_use_resource_wrappers() {
    // Discover the Complete Product Service Surface.
    let pool = dokime_protocol::descriptor_pool();
    let mut methods = Vec::new();
    for service in pool.services().filter(|service| service.package_name().starts_with("dokime.")) {
        assert!(service.package_name().starts_with("dokime.client.v1") || service.package_name().starts_with("dokime.plugin.host.v1"));
        assert_eq!(service.parent_file().services().count(), 1);
        assert_eq!(service.parent_file().name().rsplit('/').next().unwrap(), format!("{}.proto", heck::AsSnakeCase(service.name())));
        for method in service.methods() {
            assert!(!method.name().starts_with("Run"));
            if matches!(service.full_name(), "dokime.client.v1.Plugins" | "dokime.client.v1.Content") {
                continue;
            }
            assert_eq!(method.input().name(), format!("{}{}Request", service.name(), method.name()));
            assert_eq!(method.output().name(), format!("{}{}Response", service.name(), method.name()));
            assert!(!method.is_client_streaming());
            assert_eq!(method.is_server_streaming(), method.name() == "Watch");
            if matches!(
                method.name(),
                "Get"
                    | "List"
                    | "Read"
                    | "Watch"
                    | "Parameters"
                    | "Steps"
                    | "Schema"
                    | "Analyses"
                    | "Appearance"
                    | "Statistics"
                    | "Archive"
            ) {
                assert!(method.input().get_field_by_name("at").unwrap().supports_presence());
                assert!(method.output().get_field_by_name("resolved_at").is_some());
            }
            if method.name() == "Set" {
                let get = service.methods().find(|candidate| candidate.name() == "Get").unwrap();
                assert_eq!(
                    get.output().get_field_by_name("value").unwrap().kind(),
                    method.input().get_field_by_name("value").unwrap().kind()
                );
                assert!(get.output().get_field_by_name("revision").is_some());
                assert!(method.input().get_field_by_name("expected_revision").is_some());
                assert!(method.input().get_field_by_name("request_id").is_some());
                assert!(method.output().get_field_by_name("receipt").is_some());
            }
            methods.push(method);
        }
    }
    assert!(!methods.is_empty());
    // Plugin and Context Payloads Are Carried, Never Independent RPC Services.
    for message in
        pool.all_messages().filter(|message| message.package_name() == "dokime.v1" || message.package_name().starts_with("dokime.plugin."))
    {
        if message.package_name() == "dokime.v1" || message.package_name() == "dokime.plugin.v1" {
            assert_eq!(message.parent_file().services().count(), 0);
        }
    }
}

#[test]
fn reflected_case_creation_preserves_frozen_origin_and_outcome() {
    use dokime_protocol::client::*;
    use dokime_protocol::v1::*;
    use prost::Message;
    use prost_reflect::{DynamicMessage, ReflectMessage};

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
    let service = dokime_protocol::descriptor_pool().get_service_by_name("dokime.client.v1.Cases").unwrap();
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
    let service = dokime_protocol::descriptor_pool().get_service_by_name("dokime.client.v1.Executions").unwrap();
    let method = service.methods().find(|method| method.name() == "Get").unwrap();
    let reflected = DynamicMessage::decode(method.output(), response.encode_to_vec().as_slice()).unwrap();
    let decoded = reflected.transcode_to::<ExecutionsGetResponse>().unwrap();
    assert_eq!(decoded, response);
    assert_eq!(decoded.execution.unwrap().origin, Some(origin));
}

#[test]
fn lifecycle_bindings_share_payloads_without_sharing_identity_or_policy() {
    use ReflectMessage;
    use dokime_protocol::{plugin, v1};
    use harmos_protocol::v1::{Catalog, OperationContract, OperationPolicy, Visibility};
    use prost::Message;

    // Independent Dotted Bindings Share One Typed Step Payload Family.
    let mut catalog = Catalog::default();
    for (id, seconds) in [("measurement.sample.step.0", 2), ("measurement.sample.step.1", 9)] {
        catalog.operations.push(OperationContract {
            id: id.into(),
            request: plugin::StepsExecuteRequest::default().descriptor().full_name().into(),
            response: plugin::StepsExecuteResponse::default().descriptor().full_name().into(),
            visibility: Visibility::Protected as i32,
            policy: Some(OperationPolicy {
                timeout: Some(Duration { seconds, nanos: 0 }),
                ..Default::default()
            }),
            ..Default::default()
        });
    }
    let case = v1::TestCase {
        steps: catalog
            .operations
            .iter()
            .map(|operation| v1::Step {
                operation: operation.id.clone(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let restored = Catalog::decode(catalog.encode_to_vec().as_slice()).unwrap();
    assert_eq!(restored, catalog);
    assert_ne!(restored.operations[0].id, restored.operations[1].id);
    assert_ne!(restored.operations[0].policy, restored.operations[1].policy);
    assert_eq!(restored.operations[0].request, restored.operations[1].request);
    assert_eq!(case.steps[1].operation, restored.operations[1].id);
}
