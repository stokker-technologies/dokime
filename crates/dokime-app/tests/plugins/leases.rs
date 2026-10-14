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
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
// Third Party Imports
use harmos::Message;
// Local Imports
use super::*;
use dokime::core::{Identifier, TestExecutionId, Timestamp, Value, ValueType};
use dokime::execution::Signal;
use dokime::telemetry::{
    SamplingScope, TelemetryBatch, TelemetryColumn, TelemetryContract, TelemetryRole,
    TelemetrySample,
};
use dokime::transport::{ExecutionSignal, SignalReply};
use dokime_app::{InstalledPlugin, Runtime};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
struct RecordingSidecar {
    calls: Mutex<Vec<(String, Vec<u8>)>>,
}

struct CountingCollector {
    reads: Arc<AtomicUsize>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RecordingSidecar {
    fn delivered(&self) -> Vec<TelemetrySample> {
        self.calls
            .lock()
            .expect("the fixture's calls are readable")
            .iter()
            .filter(|(method, _)| method.ends_with(".signal"))
            .filter_map(|(_, params)| ExecutionSignal::decode(params.as_slice()).ok())
            .filter_map(|signal| signal.samples)
            .flat_map(|request| request.samples)
            .filter_map(|encoded| serde_json::from_str::<TelemetrySample>(&encoded).ok())
            .collect()
    }
}

impl dokime_host::HostTelemetryCollector for CountingCollector {
    fn interval(&self, stream: &str) -> Duration {
        let config = dokime_host::HostTelemetryConfig::default();
        dokime_host::HostTelemetryStream::try_from(stream)
            .map(|stream| Duration::from_millis(config.interval_for(stream).0.max(1)))
            .unwrap_or(Duration::from_secs(1))
    }

    fn collect(&self, stream: &str) -> Result<Option<TelemetryBatch>, String> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        let identifier = Identifier::new(stream).expect("a fixture stream is addressable");
        let mut sample = TelemetrySample::new(
            identifier.clone(),
            0,
            Timestamp::now_utc(),
            "counting-fixture",
        );
        sample.values.insert(
            "cpu_load_percent".to_owned(),
            dokime::core::Value::from(1.0),
        );

        Ok(Some(TelemetryBatch::new(identifier, vec![sample])))
    }
}

/* === Public Functions === */

/* === Private Functions === */
#[harmos::lanes]
enum FixtureLanes {
    #[default]
    Open,
}

macro_rules! delivery_fixture {
    ($operation:ident, $catalog:ident, $id:literal) => {
        struct $operation {
            sidecar: Arc<RecordingSidecar>,
        }
        impl harmos::operations::OperationDeclaration for $operation {
            type Request = ExecutionSignal;
            type Response = SignalReply;
            const ID: &'static str = $id;
            const VISIBILITY: harmos::operations::Visibility =
                <Signal<()> as harmos::operations::OperationDeclaration>::VISIBILITY;
            const POLICY: harmos::operations::OperationPolicy =
                <Signal<()> as harmos::operations::OperationDeclaration>::POLICY;
        }
        impl harmos::operations::WireOperation for $operation {
            const VERSION: u32 = <Signal<()> as harmos::operations::WireOperation>::VERSION;
        }
        #[harmos::catalog]
        enum $catalog {
            Deliver($operation),
        }
        impl harmos::operations::Operation for $operation {
            type Declaration = Self;
            type Resources = RecordingSidecar;
            type Endpoints = ();
            type Error = harmos::Error;
            type Context<'a> = harmos::operations::Context<'a>;
            async fn initialize<'ctx>(
                sidecar: Arc<RecordingSidecar>,
                _: Self::Context<'ctx>,
            ) -> harmos::Result<Self> {
                Ok(Self { sidecar })
            }
            async fn run<'ctx>(
                &mut self,
                _: Self::Context<'ctx>,
                request: &mut ExecutionSignal,
                _: &mut (),
            ) -> harmos::Result<harmos::operations::Response<SignalReply>> {
                self.sidecar
                    .calls
                    .lock()
                    .unwrap()
                    .push(($id.into(), request.encode_to_vec()));
                Ok(harmos::operations::Response::new(SignalReply::default()))
            }
        }
    };
}
delivery_fixture!(GatewayDelivery, GatewaySignals, "fixture.template.2.signal");
delivery_fixture!(BenchDelivery, BenchSignals, "fixture.template.0.signal");
delivery_fixture!(ProbeDelivery, ProbeSignals, "fixture.template.1.signal");

