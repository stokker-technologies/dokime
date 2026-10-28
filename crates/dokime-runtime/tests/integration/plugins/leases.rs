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
use std::sync::{Arc, Mutex};
use std::time::Duration;
// Third Party Imports
use harmos::Message;
// Local Imports
use super::*;
use dokime_domain::model;
use dokime_runtime::execution::Signal;
use dokime_runtime::telemetry::{SamplingScope, TelemetryBatch, TelemetryColumn, TelemetryContract, TelemetryRole, TelemetrySample};
use dokime_runtime::transport::{ExecutionSignal, SignalReply};
use dokime_runtime::{Identifier, TestExecutionId, Value, ValueType};
use dokime_runtime::{InstalledPlugin, Runtime};

/* === Constants === */
// One attribute line: rustfmt re-indents a continued attribute inside a macro body on every pass.
const DELIVERY_POLICY: harmos::OperationPolicy = <Signal<()> as harmos::OperationDeclaration>::POLICY;

/* === Macros === */

/* === Type Aliases === */
type Reads = Arc<tokio::sync::watch::Sender<usize>>;

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
struct GatewayRow {
    rssi: f64,
}

#[derive(Default)]
struct RecordingSidecar {
    calls: Mutex<Vec<(String, Vec<u8>)>>,
}

struct CountingCollector {
    reads: Reads,
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
        dokime_host::host_telemetry_contracts()
            .into_iter()
            .find(|contract| contract.stream.as_str() == stream)
            .and_then(|contract| contract.interval)
            .map(|interval| Duration::from_millis(interval.0))
            .unwrap_or(Duration::from_secs(1))
    }

    fn collect(&self, stream: &str) -> Result<Option<TelemetryBatch>, String> {
        self.reads.send_modify(|reads| *reads += 1);
        let identifier = Identifier::new(stream).expect("a fixture stream is addressable");
        let mut sample = TelemetrySample::new(
            dokime_runtime::model::identity::new_id(),
            identifier.clone(),
            0,
            dokime_runtime::model::time::now_utc(),
            "counting-fixture",
        );
        let host_stream = dokime_host::HostTelemetryStream::try_from(stream).map_err(|error| error.to_string())?;
        // Each Stream Carries Only Its Declared Row Fields
        if host_stream == dokime_host::HostTelemetryStream::System {
            sample.values.insert("cpu_load_percent".to_owned(), dokime_runtime::Value::from(1.0));
        }
        sample.storage_payload = Some(
            model::telemetry::host::prepare_host_telemetry_payload(host_stream, &sample.values)
                .map_err(|error| error.to_string())?
                .into_bytes(),
        );
        Ok(Some(TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier, vec![sample])))
    }
}

/* === Public Functions === */

/* === Private Functions === */
macro_rules! delivery_fixture {
    ($operation:ident, $id:literal) => {
        #[harmos::operation(id = $id, response = SignalReply, policy = DELIVERY_POLICY, visibility = Host, resources = [RecordingSidecar])]
        #[derive(Clone, PartialEq)]
        struct $operation {
            signal: ExecutionSignal,
        }
        impl harmos::Operation for $operation {
            type Error = std::convert::Infallible;

            async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<SignalReply>, Self::Error> {
                ctx.resource::<RecordingSidecar>().calls.lock().unwrap().push(($id.into(), self.signal.encode()));
                Ok(harmos::Response::new(SignalReply::default()))
            }
        }
    };
}
delivery_fixture!(GatewayDelivery, "fixture.template.t2.signal");
delivery_fixture!(BenchDelivery, "fixture.template.t0.signal");
delivery_fixture!(ProbeDelivery, "fixture.template.t1.signal");

async fn register_sidecars(handle: &dokime_runtime::RuntimeHandle, sidecars: &[(&str, &Arc<RecordingSidecar>)]) {
    use harmos::RegistrationExt;
    for (source, sidecar) in sidecars {
        let mut dependencies = harmos::Dependencies::new();
        dependencies.insert(Arc::clone(sidecar)).unwrap();
        let batch = harmos::RegistrationBatch::new(dependencies);
        let batch = match *source {
            "gateway" => batch.operation::<GatewayDelivery>(),
            "bench" => batch.operation::<BenchDelivery>(),
            "probe" => batch.operation::<ProbeDelivery>(),
            _ => panic!("unknown fixture owner"),
        };
        handle.operations().register(batch.lane(dokime_runtime::app::Lanes::Open)).await.unwrap();
    }
}

