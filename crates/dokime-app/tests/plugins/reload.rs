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
use std::{path::Path, sync::Arc, time::Duration};
// Third Party Imports
use tempfile::tempdir;
// Local Imports
use super::*;
use dokime::execution::{ExecutionStatus, ResultStatus};
use dokime::{Identifier, Metadata, TestCycleId};
use dokime_app::plugins::PluginBundle;
use dokime_app::runtime::command::CommandContext;
use dokime_app::{Runtime, RuntimeHandle};
use dokime_app::{execution::CycleView, transactions::CycleDefinition};

/* === Constants === */
// These assert lifecycle correctness across native processes and durable storage
// saves, not a wall-clock SLA. Keep a deadlock bound independent of RPC deadlines.
const LIFECYCLE_TIMEOUT: Duration = Duration::from_secs(60);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct BlockingProvider {
    entered: tokio::sync::Notify,
    release: tokio::sync::Notify,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl dokime_app::capabilities::execution::provider::ExecutionFixture for BlockingProvider {
    fn run_cycle<'a>(
        &'a self,
        _context: dokime_app::capabilities::execution::provider::ExecutionFixtureContext,
        cycle: &'a CycleView,
        _target: &'a dokime::domains::ExecutionProviderTarget,
    ) -> dokime_app::capabilities::execution::provider::ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            self.entered.notify_one();
            self.release.notified().await;
            Ok(cycle_outcome(
                cycle,
                ProviderIdentity {
                    module: "unrelated-provider",
                    registrations: 1,
                    suite: "unrelated.checks",
                },
                |case, execution| dokime::transport::ProviderExecutionOutcome {
                    case_id: case.test_case_id.as_str().to_owned(),
                    test_execution_id: execution.test_execution_id.as_str().to_owned(),
                    template_identifier: "unrelated.smoke".into(),
                    label: "Unrelated Smoke".into(),
                    module: "unrelated-provider".into(),
                    status: "passed".into(),
                    phases: Vec::new(),
                    records: Vec::new(),
                    telemetry: Vec::new(),
                    criteria: Vec::new(),
                    failures: Vec::new(),
                },
            ))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn runtime(root: &Path, plugins: impl IntoIterator<Item = PluginBundle>) -> Runtime {
    runtime_with_blocker(root, plugins, None).await
}

async fn runtime_with_blocker(
    root: &Path,
    plugins: impl IntoIterator<Item = PluginBundle>,
    blocker: Option<Arc<BlockingProvider>>,
) -> Runtime {
    let config = config(root.join("runtime")).with_plugins(plugins);
    match blocker {
        // One fixture is a legacy catch-all seam. Two keyed entries preserve
        // normal native dispatch for the plugin under test.
        Some(blocker) => {
            let fixtures: [(String, Arc<dyn ExecutionFixture>); 2] = [
                ("unrelated-provider".into(), blocker.clone()),
                ("unused-provider".into(), blocker),
            ];
            started_with_fixtures(config, fixtures).await.0
        }
        None => started(config).await.0,
    }
}

async fn run(handle: &RuntimeHandle) -> (TestCycleId, dokime::core::CommandId) {
    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    run_suite(handle, suite).await
}

async fn run_suite(
    handle: &RuntimeHandle,
    suite: dokime::definition::TestSuite,
) -> (TestCycleId, dokime::core::CommandId) {
    let suite_id = handle
        .test_suites()
        .await
        .into_iter()
        .find(|entry| {
            entry.suite.test_case_templates.iter().any(|template| {
                suite.test_case_templates.iter().any(|selected| {
                    selected.test_case_template_id == template.test_case_template_id
                })
            })
        })
        .expect("suite has a registered provider")
        .suite_id;
    let id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(
                    Identifier::new("reload.cycle").unwrap(),
                    "Reload cycle",
                    None,
                ),
                cases: materialize_cases(&suite_id, &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(id.clone(), CommandContext::default())
        .await;
    assert_eq!(
        receipt.status,
        dokime_app::runtime::command::CommandStatus::Accepted,
        "{receipt:?}"
    );
    (id, receipt.command_id)
}

async fn cycle(handle: &RuntimeHandle, id: &TestCycleId) -> CycleView {
    handle
        .cycles()
        .await
        .into_iter()
        .find(|cycle| &cycle.test_cycle_id == id)
        .unwrap()
}

async fn completed(handle: &RuntimeHandle, id: &TestCycleId) -> CycleView {
    tokio::time::timeout(LIFECYCLE_TIMEOUT, async {
        loop {
            let value = cycle(handle, id).await;
            if !value.test_executions.is_empty()
                && value
                    .test_executions
                    .iter()
                    .all(|execution| execution.finished_at.is_some())
            {
                return value;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|error| {
        panic!(
            "execution settles: {error}; cycle: {:?}",
            handle.watch_state().latest().state().cycles.get(id),
        )
    })
}

async fn marker(path: &Path) -> String {
    tokio::time::timeout(LIFECYCLE_TIMEOUT, async {
        loop {
            if let Ok(value) = std::fs::read_to_string(path)
                && !value.is_empty()
            {
                return value;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("native step writes its executable path")
}

#[cfg(unix)]
// unchanged reload needs no staging and still checks input integrity.
// incompatible stream revision is refused before stopping the plugin.
pub(crate) async fn incompatible_stream_stopping() {
    let root = tempdir().unwrap();
    let initialized = root.path().join("initialized");
    let original = revised_bundle(root.path(), "fixture-suites", "reload-suite", "old")
        .configured("fixture-suites", serde_json::json!({"marker": initialized}))
        .unwrap();
    let mut owner = runtime(root.path(), [original]).await;
    let handle = owner.handle();
    let suites = handle.test_suites().await;
    std::fs::write(&initialized, b"unchanged").unwrap();

    let candidate = revised_bundle(
        root.path(),
        "fixture-suites",
        "reload-suite",
        "changed-stream",
    );
    let inspected = harmos::plugin::Bundle::inspect(
        &std::fs::read(&candidate.path).unwrap(),
        Default::default(),
    )
    .unwrap();
    let needle = br#""id":"fixture-suites.pulse","schema":1,"window":1024"#;
    let mut changed = 0;
    let payloads: Vec<_> = inspected
        .manifest()
        .files
        .iter()
        .map(|file| {
            let mut bytes = inspected.file(&file.path).unwrap().to_vec();
            if file.executable.is_some() {
                let positions: Vec<_> = bytes
                    .windows(needle.len())
                    .enumerate()
                    .filter(|(_, value)| *value == needle)
                    .map(|(offset, _)| offset)
                    .collect();
                for offset in positions {
                    bytes[offset + needle.len() - 1] = b'7';
                    changed += 1;
                }
            }
            bytes
        })
        .collect();
    assert!(
        changed > 0,
        "the fixture carries its inspected stream inventory"
    );
    let files: Vec<_> = inspected
        .manifest()
        .files
        .iter()
        .zip(&payloads)
        .map(|(file, bytes)| harmos::plugin::Source {
            path: &file.path,
            bytes,
            executable: file.executable.clone(),
        })
        .collect();
    let replacement = harmos::plugin::Bundle::build(
        inspected.manifest().identity.clone(),
        &files,
        Default::default(),
    )
    .unwrap();
    std::fs::write(&candidate.path, replacement.bytes()).unwrap();
    let candidate = PluginBundle::new(&candidate.path)
        .configured("fixture-suites", serde_json::json!({"marker": initialized}))
        .unwrap();

    let error = owner.reload_plugin(candidate).await.unwrap_err();
    assert!(
        error.to_string().contains("without a new version"),
        "{error}"
    );
    assert_eq!(handle.test_suites().await, suites);
    assert_eq!(std::fs::read(&initialized).unwrap(), b"unchanged");
    let (id, _) = run(&handle).await;
    assert_eq!(
        completed(&handle, &id).await.result.unwrap().status,
        ResultStatus::Passed
    );
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn activation_failure_recovery() {
    let root = tempdir().unwrap();
    let unrelated_lifecycle = root.path().join("unrelated-lifecycle");
    let original = revised_bundle(root.path(), "fixture-suites", "reload-suite", "old");
    let unrelated = revised_bundle(root.path(), "fixture-telemetry", "unrelated", "original")
        .configured(
            "validation-telemetry",
            serde_json::json!({"lifecycle":unrelated_lifecycle}),
        )
        .unwrap();
    let mut owner = runtime(root.path(), [original, unrelated]).await;
    let handle = owner.handle();
    // The unrelated plugin's contracts; the refused plugin's own go with it.
    let unrelated_contracts = async || {
        let mut contracts = handle.telemetry_contracts().await;
        contracts.retain(|contract| !contract.stream.as_str().starts_with("fixture-suites."));
        contracts
    };
    let contracts = unrelated_contracts().await;
    assert!(
        contracts
            .iter()
            .any(|contract| contract.stream.as_str() == "validation-telemetry.continuous")
    );
    // A candidate that is not a distribution at all is refused without touching
    // the loaded plugin.
    let suites = handle.test_suites().await;
    let invalid = root.path().join("invalid.dokime");
    std::fs::write(&invalid, b"not a distribution").unwrap();
    assert!(
        owner
            .reload_plugin(PluginBundle::new(invalid))
            .await
            .is_err()
    );
    assert_eq!(handle.test_suites().await, suites);

    let refused = revised_bundle(root.path(), "fixture-suites", "reload-suite", "refused")
        .configured("fixture-suites", serde_json::json!({"refuse":true}))
        .unwrap();
    assert!(owner.reload_plugin(refused).await.is_err());
    assert!(
        handle
            .test_suites()
            .await
            .iter()
            .all(|suite| suite.source != "fixture-suites")
    );
    assert_eq!(unrelated_contracts().await, contracts);
    assert_eq!(std::fs::read(&unrelated_lifecycle).unwrap(), b"initialized");
    let repaired = revised_bundle(root.path(), "fixture-suites", "reload-suite", "repaired");
    owner.reload_plugin(repaired).await.unwrap();
    let (id, _) = run(&handle).await;
    assert_eq!(
        completed(&handle, &id).await.result.unwrap().status,
        ResultStatus::Passed
    );
    assert_eq!(std::fs::read(&unrelated_lifecycle).unwrap(), b"initialized");
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn queued_reload_run() {
    use dokime::execution::CycleStatus;
    use dokime_app::capabilities::execution::requests::RunPhase;

    let root = tempdir().unwrap();
    let blocked = Arc::new(BlockingProvider {
        entered: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    let execution_marker = root.path().join("queued-execution");
    let execution_release = root.path().join("queued-release");
    let original = revised_bundle(root.path(), "fixture-suites", "reload-suite", "old")
        .configured(
            "fixture-suites",
            serde_json::json!({"execution_marker":execution_marker,"block_execution":true,"execution_release":execution_release}),
        )
        .unwrap();
    let replacement = revised_bundle(root.path(), "fixture-suites", "reload-suite", "new");
    let mut owner = runtime_with_blocker(root.path(), [original], Some(blocked.clone())).await;
    let handle = owner.handle();
    let mut unrelated = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    unrelated.metadata = Metadata::new(
        Identifier::new("unrelated.checks").unwrap(),
        "Unrelated checks",
        None,
    );
    unrelated.test_case_templates[0].metadata = Metadata::new(
        Identifier::new("unrelated.smoke").unwrap(),
        "Unrelated smoke",
        None,
    );
    unrelated.test_case_templates[0].test_case_template_id = dokime::TemplateId::new();
    let mut registration =
        dokime::domains::TestSuiteDefinition::new("unrelated.checks", unrelated.clone()).unwrap();
    registration.workflow = Some("unrelated.checks".into());
    handle
        .set_suites(
            "unrelated-provider",
            dokime::definition::SuiteSet::new([registration]),
        )
        .await
        .unwrap();
    let (unrelated_id, _) = run_suite(&handle, unrelated).await;
    tokio::time::timeout(Duration::from_secs(10), blocked.entered.notified())
        .await
        .unwrap();
    let unrelated_before = cycle(&handle, &unrelated_id).await;
    let (queued_id, request_id) = run(&handle).await;
    let before = cycle(&handle, &queued_id).await;
    let queued = handle.run_request(&request_id).await.unwrap().unwrap();
    assert_eq!(queued.summary.phase, RunPhase::Queued);
    assert_eq!(queued.summary.planned, 1);
    assert_eq!(queued.summary.prepared, 0);
    assert_eq!(queued.claimed, 0);
    assert!(queued.summary.provider_started_at.is_none());
    assert!(!execution_marker.exists());
    // Planning has already frozen the execution, but queue admission has not
    // started it while the unrelated provider owns the serial execution lane.
    assert_eq!(before.test_executions.len(), 1);
    let planned = &before.test_executions[0];
    assert_eq!(planned.status, ExecutionStatus::Skipped);
    assert!(planned.started_at.is_none());
    assert!(planned.finished_at.is_none());
    let mut reload = Box::pin(owner.reload_plugin(replacement));
    tokio::select! {
        outcome = &mut reload => panic!("reload completed while affected work remained queued: {outcome:?}"),
        _ = tokio::time::sleep(Duration::from_millis(200)) => {}
    }
    assert_eq!(
        handle
            .run_request(&request_id)
            .await
            .unwrap()
            .unwrap()
            .summary
            .phase,
        RunPhase::Queued
    );
    assert_eq!(
        cycle(&handle, &queued_id).await.test_executions,
        before.test_executions
    );
    assert_eq!(
        cycle(&handle, &unrelated_id).await.lifecycle,
        CycleStatus::Running
    );
    assert_eq!(
        cycle(&handle, &unrelated_id).await.test_executions,
        unrelated_before.test_executions
    );

    blocked.release.notify_one();
    let settle = async {
        marker(&execution_marker).await;
        assert_eq!(
            cycle(&handle, &queued_id).await.test_executions[0].snapshot,
            planned.snapshot
        );
        std::fs::write(&execution_release, b"release").unwrap();
    };
    let (outcome, ()) = tokio::join!(
        async {
            tokio::time::timeout(LIFECYCLE_TIMEOUT, &mut reload)
                .await
                .unwrap()
        },
        settle,
    );
    outcome.unwrap();
    drop(reload);
    let settled = completed(&handle, &queued_id).await;
    assert_eq!(
        settled.test_executions[0].test_execution_id,
        planned.test_execution_id
    );
    assert_eq!(settled.test_executions[0].snapshot, planned.snapshot);
    assert_eq!(
        settled.test_executions[0].status,
        ExecutionStatus::Passed,
        "{settled:?}"
    );
    assert!(settled.cases().eq(before.cases()));
    assert_eq!(
        completed(&handle, &unrelated_id)
            .await
            .result
            .unwrap()
            .status,
        ResultStatus::Passed
    );
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn provider_reload_dependent() {
    let root = tempdir().unwrap();
    let events = root.path().join("dependent-lifecycle");
    let provider = revised_bundle(root.path(), "fixture-telemetry", "provider", "old");
    let dependent = revised_bundle(root.path(), "fixture-dependent", "dependent", "same")
        .configured(
            "fixture-dependent",
            serde_json::json!({"lifecycle": events}),
        )
        .unwrap();
    // Supply the dependent first: the graph must establish startup order.
    let mut owner = runtime(root.path(), [dependent, provider]).await;
    let assert_host_control = |runtime: &dokime_app::Runtime| {
        let graph = runtime.handle().router().graph();
        let provider = graph
            .ordered()
            .iter()
            .find(|definition| definition.id == "validation-telemetry")
            .expect("the provider is admitted");
        // The observer controls and the capture template's run and signal:
        // operations only the host dispatches, whoever else depends on it.
        let controls: Vec<_> = provider
            .contributions
            .operations
            .iter()
            .filter(|operation| operation.visibility == harmos::operations::Visibility::Host)
            .collect();
        let caller = runtime
            .fixture_operation_caller("fixture-dependent")
            .expect("the dependent has a live managed caller");
        assert_eq!(controls.len(), 4, "{controls:?}");
        for operation in controls {
            assert_eq!(operation.visibility, harmos::operations::Visibility::Host);
            assert!(matches!(
                caller.submit_encoded(operation.id.as_str(), Vec::new()),
                Err(harmos::Error::OperationUnauthorized { .. })
            ));
        }
    };
    assert_host_control(&owner);
    let before = std::fs::read_to_string(&events).unwrap();
    let first = before.trim().strip_prefix("initialize:").unwrap();

    let replacement = revised_bundle(root.path(), "fixture-telemetry", "provider", "new");
    owner.reload_plugin(replacement).await.unwrap();
    assert_host_control(&owner);
    let after = std::fs::read_to_string(&events).unwrap();
    let lines: Vec<_> = after.lines().collect();
    assert_eq!(lines.len(), 3, "{after}");
    assert_eq!(lines[1], format!("cleanup:{first}"));
    let second = lines[2].strip_prefix("initialize:").unwrap();
    assert_ne!(first, second, "the dependent must run in a new process");

    owner.shutdown().await.unwrap();
    assert_eq!(
        std::fs::read_to_string(&events).unwrap(),
        format!("{after}cleanup:{second}\n")
    );
}

/* === Tests === */
