/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A Client Installs the Packaged OBC Plugin and Runs Every Charger Procedure and the Bench

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;

// Third Party Imports
use prost_reflect::{DescriptorPool, DynamicMessage, Value as ReflectValue};
use sha2::{Digest, Sha256};
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use super::support;
use super::support::bundles::upload;
use dokime::Client;
use dokime::EvidenceRecordScope;
use dokime::EvidenceScope;
use dokime::MapValue;
use dokime::Value;
use dokime::client::DocumentScope;
use dokime::client::DocumentsGetOptions;
use dokime::client::EvidenceRecordsListOptions;
use dokime::client::ExecutionState;
use dokime::client::FrozenSchemaCache;
use dokime::client::ResultStatus;
use dokime::record_scope::Kind as RecordScopeKind;
use dokime_protocol::host::document_binding_result::Kind as DocumentBindingResultKind;
use dokime_protocol::host::document_frame_frame::Kind as DocumentFrameFrameKind;
use dokime_protocol::host::execution_outcome::Attribution as ExecutionOutcomeAttribution;
use dokime_protocol::model::value::Kind as ValueKind;
use dokime_runtime::PluginProcess;
use harmos_protocol::host::plugin_placement::Kind as PlacementKind;
use harmos_protocol::host::{PluginManaged, PluginPlacement, PluginRemote, PluginState, PluginValue};

/* === Constants === */

// The Bench Meter's Exact 64-Bit Register, Above Every Float's Integer Precision.
const REGISTER: u64 = 1_152_921_504_606_847_975;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/// Install the Packaged Bundle Through a Verified Managed Upload and Await Its Ready Entry.
async fn install(client: &Client) -> Result<(), Box<dyn Error>> {
    let archive = std::fs::read(support::obc_bundle()?)?;
    let content = upload(client, archive.clone()).await?;
    let placement = PluginPlacement {
        kind: Some(PlacementKind::Managed(PluginManaged {
            content: Some(content),
            sha256: Sha256::digest(&archive).to_vec(),
        })),
    };
    let entry = client.plugins.create("obc", placement, PluginValue { enabled: true }).await?.entry.ok_or("Install OBC: Missing Entry")?;
    let entry = support::plugin_state(&client.plugins, entry, PluginState::Ready).await?;
    assert_eq!(entry.state, PluginState::Ready as i32);
    Ok(())
}

/// Run One Admitted Case, Answer Its Prompt When One Is Asked, and Require a Sealed Pass.
async fn run(
    client: &Client, suite: &str, template: &str, parameters: HashMap<String, Vec<Value>>, mut answer: Option<(&str, MapValue)>,
) -> Result<(String, String), Box<dyn Error>> {
    let cases = support::materialize(client, suite, template, parameters).await?;
    assert_eq!(cases.len(), 1);
    let execution = support::admit(client, cases).await?.remove(0);
    let mut progress = client.executions.progress.watch(execution.clone()).await?;
    while let Some(frame) = progress.next().await? {
        if frame.phase == "prompt"
            && let Some((action, values)) = answer.take()
        {
            let pending = client.prompts.list(execution.clone(), true).await?;
            let prompt = pending.prompts[0].prompt.as_ref().ok_or("Answer the Prompt: Missing Question")?.prompt_id.clone();
            client.prompts.answers.create(prompt, action, values, true).await?;
        }
    }
    assert!(answer.is_none(), "{template}: The Prompt Was Never Asked");
    let detail = client.executions.get(execution.clone()).await?.execution.ok_or("Read Execution: Missing Detail")?;
    let summary = detail.summary.as_ref().ok_or("Read Execution: Missing Summary")?;
    assert_eq!(summary.state, ExecutionState::Sealed as i32);
    assert!(
        matches!(detail.outcome.as_ref().and_then(|outcome| outcome.attribution.as_ref()),
            Some(ExecutionOutcomeAttribution::Evaluated(outcome)) if outcome.status == ResultStatus::Passed as i32),
        "{template}: {:?}",
        detail.outcome
    );
    Ok((execution, detail.cycle_id.clone()))
}

/// An Operator Inspection That Passes, With a Note.
fn inspection() -> MapValue {
    MapValue {
        values: HashMap::from([
            ("inspection_passed".into(), Value { kind: Some(ValueKind::Boolean(true)) }),
            (
                "operator_note".into(),
                Value {
                    kind: Some(ValueKind::Text("Indicator Steady".into())),
                },
            ),
        ]),
    }
}