fn register_sidecars(
    handle: &dokime_app::RuntimeHandle,
    sidecars: &[(&str, &Arc<RecordingSidecar>)],
) {
    for (source, sidecar) in sidecars {
        match *source {
            "gateway" => handle
                .operations()
                .register_wire_operations::<GatewaySignals, _>(
                    *source,
                    FixtureLanes::Open,
                    Arc::clone(sidecar),
                ),
            "bench" => handle
                .operations()
                .register_wire_operations::<BenchSignals, _>(
                    *source,
                    FixtureLanes::Open,
                    Arc::clone(sidecar),
                ),
            "probe" => handle
                .operations()
                .register_wire_operations::<ProbeSignals, _>(
                    *source,
                    FixtureLanes::Open,
                    Arc::clone(sidecar),
                ),
            _ => panic!("unknown fixture owner"),
        }
        .unwrap();
    }
}

/* === Functions === */

async fn runtime(root: &std::path::Path) -> (Runtime, Arc<AtomicUsize>) {
    // The host is an installed plugin like any other, and its six streams
    // are leasable because it declared their contracts — cadence included.
    runtime_with(root, [dokime_host::installed_plugin()]).await
}

async fn runtime_with(
    root: &std::path::Path,
    artifacts: impl IntoIterator<Item = InstalledPlugin>,
) -> (Runtime, Arc<AtomicUsize>) {
    let (runtime, _) = started(config(root).with_installed_plugins(artifacts)).await;
    let reads = Arc::new(AtomicUsize::new(0));
    runtime
        .handle()
        .bind_host_telemetry_source(Arc::new(CountingCollector {
            reads: Arc::clone(&reads),
        }));

    (runtime, reads)
}

fn stream() -> Identifier {
    Identifier::new("host.system").expect("a host stream is addressable")
}

fn foreign_stream() -> Identifier {
    Identifier::new("gateway.rssi").expect("a fixture stream is addressable")
}

fn gateway_artifact() -> InstalledPlugin {
    InstalledPlugin {
        source: "gateway".to_owned(),
        version: "0.1.0".to_owned(),
        description: "Owns the gateway radio stream.".to_owned(),
        enabled: true,
        telemetry_contracts: vec![
            TelemetryContract::declared("gateway.rssi", "Gateway signal")
                .column(TelemetryColumn::declared(
                    "rssi",
                    "RSSI",
                    ValueType::Number,
                    TelemetryRole::Value,
                ))
                .every(200),
        ],
        ..Default::default()
    }
}

fn bench_artifact() -> InstalledPlugin {
    reader_artifact("bench")
}

fn probe_artifact() -> InstalledPlugin {
    reader_artifact("probe")
}

fn reader_artifact(source: &str) -> InstalledPlugin {
    InstalledPlugin {
        source: source.to_owned(),
        version: "0.1.0".to_owned(),
        description: "Runs a template that reads the gateway's stream.".to_owned(),
        enabled: true,
        ..Default::default()
    }
}

async fn stored(handle: &dokime_app::RuntimeHandle) -> usize {
    handle
        .selected_telemetry(dokime_app::domains::evidence::TelemetrySelection {
            streams: vec![foreign_stream()],
            ..Default::default()
        })
        .await
        .iter()
        .filter(|sample| sample.stream == foreign_stream())
        .count()
}

fn gateway_batch(rssi: f64) -> TelemetryBatch {
    let mut sample = TelemetrySample::new(foreign_stream(), 0, Timestamp::now_utc(), "gateway");
    sample.values.insert("rssi".to_owned(), Value::from(rssi));

    TelemetryBatch::new(foreign_stream(), vec![sample])
}

