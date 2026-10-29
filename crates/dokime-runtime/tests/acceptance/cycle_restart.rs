/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cycle Finalization Crash And Restart Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process::Command;

// Third Party Imports
use harmos::storage::state::{RecoveryEntry, RecoveryPage, RecoverySnapshot, StateBackend};
use harmos::storage::{ReadLimits, RecoveryPosition, StorageBatch, StorageFuture, StoreHistory};

// Local Imports
use crate::service_support::prelude::*;
use crate::service_support::publication::{PublicationBackend, PublicationFault};
use dokime_runtime::storage::objects::cycle::Cycle;

/* === Constants === */
const ROOT: &str = "DOKIME_CYCLE_RESTART_ROOT";
const PHASE: &str = "DOKIME_CYCLE_RESTART_PHASE";
const CHILD: &str = "cycle_restart::cycle_restart_child";
const BEFORE_REPORT: i32 = 73;
const BEFORE_ADOPTION: i32 = 74;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct CrashAdoption {
    inner: Arc<dyn StateBackend>,
    root: PathBuf,
    adoption: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl StateBackend for CrashAdoption {
    fn history(&self) -> &StoreHistory {
        self.inner.history()
    }
    fn open(&self) -> StorageFuture<'_, RecoveryPosition> {
        self.inner.open()
    }
    fn read_snapshot(&self, limit: std::num::NonZeroUsize) -> StorageFuture<'_, Option<RecoverySnapshot>> {
        self.inner.read_snapshot(limit)
    }
    fn write_snapshot(&self, snapshot: StorageBatch<RecoverySnapshot>) -> StorageFuture<'_, ()> {
        self.inner.write_snapshot(snapshot)
    }
    fn read_entries<'a>(
        &'a self, after: &'a RecoveryPosition, through: &'a RecoveryPosition, limits: ReadLimits,
    ) -> StorageFuture<'a, RecoveryPage> {
        self.inner.read_entries(after, through, limits)
    }
    fn write_entries(&self, entries: StorageBatch<RecoveryEntry>) -> StorageFuture<'_, ()> {
        Box::pin(async move {
            for entry in entries.as_slice() {
                let payload: serde_json::Value = serde_json::from_slice(&entry.payload).unwrap();
                if let Some(intent) = cycle_intent(&payload) {
                    std::fs::write(self.root.join("intent.json"), serde_json::to_vec(intent).unwrap()).unwrap();
                }
                if self.adoption && has_cycle_adoption(&payload) {
                    std::fs::write(self.root.join("adoption.json"), serde_json::to_vec(&payload).unwrap()).unwrap();
                    // The Native Cycle Seal Is Already Durable. Exit Without
                    // Drop Or Shutdown Flushing This Managed Adoption Entry.
                    std::process::exit(BEFORE_ADOPTION);
                }
            }
            self.inner.write_entries(entries).await
        })
    }
    fn flush<'a>(&'a self, through: &'a RecoveryPosition) -> StorageFuture<'a, RecoveryPosition> {
        self.inner.flush(through)
    }
    fn close(&self) -> StorageFuture<'_, ()> {
        self.inner.close()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

fn has_cycle_adoption(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(fields) => {
            fields.iter().any(|(key, value)| (key == "sealed_cycle" && value.is_object()) || has_cycle_adoption(value))
        }
        serde_json::Value::Array(values) => values.iter().any(has_cycle_adoption),
        _ => false,
    }
}

fn cycle_intent(value: &serde_json::Value) -> Option<&serde_json::Value> {
    match value {
        serde_json::Value::Object(fields) => {
            let source = fields.get("source");
            if fields.contains_key("evidence_at") && source.is_some_and(|source| source.get("execution_ids").is_some()) {
                return Some(value);
            }
            fields.values().find_map(cycle_intent)
        }
        serde_json::Value::Array(values) => values.iter().find_map(cycle_intent),
        _ => None,
    }
}

fn adopted_cycle(value: &serde_json::Value) -> Option<&serde_json::Value> {
    match value {
        serde_json::Value::Object(fields) => fields.get("sealed_cycle").or_else(|| fields.values().find_map(adopted_cycle)),
        serde_json::Value::Array(values) => values.iter().find_map(adopted_cycle),
        _ => None,
    }
}

