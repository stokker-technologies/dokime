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
use std::sync::Mutex;

// Third Party Imports

// Local Imports
use crate::service_support::prelude::*;
use dokime_domain::model::execution::result::CycleView;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::domains::execution::provider::{ExecutionFixtureContext, ExecutionFixtureFuture};

/* === Constants === */
// The Steady Charging Cycle That Stranded Its Root Had 9 Members, One With 637
// Telemetry Aggregates (49 Columns x 13 Statistics) Charged Near 4 MB.
const MEMBERS: usize = 9;
const COLUMNS: usize = 49;
const STREAM: &str = "fixture.wide";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

// Publishes One Wide Telemetry Stream Into The First Member Only
struct WideTelemetryExecutor;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl std::io::Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Captured {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl ExecutionFixture for WideTelemetryExecutor {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a dokime_domain::model::execution::result::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            for (index, execution) in runnable_executions(cycle).enumerate() {
                activate_fixture_execution(&context, execution).await;
                if index == 0 {
                    let scope = provider_execution_scope(cycle);
                    context.publish_telemetry_batch(wide_batch(scope)).await.unwrap();
                }
            }
            Ok(python_outcome(cycle, "passed", false))
        })
    }
}

impl Captured {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }

    fn clear(&self) {
        self.0.lock().unwrap().clear();
    }
}

/* === Public Functions === */

/* === Private Functions === */
// Run One Wide Cycle To Its Seal And Return Its Final Resident View
async fn sealed_cycle(handle: &RuntimeHandle, suite: TestSuite) -> CycleView {
    let cycle_id = start_cycle(handle, suite, false).await;
    handle.run_cycle_cases(cycle_id.clone(), CommandContext::default()).await;
    wait_for_cycle_lifecycle(handle, &cycle_id, CycleStatus::Completed).await;
    // Aggregates Finalize After The Terminal Transaction, Before The Seal
    until_state(handle, |state| state.cycle_views().any(|cycle| cycle.test_cycle_id == cycle_id && cycle.saved.is_some())).await;
    let cycle = cycle_snapshot(handle, &cycle_id).await;
    assert_eq!(cycle.test_executions.len(), MEMBERS);
    cycle
}

async fn resident(handle: &RuntimeHandle, cycle_id: &TestCycleId) -> bool {
    handle.cycles().await.iter().any(|cycle| &cycle.test_cycle_id == cycle_id)
}

// Every Sealed Member Reopens From Native History With Its Wide Aggregates
async fn assert_history(handle: &RuntimeHandle, cycles: &[&CycleView], entries: usize) {
    let history = handle.browse_executions(Default::default()).await.unwrap();
    assert!(history.discovery_complete && history.conflicts.is_empty());
    assert_eq!(history.entries.len(), entries);
    drop(history);
    for cycle in cycles {
        for (index, member) in cycle.test_executions.iter().enumerate() {
            let checkpoint = crate::service_support::native::checkpoint(handle, &member.test_execution_id).await;
            let view = crate::service_support::native::open(handle, &checkpoint).await;
            assert_eq!(view.execution().test_cycle_id, cycle.test_cycle_id);
            assert!(view.execution().saved.is_some());
            if index == 0 {
                let aggregates = view.execution().output_parameters.keys().filter(|key| key.starts_with("telemetry.")).count();
                assert!(aggregates >= COLUMNS * 13);
            }
        }
    }
}

async fn reboot(root: &Path) -> (Runtime, RuntimeHandle) {
    started_with_fixture(wide_config(root), Arc::new(WideTelemetryExecutor)).await
}

fn wide_config(root: &Path) -> RuntimeConfig {
    let columns = (0..COLUMNS).map(|index| fixture_host_column(&format!("value_{index:02}"), &format!("Value {index}"))).collect();
    config(root).with_installed_plugins([dokime_runtime::InstalledPlugin {
        source: "fixture.host".to_owned(),
        version: "test".to_owned(),
        description: "Fixture Host".to_owned(),
        enabled: true,
        telemetry_contracts: vec![fixture_host_contract(STREAM, "Fixture Wide", columns).with_storage_schema(wide_schema())],
        ..Default::default()
    }])
}

fn wide_schema() -> harmos::schema::graph::Schema {
    use harmos::schema::{
        Cardinality,
        graph::{Field, Kind, Schema},
    };
    let mut fields = vec![Field { name: "timestamp".into(), tag: 1, kind: Kind::String, cardinality: Cardinality::Required }];
    fields.extend((0..COLUMNS).map(|index| Field {
        name: format!("value_{index:02}").into(),
        tag: index as u32 + 2,
        kind: Kind::F64,
        cardinality: Cardinality::Required,
    }));
    Schema { root: 0, definitions: vec![fields] }
}

fn wide_batch(scope: RecordScope) -> TelemetryBatch {
    let samples = (0..4)
        .map(|sequence| {
            let mut sample = TelemetrySample::new(
                dokime_runtime::model::identity::new_id(),
                identifier(STREAM),
                sequence,
                dokime_runtime::model::time::now_utc(),
                "fixture.host",
            );
            let mut payload = Vec::new();
            harmos::message::postcard::Value::write(&sample.timestamp.as_str().to_owned(), &mut payload);
            for index in 0..COLUMNS {
                let value = index as f64 + sequence as f64 / 4.0;
                harmos::message::postcard::Value::write(&value, &mut payload);
                sample.values.insert(format!("value_{index:02}"), Value::Number(value));
            }
            sample.storage_payload = Some(payload);
            sample.scope = Some(scope.clone());
            sample
        })
        .collect();
    TelemetryBatch::new(dokime_runtime::model::identity::new_id(), identifier(STREAM), samples)
}