async fn admitted_execution(handle: &RuntimeHandle, index: usize) -> TestExecutionId {
    let mut suite = fixture_suite(&["passed"; 3]);
    let template = format!("fixture.template.{index}");
    suite
        .test_case_templates
        .retain(|candidate| candidate.metadata.identifier.as_str() == template);
    let id = handle
        .start_cycle(
            CycleDefinition {
                cases: materialize_cases("fixture.registration", &suite),
                suite,
                metadata: metadata("lease.owner", "Lease Owner"),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    cycle_snapshot(handle, &id).await.test_executions[0]
        .test_execution_id
        .clone()
}

fn scope_of(execution: &TestExecutionId) -> SamplingScope {
    SamplingScope {
        test_cycle_id: None,
        test_case_id: None,
        test_execution_id: Some(execution.to_string()),
    }
}

pub(crate) async fn failed_group_holds() {
    let root = tempdir().expect("a temporary root");
    let (mut runtime, _) = runtime_with(root.path(), [gateway_artifact()]).await;
    let handle = runtime.handle();
    let execution = admitted_execution(&handle, 0).await;
    let scope = dokime::core::RecordScope::default().with_test_execution(execution.clone());
    let unknown = Identifier::new("fixture.unregistered").unwrap();
    let failure = handle
        .lease_streams(scope_of(&execution), &[foreign_stream(), unknown], "bench")
        .await;
    assert!(failure.is_err());
    let facts = handle.stream_leases_for(&scope).await;
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].stream, foreign_stream());
    assert!(facts[0].closed_at.is_some());

    let retry = handle
        .lease_streams(scope_of(&execution), &[foreign_stream()], "bench")
        .await
        .unwrap();
    assert_eq!(retry.len(), 1);
    assert!(handle.stream_lease_is_active(&retry[0]).await);
    let facts = handle.stream_leases_for(&scope).await;
    assert_eq!(
        facts.iter().filter(|fact| fact.closed_at.is_none()).count(),
        1
    );
    handle.release_stream_leases(retry).await.unwrap();
    drop(handle);
    runtime.shutdown().await.expect("the runtime stops");
}

pub(crate) async fn two_executions_rows() {
    let root = tempdir().expect("a temporary root");
    let (mut runtime, _) = runtime_with(
        root.path(),
        [gateway_artifact(), bench_artifact(), probe_artifact()],
    )
    .await;
    let handle = runtime.handle();

    let owner = Arc::new(RecordingSidecar::default());
    let first_reader = Arc::new(RecordingSidecar::default());
    let second_reader = Arc::new(RecordingSidecar::default());
    register_sidecars(
        &handle,
        &[
            ("gateway", &owner),
            ("bench", &first_reader),
            ("probe", &second_reader),
        ],
    );

    let first = admitted_execution(&handle, 0).await;
    let second = admitted_execution(&handle, 1).await;
    let first_leases = handle
        .lease_streams(scope_of(&first), &[foreign_stream()], "bench")
        .await
        .expect("the first execution leases the stream");
    let second_leases = handle
        .lease_streams(scope_of(&second), &[foreign_stream()], "probe")
        .await
        .expect("the second execution leases the same stream");

    assert_eq!(first_leases.len(), 1);
    assert_eq!(second_leases.len(), 1);
    assert_ne!(
        first_leases[0], second_leases[0],
        "two holds are two leases even when they share one instance"
    );

    handle
        .publish_telemetry_batch(gateway_batch(-61.5))
        .await
        .expect("the owning sidecar publishes its stream");

    // The same rows, in both runs' processes, values and instant alike. The
    // scope is deliberately not compared: each copy is stamped with the run it
    // is evidence for, which is how a sidecar running several executions tells
    // whose it is.
    let measured = |sidecar: &RecordingSidecar| {
        sidecar
            .delivered()
            .iter()
            .map(|row| (row.timestamp.to_string(), row.values.clone()))
            .collect::<Vec<_>>()
    };
    let first_rows = measured(&first_reader);
    let second_rows = measured(&second_reader);
    assert_eq!(first_rows.len(), 1, "the first run was handed the row");
    assert_eq!(
        second_rows, first_rows,
        "both runs read the one measurement"
    );
    assert_eq!(
        first_reader.delivered()[0]
            .scope
            .as_ref()
            .and_then(|scope| scope.test_execution_id.as_ref().map(ToString::to_string)),
        Some(first.to_string()),
        "a delivered row is stamped with the run it is evidence for"
    );
    assert_eq!(
        second_reader.delivered()[0]
            .scope
            .as_ref()
            .and_then(|scope| scope.test_execution_id.as_ref().map(ToString::to_string)),
        Some(second.to_string()),
    );

    // Stored once, however many runs read it.
    assert_eq!(stored(&handle).await, 1);

    // Releasing one hold leaves the instance running for the other, and the
    // released run stops being evidence of anything new.
    handle.release_stream_leases(first_leases).await.unwrap();
    assert!(handle.stream_lease_is_active(&second_leases[0]).await);
    handle
        .publish_telemetry_batch(gateway_batch(-70.0))
        .await
        .expect("the owner keeps publishing");
    assert_eq!(
        first_reader.delivered().len(),
        1,
        "a released hold is handed nothing further"
    );
    assert_eq!(
        second_reader.delivered().len(),
        2,
        "the run still holding it keeps receiving"
    );
    assert_eq!(stored(&handle).await, 2, "each measurement is stored once");

    handle.release_stream_leases(second_leases).await.unwrap();
    drop(handle);
    runtime.shutdown().await.expect("the runtime stops");
}

