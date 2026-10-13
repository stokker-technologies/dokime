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
use crate::support::prelude::*;
use dokime::domains::ExecutionProviderTarget;
use dokime_app::capabilities::execution::provider::{
    ExecutionFixtureContext, ExecutionFixtureFuture,
};
use dokime_app::execution::CycleView;

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
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a dokime_app::execution::CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            for (index, execution) in runnable_executions(cycle).enumerate() {
                activate_fixture_execution(&context, execution).await;
                if index == 0 {
                    let scope = provider_execution_scope(cycle);
                    context
                        .publish_telemetry_batch(wide_batch(scope))
                        .await
                        .unwrap();
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
    handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    wait_for_cycle_lifecycle(handle, &cycle_id, CycleStatus::Completed).await;
    // Aggregates Finalize After The Terminal Transaction, Before The Seal
    let cycle = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let cycle = cycle_snapshot(handle, &cycle_id).await;
            if cycle.saved.is_some() {
                return cycle;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("completed cycle seals");
    assert_eq!(cycle.test_executions.len(), MEMBERS);
    cycle
}

async fn resident(handle: &RuntimeHandle, cycle_id: &TestCycleId) -> bool {
    handle
        .cycles()
        .await
        .iter()
        .any(|cycle| &cycle.test_cycle_id == cycle_id)
}

// Every Sealed Member Reopens From Native History With Its Wide Aggregates
async fn assert_history(handle: &RuntimeHandle, cycles: &[&CycleView], entries: usize) {
    let history = handle
        .browse_executions(Default::default(), |page, _| Ok(page.clone()))
        .await
        .unwrap();
    assert!(history.discovery_complete && history.conflicts.is_empty());
    assert_eq!(history.entries.len(), entries);
    drop(history);
    for cycle in cycles {
        for (index, member) in cycle.test_executions.iter().enumerate() {
            let checkpoint =
                crate::support::native::checkpoint(handle, &member.test_execution_id).await;
            let view = crate::support::native::open(handle, &checkpoint).await;
            assert_eq!(view.execution().test_cycle_id, cycle.test_cycle_id);
            assert!(view.execution().saved.is_some());
            if index == 0 {
                let aggregates = view
                    .execution()
                    .output_parameters
                    .keys()
                    .filter(|key| key.starts_with("telemetry."))
                    .count();
                assert!(aggregates >= COLUMNS * 13);
            }
        }
    }
}

async fn reboot(root: &Path) -> (Runtime, RuntimeHandle) {
    tokio::time::timeout(
        Duration::from_secs(60),
        started_with_fixture(wide_config(root), Arc::new(WideTelemetryExecutor)),
    )
    .await
    .expect("recovery boot is bounded")
}
fn wide_config(root: &Path) -> RuntimeConfig {
    let columns = (0..COLUMNS)
        .map(|index| fixture_host_column(&format!("value_{index:02}"), &format!("Value {index}")))
        .collect();
    config(root).with_installed_plugins([dokime_app::InstalledPlugin {
        source: "fixture.host".to_owned(),
        version: "test".to_owned(),
        description: "Fixture Host".to_owned(),
        enabled: true,
        telemetry_contracts: vec![fixture_host_contract(STREAM, "Fixture Wide", columns)],
        ..Default::default()
    }])
}

fn wide_batch(scope: RecordScope) -> TelemetryBatch {
    let samples = (0..4)
        .map(|sequence| {
            let mut sample = TelemetrySample::new(
                identifier(STREAM),
                sequence,
                Timestamp::now_utc(),
                "fixture.host",
            );
            for index in 0..COLUMNS {
                sample.values.insert(
                    format!("value_{index:02}"),
                    Value::Number(index as f64 + sequence as f64 / 4.0),
                );
            }
            sample.scope = Some(scope.clone());
            sample
        })
        .collect();
    TelemetryBatch::new(identifier(STREAM), samples)
}

/* === Tests === */

// Sealed Cycles Leave Live State At The Next Cycle Start Or Restart, So A Wide
// Cycle Never Strands Its Root And The State Snapshot Stays Bounded.
pub(crate) async fn cycle_survives_skipped_snapshot() {
    let logs = Captured::default();
    let _subscriber = tracing::subscriber::set_default(
        tracing_subscriber::fmt()
            .with_writer(logs.clone())
            .with_ansi(false)
            .with_max_level(tracing::Level::WARN)
            .finish(),
    );
    let root = tempdir().unwrap();
    let suite = fixture_suite(&["passed"; MEMBERS]);

    // The Latest Sealed Cycle Stays Resident Within Its Session
    let (mut owner, handle) = reboot(root.path()).await;
    register_python_suite(&handle, suite.clone()).await;
    let first = sealed_cycle(&handle, suite.clone()).await;
    assert!(resident(&handle, &first.test_cycle_id).await);
    drop(handle);
    owner.shutdown().await.unwrap();

    // A Restart Retires It From Live State Without Touching Its History
    let (mut owner, handle) = reboot(root.path()).await;
    assert!(!resident(&handle, &first.test_cycle_id).await);
    assert_history(&handle, &[&first], MEMBERS).await;

    // Starting Another Cycle Retires The Previous Sealed One In Session
    logs.clear();
    register_python_suite(&handle, suite.clone()).await;
    let second = sealed_cycle(&handle, suite.clone()).await;
    assert!(resident(&handle, &second.test_cycle_id).await);
    let third = start_cycle(&handle, suite, false).await;
    assert!(!resident(&handle, &second.test_cycle_id).await);
    assert!(resident(&handle, &third).await);
    drop(handle);
    owner.shutdown().await.unwrap();
    assert!(
        !logs.text().contains("snapshot skipped"),
        "one working cycle should fit one state snapshot:\n{}",
        logs.text()
    );

    let (mut owner, handle) = reboot(root.path()).await;
    assert!(!resident(&handle, &second.test_cycle_id).await);
    // The Third Cycle Was Started But Never Run; Its Planned Members Are Listed Too
    assert_history(&handle, &[&first, &second], 3 * MEMBERS).await;
    drop(handle);
    owner.shutdown().await.unwrap();
}