// Shut A Session Down And Return The Payload Bytes Of The State Snapshot It
// Wrote, After Checking The Session's Journal Against The Entry Bound The
// State Policy Derives From Its Checkpoint.
async fn shut_down(mut owner: Runtime, handle: RuntimeHandle, logs: &Captured) -> usize {
    let root = handle.storage_root();
    let limits = dokime_runtime::storage::state_settings();
    let journal = std::fs::read(root.join("state.jsonl")).unwrap();
    let largest = journal.split_inclusive(|byte| *byte == b'\n').map(<[u8]>::len).max().unwrap();
    // A Wide Member's Finalized Aggregates Are The Largest Entry, Near 344 KB.
    assert!(
        largest <= limits.max_entry_bytes.get(),
        "the session's largest journal entry of {largest} bytes against the state entry bound"
    );
    drop(handle);
    owner.shutdown().await.unwrap();
    assert!(!logs.text().contains("snapshot skipped"), "the session's state should fit one state snapshot:\n{}", logs.text());

    let snapshot = std::fs::read(root.join("state.json")).expect("shutdown writes a state snapshot");
    assert!(snapshot.len() <= limits.max_checkpoint_bytes.get());
    let record: serde_json::Value = serde_json::from_slice(&snapshot).unwrap();
    let payload = record["value"]["payload"].as_str().unwrap();
    // Unpadded Base64 Carries Three Payload Bytes In Every Four Characters.
    payload.len() * 3 / 4
}

// The Snapshot Encoding Stores Shared Catalogs And Plans Once; Decoding Restores Them
fn assert_round_trip(handle: &RuntimeHandle) {
    let latest = handle.watch_state().latest();
    let encoded = serde_json::to_vec(latest.state()).unwrap();
    let decoded: dokime_runtime::state::RuntimeState = serde_json::from_slice(&encoded).unwrap();
    assert!(&decoded == latest.state(), "state survives its snapshot encoding");
}

/* === Tests === */

// Sealed Cycles Leave Live State At The Next Cycle Start Or Restart, So A Wide
// Cycle Never Strands Its Root And The State Snapshot Stays Bounded With The
// Shipped Plugin Catalogs Loaded.
#[tokio::test]
async fn skipped_snapshot_recovery() {
    // Measured 934,729 And 673,866 Bytes: About 512 KB Of State (420 KB Of It The
    // Pooled Plugin Catalog) Plus The Journal Tail, Mostly The Wide Member's
    // Finalized Aggregates; Report Lifecycles Reference Their Frozen Revisions.
    const SNAPSHOT_LIMIT: usize = 960 * 1024;

    let logs = Captured::default();
    let _subscriber = tracing::subscriber::set_default(
        tracing_subscriber::fmt().with_writer(logs.clone()).with_ansi(false).with_max_level(tracing::Level::WARN).finish(),
    );
    let root = tempdir().unwrap();
    let suite = fixture_suite(&["passed"; MEMBERS]);

    // The Latest Sealed Cycle Stays Resident Within Its Session
    let (mut owner, handle) = reboot(root.path()).await;
    register_python_suite(&handle, suite.clone()).await;
    let first = sealed_cycle(&handle, suite.clone()).await;
    assert!(resident(&handle, &first.test_cycle_id).await);
    assert_round_trip(&handle);
    drop(handle);
    owner.shutdown().await.unwrap();
    // Not Yet Bounded: The Resident Wide Cycle Holds About 340 KB Of Aggregates
    // In State And Again About 300 KB In The Journal Tail Of Its Finalization.
    logs.clear();

    // A Restart Retires It From Live State Without Touching Its History
    let (owner, handle) = reboot(root.path()).await;
    assert!(!resident(&handle, &first.test_cycle_id).await);
    assert_history(&handle, &[&first], MEMBERS).await;

    // Starting Another Cycle Retires The Previous Sealed One In Session
    register_python_suite(&handle, suite.clone()).await;
    let second = sealed_cycle(&handle, suite.clone()).await;
    assert!(resident(&handle, &second.test_cycle_id).await);
    let third = start_cycle(&handle, suite, false).await;
    assert!(!resident(&handle, &second.test_cycle_id).await);
    assert!(resident(&handle, &third).await);
    let written = shut_down(owner, handle, &logs).await;
    assert!(written <= SNAPSHOT_LIMIT, "state snapshot of {written} bytes");

    // The Next Boot Restores That Snapshot And Writes Its Own
    let (owner, handle) = reboot(root.path()).await;
    assert!(!resident(&handle, &second.test_cycle_id).await);
    // The Third Cycle Was Started But Never Run; Its Planned Members Are Listed Too
    assert_history(&handle, &[&first, &second], 3 * MEMBERS).await;
    let written = shut_down(owner, handle, &logs).await;
    assert!(written <= SNAPSHOT_LIMIT, "state snapshot of {written} bytes");
}