pub(crate) async fn two_holds_reading() {
    let root = tempdir().expect("a temporary root");
    let (mut runtime, reads) = runtime(root.path()).await;
    let handle = runtime.handle();

    let first = handle
        .lease_streams(
            SamplingScope {
                test_cycle_id: None,
                test_case_id: None,
                test_execution_id: None,
            },
            &[stream()],
            "fixture.a",
        )
        .await
        .expect("the first execution leases the host stream");
    tokio::time::sleep(Duration::from_millis(2_500)).await;
    let alone = reads.load(Ordering::SeqCst);
    assert!(alone > 0, "the leased stream is actually being measured");

    // A second hold on the same stream joins the one session. The reading
    // rate must not change: a second instance would roughly double it.
    let second = handle
        .lease_streams(
            SamplingScope {
                test_cycle_id: None,
                test_case_id: None,
                test_execution_id: None,
            },
            &[stream()],
            "fixture.b",
        )
        .await
        .expect("the second execution leases the same stream");
    tokio::time::sleep(Duration::from_millis(3_000)).await;
    let added = reads.load(Ordering::SeqCst) - alone;
    assert!(
        added <= 6,
        "one stream at one second is about one reading a second whoever holds it: {added} readings over three seconds"
    );

    // A second stream of the same cadence adds its own reading rate and no
    // more: `host.network` declares the same one second, so the two together
    // are about two reads a second. A re-request of `host.system` would make it
    // three.
    let baseline = reads.load(Ordering::SeqCst);
    let network = handle
        .lease_streams(
            SamplingScope {
                test_cycle_id: None,
                test_case_id: None,
                test_execution_id: None,
            },
            &[Identifier::new("host.network").expect("addressable")],
            "fixture.b",
        )
        .await
        .expect("the second host stream leases");
    tokio::time::sleep(Duration::from_millis(3_000)).await;
    let both = reads.load(Ordering::SeqCst) - baseline;
    assert!(
        both <= 9,
        "two streams at one second are about two reads a second, not three: {both} reads over three seconds"
    );
    handle.release_stream_leases(network).await.unwrap();

    // The instance outlives the first release and stops on the last.
    handle.release_stream_leases(first).await.unwrap();
    assert!(handle.stream_lease_is_active(&second[0]).await);
    let before = reads.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(2_500)).await;
    assert!(
        reads.load(Ordering::SeqCst) > before,
        "the surviving hold keeps the instance running"
    );

    handle.release_stream_leases(second).await.unwrap();
    tokio::time::sleep(Duration::from_millis(4_000)).await;
    let after_stop = reads.load(Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(2_500)).await;
    assert_eq!(
        reads.load(Ordering::SeqCst),
        after_stop,
        "a stream nobody holds is not being read"
    );

    drop(handle);
    runtime.shutdown().await.expect("the runtime stops");
}