/* === Functions === */

async fn runtime(root: &std::path::Path) -> (Runtime, Reads) {
    // The host is an installed plugin like any other, and its six streams
    // are leasable because it declared their contracts — cadence included.
    runtime_with(root, [dokime_host::installed_plugin()]).await
}

async fn runtime_with(root: &std::path::Path, artifacts: impl IntoIterator<Item = InstalledPlugin>) -> (Runtime, Reads) {
    let (runtime, _) = started(config(root).with_installed_plugins(artifacts)).await;
    let reads = Reads::new(tokio::sync::watch::Sender::new(0));
    runtime.handle().bind_host_telemetry_source(Arc::new(CountingCollector { reads: Arc::clone(&reads) }));

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
                .typed::<GatewayRow>()
                .column(TelemetryColumn::declared("rssi", "RSSI", ValueType::Number, TelemetryRole::Value))
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

async fn stored(handle: &dokime_runtime::RuntimeHandle) -> usize {
    handle
        .selected_telemetry(dokime_runtime::domains::evidence::TelemetrySelection {
            streams: vec![foreign_stream().into()],
            ..Default::default()
        })
        .await
        .unwrap()
        .iter()
        .filter(|sample| sample.stream == foreign_stream())
        .count()
}

fn gateway_batch(rssi: f64) -> TelemetryBatch {
    let mut sample = TelemetrySample::new(
        dokime_runtime::model::identity::new_id(),
        foreign_stream(),
        0,
        dokime_runtime::model::time::now_utc(),
        "gateway",
    );
    sample.values.insert("rssi".to_owned(), Value::from(rssi));

    TelemetryBatch::new(
        dokime_runtime::model::identity::new_id(),
        foreign_stream(),
        vec![sample.with_storage_payload(harmos::Message::encode(&GatewayRow { rssi }))],
    )
}

async fn admitted_execution(handle: &RuntimeHandle, index: usize) -> TestExecutionId {
    let mut suite = fixture_suite(&["passed"; 3]);
    let template = format!("fixture.template.t{index}");
    suite.test_case_templates.retain(|candidate| candidate.metadata.identifier.as_str() == template);
    let suite_id = format!("fixture.registration.{index}");
    handle
        .set_suites(
            &format!("fixture.{index}"),
            dokime_runtime::definition::SuiteSet::new([
                dokime_runtime::domains::TestSuiteDefinition::new(&suite_id, suite.clone()).unwrap()
            ]),
        )
        .await
        .unwrap();
    let id = start_cases(handle, metadata("lease.owner", "Lease Owner"), materialize_cases(&suite_id, &suite)).await;
    cycle_snapshot(handle, &id).await.test_executions[0].test_execution_id.clone()
}

fn scope_of(execution: &TestExecutionId) -> SamplingScope {
    SamplingScope {
        test_cycle_id: None,
        test_case_id: None,
        test_execution_id: Some(execution.to_string()),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn failed_group_holds() {
    let root = tempdir().expect("a temporary root");
    let (mut runtime, _) = runtime_with(root.path(), [gateway_artifact()]).await;
    let handle = runtime.handle();
    let execution = admitted_execution(&handle, 0).await;
    let scope = dokime_runtime::RecordScope::default().with_test_execution(execution.clone());
    let unknown = Identifier::new("fixture.unregistered").unwrap();
    let failure = handle.lease_streams(scope_of(&execution), &[foreign_stream(), unknown], "bench").await;
    assert!(failure.is_err());
    let facts = handle.stream_leases_for(&scope).await;
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].stream, foreign_stream());
    assert!(facts[0].closed_at.is_some());

    let retry = handle.lease_streams(scope_of(&execution), &[foreign_stream()], "bench").await.unwrap();
    assert_eq!(retry.len(), 1);
    assert!(handle.stream_lease_is_active(&retry[0]).await);
    let facts = handle.stream_leases_for(&scope).await;
    assert_eq!(facts.iter().filter(|fact| fact.closed_at.is_none()).count(), 1);
    handle.release_stream_leases(retry).await.unwrap();
    drop(handle);
    runtime.shutdown().await.expect("the runtime stops");
}

#[tokio::test(flavor = "multi_thread")]
async fn two_executions_rows() {
    let root = tempdir().expect("a temporary root");
    let (mut runtime, _) = runtime_with(root.path(), [gateway_artifact(), bench_artifact(), probe_artifact()]).await;
    let handle = runtime.handle();

    let owner = Arc::new(RecordingSidecar::default());
    let first_reader = Arc::new(RecordingSidecar::default());
    let second_reader = Arc::new(RecordingSidecar::default());
    register_sidecars(&handle, &[("gateway", &owner), ("bench", &first_reader), ("probe", &second_reader)]).await;

    let first = admitted_execution(&handle, 0).await;
    let second = admitted_execution(&handle, 1).await;
    let first_leases =
        handle.lease_streams(scope_of(&first), &[foreign_stream()], "bench").await.expect("the first execution leases the stream");
    let second_leases =
        handle.lease_streams(scope_of(&second), &[foreign_stream()], "probe").await.expect("the second execution leases the same stream");

    assert_eq!(first_leases.len(), 1);
    assert_eq!(second_leases.len(), 1);
    assert_ne!(first_leases[0], second_leases[0], "two holds are two leases even when they share one instance");

    handle.publish_telemetry_batch(gateway_batch(-61.5)).await.expect("the owning sidecar publishes its stream");

    // The same rows, in both runs' processes, values and instant alike. The
    // scope is deliberately not compared: each copy is stamped with the run it
    // is evidence for, which is how a sidecar running several executions tells
    // whose it is.
    let measured = |sidecar: &RecordingSidecar| {
        sidecar.delivered().iter().map(|row| (row.timestamp.to_string(), row.values.clone())).collect::<Vec<_>>()
    };
    let first_rows = measured(&first_reader);
    let second_rows = measured(&second_reader);
    assert_eq!(first_rows.len(), 1, "the first run was handed the row");
    assert_eq!(second_rows, first_rows, "both runs read the one measurement");
    assert_eq!(
        first_reader.delivered()[0].scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref().map(ToString::to_string)),
        Some(first.to_string()),
        "a delivered row is stamped with the run it is evidence for"
    );
    assert_eq!(
        second_reader.delivered()[0].scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref().map(ToString::to_string)),
        Some(second.to_string()),
    );

    // The Unscoped Measurement And Each Execution Retain Their Own Capture.
    assert_eq!(stored(&handle).await, 3);

    // Releasing one hold leaves the instance running for the other, and the
    // released run stops being evidence of anything new.
    handle.release_stream_leases(first_leases).await.unwrap();
    assert!(handle.stream_lease_is_active(&second_leases[0]).await);
    handle.publish_telemetry_batch(gateway_batch(-70.0)).await.expect("the owner keeps publishing");
    assert_eq!(first_reader.delivered().len(), 1, "a released hold is handed nothing further");
    assert_eq!(second_reader.delivered().len(), 2, "the run still holding it keeps receiving");
    assert_eq!(stored(&handle).await, 5, "only the remaining execution captures the second measurement");

    handle.release_stream_leases(second_leases).await.unwrap();
    drop(handle);
    runtime.shutdown().await.expect("the runtime stops");
}