/// Decode Every Record of One Execution Through Its Frozen Descriptors, by Record Type.
async fn records(
    client: &Client, execution: &str, messages: &[(&str, &str)],
) -> Result<BTreeMap<String, Vec<DynamicMessage>>, Box<dyn Error>> {
    let detail = client.executions.get(execution.to_owned()).await?.execution.ok_or("Read Execution: Missing Detail")?;
    let pool = DescriptorPool::from_file_descriptor_set((*FrozenSchemaCache::default().get(client, &detail).await?).clone())?;
    let scope = EvidenceRecordScope {
        kind: Some(RecordScopeKind::Execution(execution.to_owned())),
    };
    let options = EvidenceRecordsListOptions {
        versions: Some(Vec::new()),
        between: Some(Default::default()),
        step: Some(String::new()),
        selection: Some(Default::default()),
        ..Default::default()
    };
    let listed = client.evidence.records.list_with(scope, String::new(), options).await?;
    let mut decoded = BTreeMap::<String, Vec<DynamicMessage>>::new();
    for record in listed.records {
        let (_, name) = messages.iter().find(|(kind, _)| *kind == record.record_type).ok_or("Decode Record: Unexpected Record Type")?;
        let message = pool.get_message_by_name(name).ok_or("Decode Record: Missing Frozen Message")?;
        decoded.entry(record.record_type.clone()).or_default().push(DynamicMessage::decode(message, record.payload.as_slice())?);
    }
    Ok(decoded)
}

/// Read a Value as an Exact Decimal Integer, Refusing Floating Results.
fn exact(value: &Value) -> String {
    match value.kind.as_ref() {
        Some(ValueKind::UnsignedInteger(value)) => value.to_string(),
        Some(ValueKind::Integer(value)) => value.to_string(),
        Some(ValueKind::Exact(value)) if value.denominator == "1" && !value.negative => value.numerator.clone(),
        other => panic!("Expected an Exact Integer, Found {other:?}"),
    }
}

/* === Tests === */

