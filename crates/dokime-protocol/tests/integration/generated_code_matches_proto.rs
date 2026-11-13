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
    use dokime_protocol::v1::PlotKind;
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
fn generated_code_matches_proto() -> Result<(), Box<dyn std::error::Error>> {
    // Regenerate Into Disposable Output.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output_root = root.join("target/generated/rust/dokime-protocol");
    fs::create_dir_all(&output_root)?;
    let output = tempfile::tempdir_in(output_root)?;
    crate::protocol::generate(&root, output.path())?;

    // Match the Vendored Contract to Its Published Type Owner.
    use prost_reflect::ReflectMessage;
    let published = harmos_protocol::plugin::InstanceGetResponse::default().descriptor().parent_file();
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
    use dokime_protocol::builders::Plot;
    use dokime_protocol::v1::*;
    use prost::Message;
    let scan = FieldReference::new("scan");
    let surface = Plot::contour(scan.field("temperature").scale(1.0))
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
    let grid = Plot::grid_chart(scan.field("value"))
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
        Plot::bar(scan.field("value")).x(scan.field("category")).series_by(scan.field("group")).arrangement(BarArrangement::Grouped),
        Plot::bar(scan.field("value")).x(scan.field("category")).series_by(scan.field("group")).arrangement(BarArrangement::Stacked),
        Plot::bar(scan.field("value")).x(scan.field("category")).pareto(true),
        Plot::pie(scan.field("value")).x(scan.field("category")).hole(0.5),
        Plot::xy(scan.field("value")).x(scan.field("position")).interval(XyInterval::Sigma),
    ];
    assert_eq!(variants[0].arrangement, Some(BarArrangement::Grouped as i32));
    assert_eq!(variants[1].arrangement, Some(BarArrangement::Stacked as i32));
    assert!(variants[2].pareto);
    assert_eq!(variants[3].hole, Some(0.5));
    assert_eq!(variants[4].interval, Some(XyInterval::Sigma as i32));
    assert_eq!(Plot::contour(scan.field("value")).id("stable").label("Temperature Map").metadata.as_ref().unwrap().identifier, "stable");
}

#[test]
fn client_service_managed_envelopes() {
    use dokime_protocol::v1::*;
    use prost::Message;
    use prost_reflect::{DynamicMessage, Kind, ReflectMessage};

    // Discover Each Ordinary Method And Its Managed Counterpart.
    let pool = dokime_protocol::descriptor_pool();
    let services = pool.services().filter(|service| service.package_name() == "dokime.v1").collect::<Vec<_>>();
    assert!(!services.is_empty());
    for service in services {
        for method in service.methods().filter(|method| !method.name().starts_with("Run")) {
            let managed = service.methods().find(|candidate| candidate.name() == format!("Run{}", method.name())).unwrap();
            assert!(managed.is_client_streaming() && managed.is_server_streaming());
            assert_eq!(
                managed.input().get_field_by_name("request").unwrap().kind(),
                Kind::Message(if method.is_client_streaming() {
                    let Kind::Message(request) = method.input().get_field_by_name("request").unwrap().kind() else {
                        panic!("request message")
                    };
                    request
                } else {
                    method.input()
                })
            );
            let primary = method.output();
            let response =
                if method.is_server_streaming() { primary.get_field_by_name("response").unwrap().kind() } else { Kind::Message(primary) };
            assert_eq!(managed.output().get_field_by_name("response").unwrap().kind(), response);
        }
    }

    // Open A Health Call Using Its Generated Request And Authoritative Descriptor.
    let service = pool.get_service_by_name("dokime.v1.Backend").unwrap();
    let method = service.methods().find(|method| method.name() == "RunGetHealth").unwrap();
    let opening = BackendGetHealthRunInput {
        value: Some(backend_get_health_run_input::Value::Request(GetBackendHealth {})),
    };
    let bytes = opening.encode_to_vec();
    let opening = DynamicMessage::decode(method.input(), bytes.as_slice()).unwrap();
    assert_eq!(opening.get_field_by_name("request").unwrap().as_message().unwrap().descriptor(), GetBackendHealth {}.descriptor());
    let health = BackendHealth {
        backend_id: "bench".into(),
        epoch: "boot".into(),
        readiness: BackendReadiness::Ready as i32,
        ..Default::default()
    };
    let reply = BackendGetHealthRunOutput {
        value: Some(backend_get_health_run_output::Value::Response(health.clone())),
    };
    let bytes = reply.encode_to_vec();
    let reply = DynamicMessage::decode(method.output(), bytes.as_slice()).unwrap();
    assert_eq!(reply.get_field_by_name("response").unwrap().as_message().unwrap().transcode_to::<BackendHealth>().unwrap(), health);

    // Read The Same Stream Item And Settlement Through Both Generated Envelopes.
    let log = LogFrame {
        level: LogLevel::Info as i32,
        target: "instrument".into(),
        message: "Connected".into(),
        ..Default::default()
    };
    let item = BackendWatchLogsOutput {
        value: Some(backend_watch_logs_output::Value::Item(log.clone())),
    };
    let decoded = BackendWatchLogsRunOutput::decode(item.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded.value, Some(backend_watch_logs_run_output::Value::Item(log)));
    let completed = BackendWatchLogsRunOutput {
        value: Some(backend_watch_logs_run_output::Value::Response(BackendWatchLogsCompletion {})),
    };
    let decoded = BackendWatchLogsOutput::decode(completed.encode_to_vec().as_slice()).unwrap();
    assert_eq!(decoded.value, Some(backend_watch_logs_output::Value::Response(BackendWatchLogsCompletion {})));

    // Preserve Explicit Cancellation Without Turning It Into Another Request.
    let cancelled = BackendGetHealthRunInput {
        value: Some(backend_get_health_run_input::Value::Cancel(BackendGetHealthCancellation {})),
    };
    let decoded = BackendGetHealthRunInput::decode(cancelled.encode_to_vec().as_slice()).unwrap();
    assert!(matches!(decoded.value, Some(backend_get_health_run_input::Value::Cancel(_))));
}