pub(crate) async fn foreign_stream_refuses() {
    let root = tempdir().expect("a temporary root");
    let (mut runtime, _) = runtime_with(root.path(), [gateway_artifact(), bench_artifact()]).await;
    let handle = runtime.handle();

    let owner = Arc::new(RecordingSidecar::default());
    let reader = Arc::new(RecordingSidecar::default());
    register_sidecars(&handle, &[("gateway", &owner), ("bench", &reader)]);

    let execution = admitted_execution(&handle, 0).await;
    let leases = handle
        .lease_streams(scope_of(&execution), &[foreign_stream()], "bench")
        .await
        .expect("a bench execution leases the gateway's stream");
    assert_eq!(leases.len(), 1);

    // What the owner measures, the host stores once and hands to the reader.
    handle
        .publish_telemetry_batch(gateway_batch(-61.5))
        .await
        .expect("the owning sidecar publishes its stream");

    let delivered = reader.delivered();
    assert_eq!(
        delivered.len(),
        1,
        "the leasing run in the other process was handed the row"
    );
    assert_eq!(delivered[0].stream, foreign_stream());
    assert_eq!(delivered[0].values.get("rssi"), Some(&Value::from(-61.5)));
    assert_eq!(
        delivered[0]
            .scope
            .as_ref()
            .and_then(|scope| scope.test_execution_id.as_ref().map(ToString::to_string)),
        Some(execution.to_string()),
        "the row is stamped with the run that leased it"
    );
    assert!(
        owner.delivered().is_empty(),
        "the owner is not handed back what it just published"
    );

    // A stream no installed plugin declares cannot be leased at all, and the
    // refusal names it.
    let refusal = handle
        .lease_streams(
            scope_of(&execution),
            &[Identifier::new("gateway.unknown").expect("addressable")],
            "bench",
        )
        .await
        .expect_err("a stream with no observer cannot be leased")
        .to_string();
    assert!(
        refusal.contains("gateway.unknown"),
        "the refusal names the stream: {refusal}"
    );

    // Stored once: the host's own copy, not one per leaseholder. A second
    // stored row would be the reader re-publishing what it was handed.
    let stored = handle
        .selected_telemetry(dokime_app::domains::evidence::TelemetrySelection {
            streams: vec![foreign_stream()],
            ..Default::default()
        })
        .await;
    assert_eq!(
        stored
            .iter()
            .filter(|sample| sample.stream == foreign_stream())
            .count(),
        1,
        "one measurement is stored once, however many runs read it"
    );

    // Releasing the hold stops the delivery, not the storage.
    handle.release_stream_leases(leases).await.unwrap();
    handle
        .publish_telemetry_batch(gateway_batch(-70.0))
        .await
        .expect("the owning sidecar keeps publishing");
    assert_eq!(
        reader.delivered().len(),
        1,
        "a released hold is handed nothing further"
    );

    drop(handle);
    runtime.shutdown().await.expect("the runtime stops");
}

/* === Tests === */
// template record demand captures startup and releases on cancellation.