#[tokio::test]
async fn cycle_restart_child() {
    let Some(root) = std::env::var_os(ROOT) else {
        return;
    };
    let phase = std::env::var(PHASE).unwrap();
    let adoption = phase == "adoption";
    let root = PathBuf::from(root);
    let injected_root = root.clone();
    let fault = Arc::new(if adoption { PublicationFault::new("unused") } else { PublicationFault::crash(".cycle-report", BEFORE_REPORT) });
    let providers = BTreeMap::from([("testdb".to_owned(), Arc::new(PublishingExecutor) as Arc<dyn ExecutionFixture>)]);
    let owner = Runtime::with_fixture_backends(config(root), providers, move |mut backends| {
        backends.primary = Arc::new(PublicationBackend::new(backends.primary, fault));
        backends.state = Arc::new(CrashAdoption {
            inner: backends.state,
            root: injected_root,
            adoption,
        });
        backends
    })
    .await
    .unwrap();
    let runtime = owner.handle();
    let suite = fixture_suite(&["passed", "passed"]);
    register_custom_suite(&runtime, suite.clone()).await;
    let id = start_cycle(&runtime, suite).await;
    let receipt = runtime.start_next_execution(id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    crate::service_support::waits::wait_for_run_settled(&runtime, &receipt.command_id).await;
    // The Start Receipt Covers One Execution; Ordered Continuation Finishes The Cycle.
    let completed = crate::service_support::waits::wait_for_cycle_outcome(&runtime, &id, ResultStatus::Passed).await;
    assert_eq!(completed.test_executions.len(), 2);
    assert!(completed.test_executions.iter().all(|execution| execution.saved.is_some()));
    runtime.seal_cycle(id, CommandContext::default()).await.unwrap();
    tokio::time::timeout(Duration::from_secs(90), std::future::pending::<()>())
        .await
        .expect("the selected backend phase must terminate the child process");
}

#[tokio::test]
async fn cycle_crash_recovery() {
    for (phase, status) in [("report", BEFORE_REPORT), ("adoption", BEFORE_ADOPTION)] {
        let root = tempdir().unwrap();
        // Only The Dedicated Child Test Receives The Exit Injection.
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env(ROOT, root.path())
            .env(PHASE, phase)
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(status),
            "{phase}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let intent: dokime_runtime::transactions::CycleReportFinalization =
            serde_json::from_slice(&std::fs::read(root.path().join("intent.json")).unwrap()).unwrap();
        let id = intent.source.test_cycle_id.clone();
        assert_eq!(intent.source.lifecycle, CycleStatus::Completed);
        let (mut owner, runtime) = booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
        let snapshot = runtime.watch_state().latest();
        let state = snapshot.state();
        assert!(state.cycle_report_finalizations.is_empty(), "startup finishes the frozen seal before admission");
        assert!(state.run_requests.values().all(|request| request.summary.phase.terminal()));
        assert!(runtime.storage().cursor_saved().covers(&intent.evidence_at).unwrap());
        // The Startup Seal Retires The Cycle From Live State; Native History Serves It.
        assert!(!state.cycles.contains_key(&id), "a sealed cycle is served from native history after restart");
        let children = intent.source.execution_ids.clone();
        drop(snapshot);
        for child in children {
            let native = dokime_runtime::runtime::ExecutionView::open(runtime.storage(), &child, harmos::storage::ObjectVersion::Latest)
                .await
                .unwrap()
                .unwrap();
            let seal = native.execution().saved.as_ref().expect("every child was sealed before the intent");
            assert!(intent.evidence_at.covers(&native.revision().unwrap().position).unwrap());
            assert!(intent.evidence_at.covers(&seal.dependencies_saved).unwrap());
        }
        let native = runtime.storage().read_object::<Cycle>(id.as_str(), harmos::storage::ObjectVersion::Latest).await.unwrap().unwrap();
        let sealed_at_startup = native.value.restore().unwrap();
        assert_eq!(sealed_at_startup.phase, dokime_domain::model::execution::result::CyclePhase::Sealed);
        if phase == "adoption" {
            let payload: serde_json::Value = serde_json::from_slice(&std::fs::read(root.path().join("adoption.json")).unwrap()).unwrap();
            let before: dokime_domain::model::execution::result::TestCycle =
                serde_json::from_value(adopted_cycle(&payload).unwrap().clone()).unwrap();
            assert_eq!(before, sealed_at_startup, "startup adopts the exact durable seal without rerendering");
        }
        drop(native);

        // Introduce A Later Durable Position Before Retrying The Frozen Report.
        let marker = runtime
            .storage()
            .write_file(
                harmos::storage::FileMetadata {
                    id: harmos::Identifier::new("restart-marker").unwrap(),
                    scope: None,
                    media_type: "application/octet-stream".into(),
                    expected_length: Some(1),
                    expected_sha256: None,
                },
                vec![1],
            )
            .await
            .unwrap();
        runtime.storage().flush_until(&marker.position).await.unwrap();
        assert!(marker.position.sequence > intent.evidence_at.sequence);
        let mut opened = runtime.open_cycle_report(&id).await.unwrap();
        let original_html = crate::service_support::reports::opened_bytes(&mut opened).await;
        drop(opened);
        runtime.seal_cycle(id.clone(), CommandContext::default()).await.unwrap();
        assert!(!runtime.watch_state().latest().state().cycle_report_finalizations.contains_key(&id));
        let native = runtime.storage().read_object::<Cycle>(id.as_str(), harmos::storage::ObjectVersion::Latest).await.unwrap().unwrap();
        let sealed = native.value.restore().unwrap();
        let seal = sealed.saved.as_ref().unwrap();
        assert_eq!(seal.evidence_through, intent.evidence_at, "retry cannot adopt the later marker boundary");
        let mut header = sealed.clone();
        header.saved = None;
        header.phase = dokime_domain::model::execution::result::CyclePhase::Sealing;
        assert_eq!(header, intent.source, "restart cannot rewrite the frozen terminal lifecycle");
        assert_eq!(sealed, sealed_at_startup, "an explicit repeat cannot replace the startup seal");
        let mut opened = runtime.open_cycle_report(&id).await.unwrap();
        assert_eq!(opened.origin, dokime_runtime::ReportOrigin::Sealed);
        assert_eq!(crate::service_support::reports::opened_bytes(&mut opened).await.as_slice(), original_html.as_slice());
        drop((opened, native, runtime));
        owner.shutdown().await.unwrap();
    }
}