#[test]
fn reflected_client_story_retains_materialized_origins_and_outcomes() {
    use dokime_protocol::v1::*;
    use prost::Message;
    use prost_reflect::{DynamicMessage, ReflectMessage};

    // Reflect the Public Materialization Request Against Its Service Descriptor.
    let request = PlanningMaterializeTemplateRequest {
        selection: Some(RegisteredSelection {
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
        }),
    };
    let service = dokime_protocol::descriptor_pool().get_service_by_name("dokime.v1.Planning").unwrap();
    let method = service.methods().find(|method| method.name() == "MaterializeTemplate").unwrap();
    assert!(!method.is_client_streaming() && !method.is_server_streaming());
    assert_eq!(method.input(), request.descriptor());
    let reflected = DynamicMessage::decode(method.input(), request.encode_to_vec().as_slice()).unwrap();
    assert_eq!(reflected.transcode_to::<PlanningMaterializeTemplateRequest>().unwrap(), request);

    // Carry the Frozen Case Through Planning and the Execution Read Model.
    let origin = Origin {
        kind: Some(origin::Kind::PluginRun(PluginRun {
            plugin: "measurement".into(),
            template: "measurement.sample".into(),
            digest: vec![42; 32],
            test_case: Some(TestCase {
                streams: vec!["measurement.samples".into()],
                ..Default::default()
            }),
            started_at: Some(prost_types::Timestamp { seconds: 1_700_000_000, nanos: 17 }),
        })),
    };
    let planned = PreviewSelectionReply {
        cases: vec![PlannedCase {
            case_id: "case".into(),
            origin: Some(origin.clone()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let planned = PreviewSelectionReply::decode(planned.encode_to_vec().as_slice()).unwrap();
    let execution = ExecutionDetail {
        origin: planned.cases[0].origin.clone(),
        outcome: Some(ExecutionOutcome {
            attribution: Some(execution_outcome::Attribution::Evaluated(EvaluatedOutcome {
                status: ResultStatus::Passed as i32,
                reason: "criterion".into(),
            })),
        }),
        ..Default::default()
    };
    let execution_service = dokime_protocol::descriptor_pool().get_service_by_name("dokime.v1.Execution").unwrap();
    let method = execution_service.methods().find(|method| method.name() == "GetExecution").unwrap();
    let reflected = DynamicMessage::decode(method.output(), execution.encode_to_vec().as_slice()).unwrap();
    let decoded = reflected.transcode_to::<ExecutionDetail>().unwrap();
    assert_eq!(decoded, execution);
    assert_eq!(decoded.origin, Some(origin));
}

#[test]
fn client_transfer_routes_preserve_input_finish_cancel_and_final_response() {
    use dokime_protocol::v1::*;
    use prost::Message;
    use prost_reflect::ReflectMessage;

    // Inspect the Upload and Its Cancellable Duplex Companion.
    let service = dokime_protocol::descriptor_pool().get_service_by_name("dokime.v1.Plugins").unwrap();
    let upload = service.methods().find(|method| method.name() == "Install").unwrap();
    let run = service.methods().find(|method| method.name() == "RunInstall").unwrap();
    assert!(upload.is_client_streaming() && !upload.is_server_streaming());
    assert!(run.is_client_streaming() && run.is_server_streaming());

    // Round-Trip the Distinct Ordered Control and Data Frames.
    let frames = [
        plugins_install_run_input::Value::Request(PluginsInstallRequest {}),
        plugins_install_run_input::Value::Item(InstallPluginFrame {
            frame: Some(InstallPluginFrameFrame {
                kind: Some(install_plugin_frame_frame::Kind::Header(InstallPluginHeader { size_bytes: 3, sha256: "digest".into() })),
            }),
        }),
        plugins_install_run_input::Value::Item(InstallPluginFrame {
            frame: Some(InstallPluginFrameFrame {
                kind: Some(install_plugin_frame_frame::Kind::Chunk(InstallPluginChunk { offset: 0, data: vec![0, 1, 2] })),
            }),
        }),
        plugins_install_run_input::Value::Finish(ClientEmpty {}),
        plugins_install_run_input::Value::Cancel(ClientEmpty {}),
    ];
    for value in frames {
        let frame = PluginsInstallRunInput { value: Some(value) };
        assert_eq!(run.input(), frame.descriptor());
        assert_eq!(PluginsInstallRunInput::decode(frame.encode_to_vec().as_slice()).unwrap(), frame);
    }
    let terminal = PluginsInstallRunOutput {
        value: Some(plugins_install_run_output::Value::Response(PluginInstallation {
            install_id: "installation".into(),
            plugin: Some(Plugin {
                entry: "entry".into(),
                placement: Some(plugin::Placement::Managed(ManagedPlugin { bundle: "bundle".into() })),
                state: PluginState::Starting as i32,
                ..Default::default()
            }),
            ..Default::default()
        })),
    };
    assert_eq!(run.output(), terminal.descriptor());
    assert_eq!(PluginsInstallRunOutput::decode(terminal.encode_to_vec().as_slice()).unwrap(), terminal);
}
