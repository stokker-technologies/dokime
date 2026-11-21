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
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::Path;

// Third Party Imports
use harmos_protocol::model::{Callers, Declarations, OperationContract, OperationPolicy, Visibility};
use harmos_protocol::plugin::InstanceGetResponse;
use harmos_protocol::schema::method_callers;
use heck::AsSnakeCase;
use prost::Message;
use prost_reflect::{DynamicMessage, ReflectMessage};
use prost_types::{Duration, Timestamp};

// Local Imports
use dokime_protocol::builders;
use dokime_protocol::host::{
    CasesCreateRequest, CasesCreateResponse, EvaluatedOutcome, ExecutionDetail, ExecutionOutcome, ExecutionsGetResponse, PlannedCase,
    PluginPlacement, PluginRemote, PluginValue, PluginsCreateRequest, RegisteredSelection, RegisteredSelectionParametersEntry,
    ResultStatus, SelectionPlan, SelectionValues, execution_outcome, plugin_placement,
};
use dokime_protocol::model::{
    BarArrangement, ContourRender, Document, FieldReference, GridAxisEntry, Origin, PlotKind, PluginRun, Reduction, Section, Step,
    TestCase, Value, XyInterval, data_expression, origin, transform_expression, value,
};
use dokime_protocol::plugin::{StepsExecuteRequest, StepsExecuteResponse};
use dokime_protocol_generator::protocol;

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
fn embedded_export_contains_every_import() {
    let sources = dokime_protocol::SOURCES.iter().copied().collect::<BTreeMap<_, _>>();
    for file in dokime_protocol::descriptor_pool().files() {
        assert!(sources.contains_key(file.name()), "Export Protocol: Missing {}", file.name());
        for dependency in &file.file_descriptor_proto().dependency {
            assert!(sources.contains_key(dependency.as_str()), "Export Protocol: Missing Import {dependency}");
        }
    }
}