#[tokio::test(flavor = "multi_thread")]
async fn two_holds_reading() {
    let root = tempdir().expect("a temporary root");
    let (mut runtime, reads) = runtime(root.path()).await;
    let handle = runtime.handle();
    let unscoped = || SamplingScope {
        test_cycle_id: None,
        test_case_id: None,
        test_execution_id: None,
    };

    let first = handle.lease_streams(unscoped(), &[stream()], "fixture.a").await.expect("the first execution leases the host stream");
    wait_for_reads(&reads, 1).await;

    // A second hold on the same stream joins the one session, and a second
    // stream leases independently of both.
    let second = handle.lease_streams(unscoped(), &[stream()], "fixture.b").await.expect("the second execution leases the same stream");
    let network = handle
        .lease_streams(unscoped(), &[Identifier::new("host.network").expect("addressable")], "fixture.b")
        .await
        .expect("the second host stream leases");
    assert!(handle.stream_lease_is_active(&first[0]).await);
    assert!(handle.stream_lease_is_active(&second[0]).await);
    assert!(handle.stream_lease_is_active(&network[0]).await);
    handle.release_stream_leases(network).await.unwrap();

    // The instance outlives the first release and keeps being read.
    handle.release_stream_leases(first.clone()).await.unwrap();
    assert!(!handle.stream_lease_is_active(&first[0]).await);
    assert!(handle.stream_lease_is_active(&second[0]).await);
    let before = *reads.borrow();
    wait_for_reads(&reads, before + 1).await;

    handle.release_stream_leases(second.clone()).await.unwrap();
    assert!(!handle.stream_lease_is_active(&second[0]).await, "a stream nobody holds has no active lease");

    drop(handle);
    runtime.shutdown().await.expect("the runtime stops");
}