#[test]
fn client_runs_packaged_obc_plugin() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Install the Packaged Bundle Through a Verified Managed Upload
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            install(&client).await?;
            let suites = client.suites.list(String::new()).await?;
            let suites = suites.values.iter().map(|suite| suite.metadata.as_ref().unwrap().identifier.clone()).collect::<BTreeSet<_>>();
            assert!(
                ["obc.checks", "obc.charger.startup", "obc.charger.steady", "obc.charger.protection", "obc.charger.recovery", "obc.bench"]
                    .iter()
                    .all(|suite| suites.contains(*suite))
            );

            // Run the Start-Up Sequence and Accept the Operator Inspection
            let answer = Some(("submit", inspection()));
            let (startup, _) = run(&client, "obc.charger.startup", "obc.charger.startup.sequence", HashMap::new(), answer).await?;
            let messages = [
                ("obc.charger.startup.initialized", "obc.Initialized"),
                ("obc.charger.startup.started", "obc.Started"),
                ("obc.charger.startup.checkpoint", "obc.Checkpoint"),
                ("obc.charger.startup.completed", "obc.Completed"),
            ];
            let captured = records(&client, &startup, &messages).await?;
            for record in ["initialized", "started", "checkpoint", "completed"] {
                assert_eq!(captured[&format!("obc.charger.startup.{record}")].len(), 1, "{record}");
            }
            let checkpoint = &captured["obc.charger.startup.checkpoint"][0];
            assert_eq!(checkpoint.get_field_by_name("sample_index").unwrap().as_ref(), &ReflectValue::U64(10));
            let completed = &captured["obc.charger.startup.completed"][0];
            assert_eq!(completed.get_field_by_name("sample_count").unwrap().as_ref(), &ReflectValue::U64(20));
            assert_eq!(completed.get_field_by_name("accepted").unwrap().as_ref(), &ReflectValue::Bool(true));
            assert_eq!(completed.get_field_by_name("operator_note").unwrap().as_ref(), &ReflectValue::String("Indicator Steady".into()));
            let scope = EvidenceScope {
                execution_id: Some(startup.clone()),
                ..Default::default()
            };
            let inrush = client.evidence.statistics(scope, "obc.charger.startup.samples", "current_rms_a").await?.statistics.unwrap();
            assert_eq!(inrush.count, 20);
            assert!(matches!(inrush.first.and_then(|value| value.kind), Some(ValueKind::Number(first)) if (first - 3.45).abs() < 1e-9));

            // Run the Module Array Bench and Read Its Exact Register Statistics
            let answer = Some(("yes", MapValue::default()));
            let (bench, cycle) = run(&client, "obc.bench", "obc.bench.module_array", HashMap::new(), answer).await?;
            let captured = records(&client, &bench, &[("obc.bench.confirmation", "obc.Confirmation")]).await?;
            let confirmation = &captured["obc.bench.confirmation"];
            assert_eq!(confirmation.len(), 1);
            assert_eq!(confirmation[0].get_field_by_name("accepted").unwrap().as_ref(), &ReflectValue::Bool(true));
            let scope = EvidenceScope {
                execution_id: Some(bench.clone()),
                ..Default::default()
            };
            let register = client.evidence.statistics(scope, "obc.bench.meter", "energy_register").await?.statistics.unwrap();
            assert_eq!(register.count, 250);
            for value in [&register.min, &register.max, &register.first, &register.last, &register.median] {
                assert_eq!(exact(value.as_ref().unwrap()), REGISTER.to_string());
            }

            // Read the Bench Document: One Hundred Plots and Eleven Exact Tiles at One Cut
            let scope = DocumentScope { cycle_id: cycle, execution_id: bench };
            let mut options = DocumentsGetOptions {
                max_frames: Some(32),
                ..Default::default()
            };
            let mut plots = BTreeSet::new();
            let mut tiles = BTreeMap::new();
            loop {
                let page = client.documents.get_with(scope.clone(), "obc.bench.module_array.report", options.clone()).await?;
                for frame in page.frames.into_iter().filter_map(|frame| frame.frame.and_then(|frame| frame.kind)) {
                    match frame {
                        DocumentFrameFrameKind::Binding(binding) => {
                            if let Some(DocumentBindingResultKind::Plot(_)) =
                                binding.result.as_ref().and_then(|result| result.kind.as_ref())
                            {
                                plots.insert(binding.source_id);
                            }
                        }
                        DocumentFrameFrameKind::SummaryTile(tile) => {
                            assert!(tile.unavailable.is_none(), "{:?}", tile.unavailable);
                            tiles.insert(tile.metadata.unwrap().identifier, tile.value.unwrap());
                        }
                        _ => {}
                    }
                }
                if page.next_page_token.is_empty() {
                    break;
                }
                options.page_token = Some(page.next_page_token);
                options.at = page.resolved_at;
            }
            assert_eq!(plots.len(), 100);
            assert_eq!(exact(&tiles["count"]), "250");
            assert_eq!(exact(&tiles["sum"]), (u128::from(REGISTER) * 250).to_string());
            for name in ["mean", "median", "min", "max", "first", "last", "p05", "p95", "p99"] {
                assert_eq!(exact(&tiles[name]), REGISTER.to_string(), "{name}");
            }

            // The Pulse Saves Its Requested Samples Between Its Run Records
            let (pulse, _) = run(&client, "obc.checks", "obc.checks.pulse", HashMap::new(), None).await?;
            let messages = [("obc.run.started", "obc.Started"), ("obc.run.completed", "obc.RunCompleted")];
            let captured = records(&client, &pulse, &messages).await?;
            assert_eq!(captured["obc.run.completed"][0].get_field_by_name("samples").unwrap().as_ref(), &ReflectValue::U64(8));

            // One Steady-State Case of the Sweep Holds for Its Samples and Records Both Supply Events
            let parameters = HashMap::from([
                ("ac_voltage_v".into(), vec![Value { kind: Some(ValueKind::Number(253.0)) }]),
                ("load_ohm".into(), vec![Value { kind: Some(ValueKind::Number(80.0)) }]),
            ]);
            let answer = Some(("submit", inspection()));
            let (steady, _) = run(&client, "obc.charger.steady", "obc.charger.steady.measurement", parameters, answer).await?;
            let messages = [
                ("obc.charger.steady.initialized", "obc.Initialized"),
                ("obc.charger.steady.started", "obc.Started"),
                ("obc.charger.steady.hold", "obc.HoldEvent"),
                ("obc.charger.steady.completed", "obc.Completed"),
            ];
            let captured = records(&client, &steady, &messages).await?;
            let events = captured["obc.charger.steady.hold"]
                .iter()
                .map(|event| event.get_field_by_name("event").unwrap().as_str().unwrap().to_owned())
                .collect::<Vec<_>>();
            assert_eq!(events, ["hold_started", "hold_reached"]);
            let completed = &captured["obc.charger.steady.completed"][0];
            assert_eq!(completed.get_field_by_name("sample_count").unwrap().as_ref(), &ReflectValue::U64(50));

            // Protection Trips on the Third Load Step and Refuses a Latched Restart
            let (protection, _) =
                run(&client, "obc.charger.protection", "obc.charger.protection.overcurrent", HashMap::new(), None).await?;
            let messages = [
                ("obc.charger.protection.initialized", "obc.Initialized"),
                ("obc.charger.protection.started", "obc.Started"),
                ("obc.charger.protection.faulted", "obc.Faulted"),
                ("obc.charger.protection.stopped", "obc.Stopped"),
            ];
            let captured = records(&client, &protection, &messages).await?;
            let stopped = &captured["obc.charger.protection.stopped"][0];
            assert_eq!(stopped.get_field_by_name("tripped_at_step").unwrap().as_ref(), &ReflectValue::U64(3));
            let faulted = &captured["obc.charger.protection.faulted"][0];
            assert_eq!(faulted.get_field_by_name("fault").unwrap().as_str(), Some("overcurrent"));

            // Recovery Contains and Recovers Every Outage of the Schedule
            let (recovery, _) = run(&client, "obc.charger.recovery", "obc.charger.recovery.power", HashMap::new(), None).await?;
            let messages = [
                ("obc.charger.recovery.initialized", "obc.Initialized"),
                ("obc.charger.recovery.started", "obc.Started"),
                ("obc.charger.recovery.faulted", "obc.Faulted"),
                ("obc.charger.recovery.cleared", "obc.Cleared"),
                ("obc.charger.recovery.stopped", "obc.Stopped"),
            ];
            let captured = records(&client, &recovery, &messages).await?;
            assert_eq!(captured["obc.charger.recovery.faulted"].len(), 3);
            assert_eq!(captured["obc.charger.recovery.cleared"].len(), 3);
            assert_eq!(captured["obc.charger.recovery.started"].len(), 4);
            let stopped = &captured["obc.charger.recovery.stopped"][0];
            assert_eq!(stopped.get_field_by_name("cycles").unwrap().as_ref(), &ReflectValue::U64(3));
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn real_clock_executions_end_passed() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Install the Packaged OBC Bundle and the Remote Thermal Plugin on a Real-Clock Host
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let mut thermal = PluginProcess::start(&support::thermal_fixture()?, true, &support::settings().placement).await?;
            let process = HostProcess::start_with_clock(root.path(), &credentials.environment, false).await?;
            let client = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, support::client_settings())?;
            install(&client).await?;
            let placement = PluginPlacement {
                kind: Some(PlacementKind::Remote(PluginRemote { url: thermal.endpoint().into() })),
            };
            let entry = client
                .plugins
                .create("thermal", placement, PluginValue { enabled: true })
                .await?
                .entry
                .ok_or("Add Thermal: Missing Entry")?;
            support::plugin_state(&client.plugins, entry, PluginState::Ready).await?;

            // Every Run Seals Passed; Counts and Durations Belong to the Controlled-Clock Stories
            let limit = HashMap::from([("limit".into(), vec![Value { kind: Some(ValueKind::Number(85.0)) }])]);
            run(&client, "thermal", "thermal.cycle", limit, Some(("yes", MapValue::default()))).await?;
            run(&client, "obc.charger.startup", "obc.charger.startup.sequence", HashMap::new(), Some(("submit", inspection()))).await?;
            run(&client, "obc.bench", "obc.bench.module_array", HashMap::new(), Some(("yes", MapValue::default()))).await?;
            run(&client, "obc.checks", "obc.checks.pulse", HashMap::new(), None).await?;
            let parameters = HashMap::from([
                ("ac_voltage_v".into(), vec![Value { kind: Some(ValueKind::Number(253.0)) }]),
                ("load_ohm".into(), vec![Value { kind: Some(ValueKind::Number(80.0)) }]),
            ]);
            run(&client, "obc.charger.steady", "obc.charger.steady.measurement", parameters, Some(("submit", inspection()))).await?;
            run(&client, "obc.charger.protection", "obc.charger.protection.overcurrent", HashMap::new(), None).await?;
            run(&client, "obc.charger.recovery", "obc.charger.recovery.power", HashMap::new(), None).await?;
            process.close().await?;
            thermal.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