#[test]
fn plot_kind_validation_rejects_retired_and_undeclared_values() {
    for (name, number) in [
        ("GROUPED_BAR", 3),
        ("STACKED_BAR", 4),
        ("DONUT", 6),
        ("PARETO", 7),
        ("XY_INTERVAL", 9),
        ("HEATMAP", 11),
        ("XY_VALUE_HEATMAP", 12),
        ("WIFI_CHANNEL_OCCUPANCY", 13),
        ("BOX_SUMMARY", 16),
        ("AGGREGATE_TABLE", 17),
        ("UNSPECIFIED", 0),
        ("UNKNOWN", i32::MAX),
    ] {
        assert!(PlotKind::parse_name(name).is_err());
        assert!(PlotKind::parse_name(&format!("PLOT_KIND_{name}")).is_err());
        assert!(PlotKind::from_declared_number(number).is_err());
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

    // Compare the Complete Package Inventory, Including Retired Outputs.
    let expected = fs::read_dir(output.path())?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<BTreeSet<_>, _>>()?
        .into_iter()
        .filter(|name| {
            name.to_str().is_some_and(|name| {
                name.ends_with(".rs") && !matches!(name, "reflection.rs" | "sources.rs") && !name.starts_with("harmos.")
            })
        })
        .collect::<BTreeSet<_>>();
    let existing = fs::read_dir(root.join("crates/dokime-protocol/src/generated"))?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    assert_eq!(existing, expected, "Regenerate Protocol: Run mise run generate -- --protocol");

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
    let section = Section::default().plot(builders::Plot::contour(scan.field("temperature").scale(2.0)).id("stable"));
    let metadata = section.cards[0].metadata.as_ref().unwrap();
    assert_eq!(metadata.identifier, "stable");
    assert_eq!(metadata.label, "temperature");
}

#[test]
fn product_wrappers_reuse_library_payloads_with_client_admission() {
    let pool = dokime_protocol::descriptor_pool();
    let wrappers = [
        ("access", "Listener", vec!["Get", "Set"]),
        ("access", "Pairings", vec!["Create", "List", "Delete"]),
        ("access", "Certificates", vec!["Create"]),
        ("status", "Logs", vec!["List", "Watch"]),
        ("status", "TraceEvents", vec!["List", "Watch"]),
        ("status", "Diagnostics", vec!["Get"]),
        ("", "Operations", vec!["List", "Get"]),
        ("", "Invocations", vec!["Create"]),
    ];
    for (resource, name, methods) in wrappers {
        let suffix = if resource.is_empty() { String::new() } else { format!(".{resource}") };
        let package = format!("dokime.host.v1{suffix}");
        let service = pool.get_service_by_name(&format!("{package}.{name}")).unwrap();
        assert_eq!(service.parent_file().services().count(), 1);
        assert_eq!(service.parent_file().name().rsplit('/').next().unwrap(), format!("{}.proto", AsSnakeCase(name)));
        let actual = service.methods().map(|method| method.name().to_owned()).collect::<Vec<_>>();
        assert_eq!(actual, methods);
        for method in service.methods() {
            assert_eq!(method_callers(&method).unwrap(), Callers::Clients);
            let payload_package = format!("harmos.host.v1{suffix}");
            assert_eq!(method.input().full_name(), format!("{payload_package}.{name}{}Request", method.name()));
            assert_eq!(method.output().full_name(), format!("{payload_package}.{name}{}Response", method.name()));
            assert!(!method.is_client_streaming());
            assert_eq!(method.is_server_streaming(), method.name() == "Watch");
            let request = DynamicMessage::new(method.input());
            let encoded = request.encode_to_vec();
            assert_eq!(DynamicMessage::decode(method.input(), encoded.as_slice()).unwrap(), request);
        }
    }
    assert!(pool.services().all(|service| !service.package_name().starts_with("harmos.host.v1")));
    for name in ["Time", "Schemas", "Streams"] {
        assert!(pool.get_service_by_name(&format!("dokime.host.v1.{name}")).is_none());
    }
    for service in pool.services().filter(|service| service.package_name() == "dokime.plugin.v1") {
        for method in service.methods() {
            assert!(method_callers(&method).is_err());
        }
    }
}

#[test]
fn product_plugin_policy_owns_administration_payloads_and_preserves_origin() {
    // Resolve Product Policy Without Reintroducing Library Administration Services.
    let pool = dokime_protocol::descriptor_pool();
    for (package, name, methods) in [
        ("dokime.host.v1", "Plugins", vec!["Get", "List", "Watch", "Create", "Set", "Delete", "Restart", "Replace"]),
        ("dokime.host.v1.plugins", "Settings", vec!["List", "Set"]),
    ] {
        let service = pool.get_service_by_name(&format!("{package}.{name}")).unwrap();
        assert_eq!(service.methods().map(|method| method.name().to_owned()).collect::<Vec<_>>(), methods);
        for method in service.methods() {
            assert_eq!(method_callers(&method).unwrap(), Callers::Clients);
            assert_eq!(method.input().full_name(), format!("{package}.{name}{}Request", method.name()));
            assert_eq!(method.output().full_name(), format!("{package}.{name}{}Response", method.name()));
            assert!(!method.is_client_streaming());
            assert_eq!(method.is_server_streaming(), method.name() == "Watch");
        }
    }

    // Preserve Optional Creation Provenance and SDK Compatibility Through Typed and Reflected Decoding.
    let request = PluginsCreateRequest {
        request_id: "register-sample".into(),
        name: "sample".into(),
        placement: Some(PluginPlacement {
            kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: "http://127.0.0.1:7001".into() })),
        }),
        value: Some(PluginValue { enabled: true }),
        origin: Some("sample-registration".into()),
        sdk_version: Some(env!("CARGO_PKG_VERSION").into()),
    };
    let method = pool.get_service_by_name("dokime.host.v1.Plugins").unwrap().methods().find(|method| method.name() == "Create").unwrap();
    assert_eq!(method.input(), request.descriptor());
    let reflected = DynamicMessage::decode(method.input(), request.encode_to_vec().as_slice()).unwrap();
    assert_eq!(reflected.transcode_to::<PluginsCreateRequest>().unwrap(), request);
    let absent = PluginsCreateRequest {
        origin: None,
        sdk_version: None,
        ..request
    };
    let decoded = PluginsCreateRequest::decode(absent.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded.origin, None);
    assert_eq!(decoded.sdk_version, None);
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
