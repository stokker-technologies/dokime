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
use harmos::storage::{ReadLimits, RecoveryPosition, StorageFuture, StoreHistory};

// Local Imports
use crate::support::prelude::*;
use crate::support::publication::{PublicationBackend, PublicationFault};
use dokime::storage::objects::cycle::Cycle;

/* === Constants === */
const ROOT: &str = "DOKIME_CYCLE_RESTART_ROOT";
const PHASE: &str = "DOKIME_CYCLE_RESTART_PHASE";
const CHILD: &str = "runtime::storage::cycle_restart::cycle_restart_child";
const BEFORE_REPORT: i32 = 73;
const BEFORE_ADOPTION: i32 = 74;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct CrashAdoption {
    inner: Arc<dyn StateBackend>,
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
    fn read_snapshot(
        &self,
        limit: std::num::NonZeroUsize,
    ) -> StorageFuture<'_, Option<RecoverySnapshot>> {
        self.inner.read_snapshot(limit)
    }
    fn write_snapshot<'a>(&'a self, snapshot: &'a RecoverySnapshot) -> StorageFuture<'a, ()> {
        self.inner.write_snapshot(snapshot)
    }
    fn read_entries<'a>(
        &'a self,
        after: &'a RecoveryPosition,
        through: &'a RecoveryPosition,
        limits: ReadLimits,
    ) -> StorageFuture<'a, RecoveryPage> {
        self.inner.read_entries(after, through, limits)
    }
    fn write_entries<'a>(&'a self, entries: &'a [RecoveryEntry]) -> StorageFuture<'a, ()> {
        Box::pin(async move {
            for entry in entries {
                let payload: serde_json::Value = serde_json::from_slice(&entry.payload).unwrap();
                if has_cycle_adoption(&payload) {
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
        serde_json::Value::Object(fields) => fields.iter().any(|(key, value)| {
            (key == "sealed_cycle" && value.is_object()) || has_cycle_adoption(value)
        }),
        serde_json::Value::Array(values) => values.iter().any(has_cycle_adoption),
        _ => false,
    }
}

#[tokio::test]
async fn cycle_restart_child() {
    let Some(root) = std::env::var_os(ROOT) else {
        return;
    };
    let phase = std::env::var(PHASE).unwrap();
    let adoption = phase == "adoption";
    let fault = Arc::new(if adoption {
        PublicationFault::new("unused")
    } else {
        PublicationFault::crash(".cycle-report", BEFORE_REPORT)
    });
    let providers = BTreeMap::from([(
        "testdb".to_owned(),
        Arc::new(PublishingExecutor) as Arc<dyn ExecutionFixture>,
    )]);
    let parts = RuntimeHandle::boot_with_backends(
        config(PathBuf::from(root)),
        providers,
        true,
        Box::new(move |mut backends| {
            backends.primary = Arc::new(PublicationBackend::new(backends.primary, fault));
            if adoption {
                backends.state = Arc::new(CrashAdoption {
                    inner: backends.state,
                });
            }
            backends
        }),
    )
    .await
    .unwrap();
    let mut owner = Runtime::from_parts(parts);
    owner.install_plugins().await.unwrap();
    let runtime = owner.handle();
    let suite = fixture_suite(&["passed", "passed"]);
    register_custom_suite(&runtime, suite.clone()).await;
    let id = start_cycle(&runtime, suite, false).await;
    let receipt = runtime.run_cycle_cases(id, CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    tokio::time::timeout(Duration::from_secs(90), std::future::pending::<()>())
        .await
        .expect("the selected backend phase must terminate the child process");
}

#[tokio::test]
async fn frozen_cycle_cursor_and_durable_seal_survive_process_crash() {
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

        let (mut owner, runtime) =
            booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
        let state = runtime.state_read();
        assert_eq!(state.cycle_report_finalizations.len(), 1);
        let intent = state
            .cycle_report_finalizations
            .values()
            .next()
            .unwrap()
            .clone();
        let id = intent.source.test_cycle_id.clone();
        assert_eq!(intent.source.lifecycle, CycleStatus::Completed);
        assert_eq!(
            state
                .run_requests
                .values()
                .next()
                .unwrap()
                .completion_at
                .as_ref(),
            Some(&intent.source.updated_at)
        );
        assert!(
            runtime
                .storage()
                .cursor_saved()
                .covers(&intent.evidence_at)
                .unwrap()
        );
        let children = state.cycles[&id].execution_ids.clone();
        drop(state);
        for child in children {
            let view = runtime
                .storage()
                .view_with_limits(crate::support::budgets::story_settings().finalization())
                .await
                .unwrap();
            let native = crate::runtime::storage::execution::ExecutionView::open(
                runtime.storage(),
                view,
                &child,
                harmos::storage::ObjectVersion::Latest,
                crate::support::budgets::story_settings().decoded_bytes,
            )
            .await
            .unwrap()
            .unwrap();
            let seal = native
                .execution()
                .saved
                .as_ref()
                .expect("every child was sealed before the intent");
            assert!(
                intent
                    .evidence_at
                    .covers(&native.revision().unwrap().position)
                    .unwrap()
            );
            assert!(intent.evidence_at.covers(&seal.dependencies_saved).unwrap());
        }
        let native = runtime
            .storage()
            .read_object::<Cycle>(
                id.as_str(),
                harmos::storage::ObjectVersion::Latest,
                crate::support::budgets::story_settings().decoded_bytes,
            )
            .await
            .unwrap()
            .unwrap();
        let before = native.value.restore().unwrap().saved;
        assert_eq!(before.is_some(), phase == "adoption");
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
                &[1],
            )
            .await
            .unwrap();
        runtime
            .storage()
            .flush_until(&marker.position)
            .await
            .unwrap();
        assert!(marker.position.sequence > intent.evidence_at.sequence);
        let opened = runtime.open_cycle_report(&id).await.unwrap();
        let original_html = opened.html.to_vec();
        drop(opened);
        assert_eq!(runtime.state_read().cycle_report_finalizations[&id], intent);
        drop(runtime);
        owner.shutdown().await.unwrap();
        drop(owner);

        let (mut owner, runtime) =
            booted_with_fixture(root.path(), Arc::new(PublishingExecutor)).await;
        assert!(
            !runtime
                .state_read()
                .cycle_report_finalizations
                .contains_key(&id)
        );
        let native = runtime
            .storage()
            .read_object::<Cycle>(
                id.as_str(),
                harmos::storage::ObjectVersion::Latest,
                crate::support::budgets::story_settings().decoded_bytes,
            )
            .await
            .unwrap()
            .unwrap();
        let sealed = native.value.restore().unwrap();
        let seal = sealed.saved.as_ref().unwrap();
        assert_eq!(
            seal.evidence_through, intent.evidence_at,
            "retry cannot adopt the later marker boundary"
        );
        let mut header = sealed.clone();
        header.saved = None;
        assert_eq!(
            header, intent.source,
            "restart cannot rewrite the frozen terminal lifecycle"
        );
        if let Some(before) = before {
            assert_eq!(
                seal, &before,
                "durable seal adoption must not rerender or replace artifacts"
            );
        }
        let opened = runtime.open_cycle_report(&id).await.unwrap();
        assert_eq!(opened.origin, crate::ReportOrigin::Sealed);
        assert_eq!(opened.html.as_slice(), original_html.as_slice());
        drop((opened, native, runtime));
        owner.shutdown().await.unwrap();
    }
}