// Liveness only: waits until the collector has read at least `count` times.
async fn wait_for_reads(reads: &Reads, count: usize) {
    reads.subscribe().wait_for(|reads| *reads >= count).await.expect("the collector count outlives the story");
}

#[tokio::test(flavor = "multi_thread")]
async fn foreign_stream_refuses() {
    let root = tempdir().expect("a temporary root");
    let (mut runtime, _) = runtime_with(root.path(), [gateway_artifact(), bench_artifact()]).await;
    let handle = runtime.handle();

    let owner = Arc::new(RecordingSidecar::default());
    let reader = Arc::new(RecordingSidecar::default());
    register_sidecars(&handle, &[("gateway", &owner), ("bench", &reader)]).await;

    let execution = admitted_execution(&handle, 0).await;
    let leases = handle
        .lease_streams(scope_of(&execution), &[foreign_stream()], "bench")
        .await
        .expect("a bench execution leases the gateway's stream");
    assert_eq!(leases.len(), 1);

    // What the owner measures, the host stores once and hands to the reader.
    handle.publish_telemetry_batch(gateway_batch(-61.5)).await.expect("the owning sidecar publishes its stream");

    let delivered = reader.delivered();
    assert_eq!(delivered.len(), 1, "the leasing run in the other process was handed the row");
    assert_eq!(delivered[0].stream, foreign_stream());
    assert_eq!(delivered[0].values.get("rssi"), Some(&Value::from(-61.5)));
    assert_eq!(
        delivered[0].scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref().map(ToString::to_string)),
        Some(execution.to_string()),
        "the row is stamped with the run that leased it"
    );
    assert!(owner.delivered().is_empty(), "the owner is not handed back what it just published");

    // A stream no installed plugin declares cannot be leased at all, and the
    // refusal names it.
    let refusal = handle
        .lease_streams(scope_of(&execution), &[Identifier::new("gateway.unknown").expect("addressable")], "bench")
        .await
        .expect_err("a stream with no observer cannot be leased")
        .to_string();
    assert!(refusal.contains("gateway.unknown"), "the refusal names the stream: {refusal}");

    // The host keeps the unscoped measurement and the leasing run keeps its
    // own capture; a third row would be the reader re-publishing what it was
    // handed.
    let stored = handle
        .selected_telemetry(dokime_runtime::domains::evidence::TelemetrySelection {
            streams: vec![foreign_stream().into()],
            ..Default::default()
        })
        .await
        .unwrap()
        .iter()
        .filter(|sample| sample.stream == foreign_stream())
        .map(|sample| sample.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()).map(|id| id.to_string()))
        .collect::<Vec<_>>();
    assert_eq!(stored.len(), 2, "the reader does not re-publish its rows");
    assert!(stored.contains(&None), "the unscoped measurement is stored");
    assert!(stored.contains(&Some(execution.to_string())), "the leasing run captures its own copy");

    // Releasing the hold stops the delivery, not the storage.
    handle.release_stream_leases(leases).await.unwrap();
    handle.publish_telemetry_batch(gateway_batch(-70.0)).await.expect("the owning sidecar keeps publishing");
    assert_eq!(reader.delivered().len(), 1, "a released hold is handed nothing further");

    drop(handle);
    runtime.shutdown().await.expect("the runtime stops");
}

/* === Tests === */
// template record demand captures startup and releases on cancellation.
