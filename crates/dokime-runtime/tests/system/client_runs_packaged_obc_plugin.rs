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
use std::fs;

// Third Party Imports
use harmos_protocol::host::plugin_placement;
use harmos_protocol::host::{PluginManaged, PluginPlacement, PluginRemote, PluginState, PluginValue};
use prost_reflect::{self, DescriptorPool, DynamicMessage};
use sha2::{Digest, Sha256};
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use super::support;
use super::support::bundles::upload;
use dokime::Client;
use dokime::EvidenceScope;
use dokime::MapValue;
use dokime::RecordScope;
use dokime::Value;
use dokime::client::DocumentScope;
use dokime::client::DocumentsGetOptions;
use dokime::client::EvidenceRecordsListOptions;
use dokime::client::EvidenceStatisticsOptions;
use dokime::client::ExecutionState;
use dokime::client::FrozenSchemaCache;
use dokime::client::ResultStatus;
use dokime::record_scope;
use dokime_protocol::host::document_binding_result;
use dokime_protocol::host::document_frame_frame;
use dokime_protocol::host::execution_outcome;
use dokime_protocol::model::value;
use dokime_runtime::PluginProcess;

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
    let archive = fs::read(support::obc_bundle()?)?;
    let content = upload(client, archive.clone()).await?;
    let placement = PluginPlacement {
        kind: Some(plugin_placement::Kind::Managed(PluginManaged {
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
            Some(execution_outcome::Attribution::Evaluated(outcome)) if outcome.status == ResultStatus::Passed as i32),
        "{template}: {:?}",
        detail.outcome
    );
    Ok((execution, detail.cycle_id.clone()))
}

/// An Operator Inspection That Passes, With a Note.
fn inspection() -> MapValue {
    MapValue {
        values: HashMap::from([
            ("inspection_passed".into(), Value { kind: Some(value::Kind::Boolean(true)) }),
            (
                "operator_note".into(),
                Value {
                    kind: Some(value::Kind::Text("Indicator Steady".into())),
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
    let scope = RecordScope {
        kind: Some(record_scope::Kind::Execution(execution.to_owned())),
    };
    let options = EvidenceRecordsListOptions {
        scope: Some(scope),
        versions: Some(Vec::new()),
        between: Some(Default::default()),
        step: Some(String::new()),
        selection: Some(Default::default()),
        ..Default::default()
    };
    let listed = client.evidence.records.list_with(String::new(), options).await?;
    let mut decoded = BTreeMap::<String, Vec<DynamicMessage>>::new();
    for record in listed.records {
        let (_, name) = messages
            .iter()
            .find(|(kind, _)| *kind == record.record_type)
            .ok_or_else(|| format!("Decode {}: Declare Its Frozen Message in This Story", record.record_type))?;
        let message = pool.get_message_by_name(name).ok_or("Decode Record: Missing Frozen Message")?;
        decoded.entry(record.record_type.clone()).or_default().push(DynamicMessage::decode(message, record.payload.as_slice())?);
    }
    Ok(decoded)
}

/// Read a Value as an Exact Decimal Integer, Refusing Floating Results.
fn exact(value: &Value) -> String {
    match value.kind.as_ref() {
        Some(value::Kind::UnsignedInteger(value)) => value.to_string(),
        Some(value::Kind::Integer(value)) => value.to_string(),
        Some(value::Kind::Exact(value)) if value.denominator == "1" && !value.negative => value.numerator.clone(),
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
            assert!(["obc.steady_state", "obc.transient", "obc.benchmark"].iter().all(|suite| suites.contains(*suite)));

            // Run the Start-Up Sequence and Accept the Operator Inspection
            let answer = Some(("submit", inspection()));
            let (startup, _) = run(&client, "obc.transient", "obc.transient.startup", HashMap::new(), answer).await?;
            let messages = [
                ("obc.transient.startup.initialized", "obc.Initialized"),
                ("obc.transient.startup.started", "obc.Started"),
                ("obc.transient.startup.checkpoint", "obc.Checkpoint"),
                ("obc.transient.startup.completed", "obc.Completed"),
            ];
            let captured = records(&client, &startup, &messages).await?;
            for record in ["initialized", "started", "checkpoint", "completed"] {
                let identifier = format!("obc.transient.startup.{record}");
                assert_eq!(captured[&identifier].len(), 1, "{record}");
            }
            let checkpoint = &captured["obc.transient.startup.checkpoint"][0];
            assert_eq!(checkpoint.get_field_by_name("sample_index").unwrap().as_ref(), &prost_reflect::Value::U64(10));
            let completed = &captured["obc.transient.startup.completed"][0];
            assert_eq!(completed.get_field_by_name("sample_count").unwrap().as_ref(), &prost_reflect::Value::U64(20));
            assert_eq!(completed.get_field_by_name("accepted").unwrap().as_ref(), &prost_reflect::Value::Bool(true));
            assert_eq!(
                completed.get_field_by_name("operator_note").unwrap().as_ref(),
                &prost_reflect::Value::String("Indicator Steady".into())
            );
            let scope = EvidenceScope {
                execution_id: Some(startup.clone()),
                ..Default::default()
            };
            let inrush = client
                .evidence
                .statistics_with("obc.measurement", "current_rms_a", EvidenceStatisticsOptions { scope: Some(scope), ..Default::default() })
                .await?
                .statistics
                .unwrap();
            assert_eq!(inrush.count, 20);
            assert!(matches!(inrush.first.and_then(|value| value.kind), Some(value::Kind::Number(first)) if (first - 3.45).abs() < 1e-9));

            // Run the Exact Values Workload and Read Its Register Statistics
            let answer = Some(("yes", MapValue::default()));
            let (bench, cycle) = run(&client, "obc.benchmark", "obc.benchmark.exact_values", HashMap::new(), answer).await?;
            let captured = records(&client, &bench, &[("obc.benchmark.exact_values.confirmation", "obc.Confirmation")]).await?;
            let confirmation = &captured["obc.benchmark.exact_values.confirmation"];
            assert_eq!(confirmation.len(), 1);
            assert_eq!(confirmation[0].get_field_by_name("accepted").unwrap().as_ref(), &prost_reflect::Value::Bool(true));
            let scope = EvidenceScope {
                execution_id: Some(bench.clone()),
                ..Default::default()
            };
            let register = client
                .evidence
                .statistics_with(
                    "obc.benchmark.exact_values.meter",
                    "energy_register",
                    EvidenceStatisticsOptions { scope: Some(scope), ..Default::default() },
                )
                .await?
                .statistics
                .unwrap();
            assert_eq!(register.count, 250);
            for value in [&register.min, &register.max, &register.first, &register.last] {
                assert_eq!(exact(value.as_ref().unwrap()), REGISTER.to_string());
            }

            // Read the Exact Values Document and Its Eleven Tiles at One Cut
            let scope = DocumentScope { cycle_id: cycle, execution_id: bench };
            let mut options = DocumentsGetOptions {
                max_frames: Some(32),
                ..Default::default()
            };
            let mut plots = BTreeSet::new();
            let mut tiles = BTreeMap::new();
            loop {
                let page = client.documents.get_with(scope.clone(), "obc.benchmark.exact_values.report", options.clone()).await?;
                for frame in page.frames.into_iter().filter_map(|frame| frame.frame.and_then(|frame| frame.kind)) {
                    match frame {
                        document_frame_frame::Kind::Binding(binding) => {
                            if let Some(document_binding_result::Kind::Plot(_)) =
                                binding.result.as_ref().and_then(|result| result.kind.as_ref())
                            {
                                plots.insert(binding.source_id);
                            }
                        }
                        document_frame_frame::Kind::SummaryTile(tile) => {
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
            assert!(plots.is_empty());
            assert_eq!(exact(&tiles["count"]), "250");
            assert_eq!(exact(&tiles["sum"]), (u128::from(REGISTER) * 250).to_string());
            for name in ["mean", "min", "max", "first", "last"] {
                assert_eq!(exact(&tiles[name]), REGISTER.to_string(), "{name}");
            }

            // A Single-Phase Case Captures Settling and the Accepted Observation Window
            let parameters = HashMap::from([
                ("ac_voltage_v".into(), vec![Value { kind: Some(value::Kind::Number(253.0)) }]),
                ("load_ohm".into(), vec![Value { kind: Some(value::Kind::Number(80.0)) }]),
            ]);
            let answer = Some(("submit", inspection()));
            let (steady, _) = run(&client, "obc.steady_state", "obc.steady_state.single_phase", parameters, answer).await?;
            let messages = [
                ("obc.steady_state.single_phase.initialized", "obc.Initialized"),
                ("obc.steady_state.single_phase.started", "obc.Started"),
                ("obc.steady_state.single_phase.hold", "obc.HoldEvent"),
                ("obc.steady_state.single_phase.completed", "obc.Completed"),
            ];
            let captured = records(&client, &steady, &messages).await?;
            let events = captured["obc.steady_state.single_phase.hold"]
                .iter()
                .map(|record| record.get_field_by_name("event").unwrap().as_str().unwrap().to_owned())
                .collect::<Vec<_>>();
            assert_eq!(events, ["hold_started", "hold_reached"]);
            let completed = &captured["obc.steady_state.single_phase.completed"][0];
            assert_eq!(completed.get_field_by_name("sample_count").unwrap().as_ref(), &prost_reflect::Value::U64(150));

            // Protection Trips on the Third Load Step and Refuses a Latched Restart
            let (protection, _) = run(&client, "obc.transient", "obc.transient.overcurrent", HashMap::new(), None).await?;
            let messages = [
                ("obc.transient.overcurrent.initialized", "obc.Initialized"),
                ("obc.transient.overcurrent.started", "obc.Started"),
                ("obc.transient.overcurrent.faulted", "obc.Faulted"),
                ("obc.transient.overcurrent.stopped", "obc.Stopped"),
            ];
            let captured = records(&client, &protection, &messages).await?;
            let stopped = &captured["obc.transient.overcurrent.stopped"][0];
            assert_eq!(stopped.get_field_by_name("tripped_at_step").unwrap().as_ref(), &prost_reflect::Value::U64(3));
            let faulted = &captured["obc.transient.overcurrent.faulted"][0];
            assert_eq!(faulted.get_field_by_name("fault").unwrap().as_str(), Some("overcurrent"));

            // Overvoltage Has Its Own Protection Ladder and Restart Refusal
            let (voltage, _) = run(&client, "obc.transient", "obc.transient.overvoltage", HashMap::new(), None).await?;
            let captured = records(
                &client,
                &voltage,
                &[
                    ("obc.transient.overvoltage.initialized", "obc.Initialized"),
                    ("obc.transient.overvoltage.started", "obc.Started"),
                    ("obc.transient.overvoltage.faulted", "obc.Faulted"),
                    ("obc.transient.overvoltage.stopped", "obc.Stopped"),
                ],
            )
            .await?;
            assert_eq!(captured["obc.transient.overvoltage.faulted"][0].get_field_by_name("fault").unwrap().as_str(), Some("overvoltage"));
            assert_eq!(
                captured["obc.transient.overvoltage.stopped"][0].get_field_by_name("tripped_at_step").unwrap().as_ref(),
                &prost_reflect::Value::U64(3)
            );
            // Exercise Each Independent Benchmark and the Balanced Three-Phase Template
            run(&client, "obc.steady_state", "obc.steady_state.three_phase", HashMap::new(), Some(("submit", inspection()))).await?;
            run(&client, "obc.benchmark", "obc.benchmark.streams", HashMap::new(), None).await?;
            run(&client, "obc.benchmark", "obc.benchmark.observers", HashMap::new(), None).await?;
            run(&client, "obc.benchmark", "obc.benchmark.plots", HashMap::new(), Some(("yes", MapValue::default()))).await?;
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
                kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: thermal.endpoint().into() })),
            };
            let entry = client
                .plugins
                .create("thermal", placement, PluginValue { enabled: true })
                .await?
                .entry
                .ok_or("Add Thermal: Missing Entry")?;
            support::plugin_state(&client.plugins, entry, PluginState::Ready).await?;

            // Every Run Seals Passed; Counts and Durations Belong to the Controlled-Clock Stories
            let limit = HashMap::from([("limit".into(), vec![Value { kind: Some(value::Kind::Number(85.0)) }])]);
            run(&client, "thermal", "thermal.cycle", limit, Some(("0", MapValue::default()))).await?;
            run(&client, "obc.transient", "obc.transient.startup", HashMap::new(), Some(("submit", inspection()))).await?;
            run(&client, "obc.benchmark", "obc.benchmark.exact_values", HashMap::new(), Some(("yes", MapValue::default()))).await?;
            let parameters = HashMap::from([
                ("ac_voltage_v".into(), vec![Value { kind: Some(value::Kind::Number(253.0)) }]),
                ("load_ohm".into(), vec![Value { kind: Some(value::Kind::Number(80.0)) }]),
            ]);
            run(&client, "obc.steady_state", "obc.steady_state.single_phase", parameters, Some(("submit", inspection()))).await?;
            run(&client, "obc.transient", "obc.transient.overcurrent", HashMap::new(), None).await?;
            run(&client, "obc.transient", "obc.transient.overvoltage", HashMap::new(), None).await?;
            run(&client, "obc.steady_state", "obc.steady_state.three_phase", HashMap::new(), Some(("submit", inspection()))).await?;
            process.close().await?;
            thermal.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