pub(crate) async fn template_record_on() {
    use dokime_app::domains::evidence::RecordSelection;
    use dokime_app::transactions::CycleDefinition;

    let root = tempdir().unwrap();
    let marker = root.path().join("record-observer");
    let bundle = fixture_bundle(root.path(), "fixture-telemetry")
        .configured(
            "validation-telemetry",
            serde_json::json!({ "record_lifecycle": marker }),
        )
        .unwrap();
    let (mut runtime, handle) = started(config(root.path()).with_plugins([bundle])).await;
    let selection = || RecordSelection {
        record_types: vec![Identifier::new("validation-telemetry.started").unwrap()],
        ..Default::default()
    };

    let unexpected_start = tokio::time::timeout(Duration::from_millis(100), async {
        while !marker.with_extension("started").exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    assert!(
        unexpected_start.is_err(),
        "registering the record and observer does not create demand",
    );
    assert!(
        handle
            .selected_records(selection())
            .await
            .unwrap()
            .is_empty()
    );

    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "validation-telemetry")
        .expect("the fixture exports its record subscription template")
        .suite;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(
                    Identifier::new("record-demand.cycle").unwrap(),
                    "Record Demand Cycle",
                    None,
                ),
                cases: materialize_cases("validation-telemetry.records", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");

    let (execution_id, captured_record) = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let cycle = handle
                .cycles()
                .await
                .into_iter()
                .find(|cycle| cycle.test_cycle_id == cycle_id)
                .unwrap();
            if let Some(execution) = cycle.test_executions.first() {
                let mut captured = selection();
                captured.scope.test_execution_id = Some(execution.test_execution_id.clone());
                let records = handle.selected_records(captured).await.unwrap();
                if !records.is_empty() {
                    assert_eq!(
                        records.len(),
                        1,
                        "the observer emits only its startup event"
                    );
                    let captured = &records[0];
                    let scope = captured.scope.as_ref().expect("captured ownership");
                    let mut owner = scope.clone();
                    owner.step_id = None;
                    owner.criterion_id = None;
                    // Subscription Delivery Uses The Execution Phase At Capture Time.
                    assert!(scope.criterion_id.is_none());
                    match &scope.origin {
                        dokime::core::EvidenceOrigin::Step { attempt_id } => {
                            let snapshot = handle.watch_state().latest();
                            let current =
                                &snapshot.state().executions[&execution.test_execution_id];
                            let step_id = current
                                .active_step_attempt
                                .as_ref()
                                .filter(|attempt| &attempt.attempt_id == attempt_id)
                                .map(|attempt| &attempt.step_id)
                                .or_else(|| {
                                    current
                                        .step_attempts
                                        .iter()
                                        .find(|attempt| &attempt.attempt_id == attempt_id)
                                        .map(|attempt| &attempt.step_id)
                                })
                                .expect("captured origin names an actual execution attempt");
                            assert_eq!(scope.step_id.as_ref(), Some(step_id));
                        }
                        _ => assert!(scope.step_id.is_none()),
                    }
                    owner.origin = execution.scope.origin.clone();
                    assert_eq!(owner, execution.scope);
                    break (execution.test_execution_id.clone(), captured.clone());
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the subscription captures the observer's only startup event");
    assert!(marker.with_extension("started").exists());
    assert!(
        !marker.with_extension("stopped").exists(),
        "the active template keeps its record-only observer alive",
    );

    handle
        .stop_cycle(cycle_id.clone(), CommandContext::default())
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let cycle = handle
                .cycles()
                .await
                .into_iter()
                .find(|cycle| cycle.test_cycle_id == cycle_id)
                .unwrap();
            if marker.with_extension("stopped").exists()
                && cycle.test_executions.iter().any(|execution| {
                    execution.test_execution_id == execution_id && execution.finished_at.is_some()
                })
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("cancelling the template releases the observer's final record hold");

    let cycle = handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| cycle.test_cycle_id == cycle_id)
        .unwrap();
    let execution = cycle
        .test_executions
        .iter()
        .find(|execution| execution.test_execution_id == execution_id)
        .unwrap();
    assert_eq!(
        execution
            .records
            .subscribed_records()
            .get(&captured_record.record_id),
        captured_record.scope.as_ref(),
        "queries use the scope frozen when the record was admitted",
    );

    let mut selected = selection();
    selected.scope.test_execution_id = Some(execution_id.clone());
    assert_eq!(
        handle.selected_records(selected).await.unwrap(),
        vec![captured_record.clone()],
        "cancellation does not rewrite captured ownership",
    );
    let producer = handle.selected_records(selection()).await.unwrap();
    assert_eq!(producer.len(), 1);
    assert_eq!(producer[0].record_id, captured_record.record_id);
    assert!(
        producer[0]
            .scope
            .as_ref()
            .is_none_or(|scope| { scope.test_execution_id.is_none() && scope.step_id.is_none() }),
        "backend monitoring retains the producer scope"
    );

    let checkpoint = crate::support::native::checkpoint(&handle, &execution_id).await;
    let saved: Vec<_> = crate::support::native::records(&handle, &checkpoint)
        .await
        .into_iter()
        .filter(|record| record.record_id == captured_record.record_id)
        .collect();
    assert_eq!(saved, vec![captured_record]);

    drop(handle);
    runtime.shutdown().await.unwrap();
}
