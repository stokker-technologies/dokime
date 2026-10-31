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

use crate::service_support::prelude::*;
use async_trait::async_trait;
use dokime_domain::model::{execution::effects::PendingEffect, plugins::RunStartRegistration};
use dokime_runtime::operations::effects::{ConfirmEffect, EffectResult, PrepareEffect, ReadEnvironment, StartEvidence};
use dokime_runtime::transport::{
    ExecutionSignal, RunExecution, SidecarCriterionOutcome, SidecarCriterionStatus, SidecarExecutionOutcome, SignalReply, TransportFuture,
};
use dokime_runtime::{
    InstalledPlugin, Message, Refusal,
    app::Composition,
    execution::{Caller, ExecutionHost, ExecutionScope, Runnable, SampleOutput},
    runtime::EmbeddedProvider,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Physical {
    on: AtomicBool,
    adapter_on: AtomicBool,
    restorations: Mutex<Vec<String>>,
    fail_restore: AtomicBool,
    captured: AtomicUsize,
    runtime: Mutex<Option<RuntimeHandle>>,
    entered: tokio::sync::Notify,
    release: tokio::sync::Notify,
    block_restore: AtomicBool,
    restoring: tokio::sync::Notify,
    restore_release: tokio::sync::Notify,
}

#[harmos::operation(id = "fixture.effect.restore", response = EffectResult, policy = dokime_runtime::operations::ORDINARY, visibility = Host, resources = [Physical])]
#[derive(Clone, Default, PartialEq)]
struct Inverse {
    pub on: bool,
    pub resource: String,
}

#[harmos::operation(id = "fixture.effect.capture", response = StartEvidence, policy = dokime_runtime::operations::ORDINARY, visibility = Host, resources = [Physical])]
#[derive(Clone, Default, PartialEq)]
struct Capture {}

struct Provider {
    physical: Arc<Physical>,
    mode: Mode,
}
struct Probe {
    physical: Arc<Physical>,
    mode: Mode,
}

/* === Enums === */

#[derive(Clone, Copy, Debug)]
enum Mode {
    Pass,
    Fail,
    Cancel,
    Resources,
}

#[harmos::catalog]
enum Operations {
    Inverse(Inverse),
    Capture(Capture),
}

/* === Traits === */

/* === Implementations === */

impl Physical {
    fn new() -> Self {
        Self {
            on: AtomicBool::new(true),
            adapter_on: AtomicBool::new(true),
            restorations: Mutex::new(Vec::new()),
            fail_restore: AtomicBool::new(false),
            captured: AtomicUsize::new(0),
            runtime: Mutex::new(None),
            entered: tokio::sync::Notify::new(),
            release: tokio::sync::Notify::new(),
            block_restore: AtomicBool::new(false),
            restoring: tokio::sync::Notify::new(),
            restore_release: tokio::sync::Notify::new(),
        }
    }
}

impl harmos::Operation for Inverse {
    type Error = Refusal;
    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<EffectResult>, Refusal> {
        let physical = ctx.resource::<Physical>();
        if physical.fail_restore.load(Ordering::SeqCst) {
            return Err(Refusal::of("controlled inverse failure"));
        }
        if physical.block_restore.load(Ordering::SeqCst) {
            physical.restoring.notify_one();
            physical.restore_release.notified().await;
        }
        if self.resource == "fixture.adapter" {
            physical.adapter_on.store(self.on, Ordering::SeqCst);
        } else {
            physical.on.store(self.on, Ordering::SeqCst);
        }
        physical.restorations.lock().unwrap().push(self.resource);
        Ok(harmos::Response::new(EffectResult { at: now().to_string() }))
    }
}

impl harmos::Operation for Capture {
    type Error = Refusal;
    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<StartEvidence>, Refusal> {
        let physical = ctx.resource::<Physical>();
        assert!(physical.on.load(Ordering::SeqCst));
        physical.captured.fetch_add(1, Ordering::SeqCst);
        Ok(harmos::Response::new(StartEvidence {
            payload_json: r#"{"adapters":[{"id":"before"}]}"#.into(),
        }))
    }
}

#[async_trait]
impl EmbeddedProvider for Provider {
    fn catalog(&self) -> InstalledPlugin {
        let mut suite = TestSuiteDefinition::new("fixture.effects", fixture_suite(&["passed"])).unwrap();
        suite.workflow = Some("run".into());
        InstalledPlugin {
            source: "effect_fixture".into(),
            version: "0.1.0".into(),
            enabled: true,
            suites: dokime_runtime::definition::SuiteSet::new([suite]),
            ..Default::default()
        }
    }
    fn streams(&self) -> Vec<harmos::sidecar::StreamDeclaration> {
        Vec::new()
    }
    async fn start(&self) -> dokime_runtime::runtime::Result<Vec<Arc<dyn Runnable>>> {
        Ok(vec![Arc::new(Probe {
            physical: self.physical.clone(),
            mode: self.mode,
        })])
    }
    async fn stop(&self) -> dokime_runtime::runtime::Result<()> {
        Ok(())
    }
}

impl Runnable for Probe {
    fn template_identifier(&self) -> &str {
        "fixture.template.t0"
    }
    fn run(
        &self, request: RunExecution, _: Arc<dyn ExecutionHost>, _: Option<SampleOutput>, _: Caller,
    ) -> TransportFuture<'_, Result<SidecarExecutionOutcome, Refusal>> {
        Box::pin(async move {
            let runtime = self.physical.runtime.lock().unwrap().clone().unwrap();
            assert_eq!(self.physical.captured.load(Ordering::SeqCst), 1);
            let id = TestExecutionId::from_uuid(request.execution_token).unwrap();
            let execution = runtime.watch_state().latest().state().executions.get(&id).unwrap().clone();
            // Match The Bridge's Run Scope Rather Than The Live Evidence Capture Phase.
            let mut effect_scope = execution.capture_scope();
            effect_scope.origin = Default::default();
            let mut context = harmos::operations::CallContext::default();
            context.insert_portable(ExecutionScope::execution(id.as_str())).unwrap();
            let caller = runtime.operations().with_context(context);
            for sample in 0..100 {
                let on = sample % 2 != 0;
                let pending = pending(effect_scope.clone(), self.physical.on.load(Ordering::SeqCst));
                let ticket = if sample == 0 {
                    let ticket = pending.ticket.clone();
                    runtime.begin_external_change(pending).await.map_err(Refusal::of)?;
                    ticket
                } else {
                    caller
                        .call::<PrepareEffect>(PrepareEffect {
                            resource: pending.resource,
                            inverse_operation: "effect_fixture.fixture.effect.restore".into(),
                            inverse: pending.inverse,
                            description: pending.description,
                        })
                        .await
                        .map_err(Refusal::of)?
                        .id
                        .clone()
                };
                assert_eq!(runtime.watch_state().latest().state().pending_effects.len(), 1);
                self.physical.on.store(on, Ordering::SeqCst);
                caller
                    .call::<ConfirmEffect>(ConfirmEffect {
                        ticket,
                        at: now().to_string(),
                        restored: false,
                    })
                    .await
                    .map_err(Refusal::of)?;
            }
            // A Later Failed Write Rolls Back Only That Call, Keeping The Original Inverse.
            let mut failed = pending(effect_scope.clone(), true);
            failed.source = "other_fixture".into();
            let ticket = failed.ticket.clone();
            runtime.begin_external_change(failed).await.map_err(Refusal::of)?;
            self.physical.on.store(false, Ordering::SeqCst);
            self.physical.on.store(true, Ordering::SeqCst);
            caller
                .call::<ConfirmEffect>(ConfirmEffect {
                    ticket,
                    at: now().to_string(),
                    restored: true,
                })
                .await
                .map_err(Refusal::of)?;
            assert_eq!(runtime.watch_state().latest().state().pending_effects.len(), 1);

            if matches!(self.mode, Mode::Resources) {
                let mut adapter = pending(effect_scope.clone(), true);
                adapter.resource = "fixture.adapter".into();
                adapter.inverse = Inverse {
                    on: true,
                    resource: adapter.resource.clone(),
                }
                .encode();
                let ticket = adapter.ticket.clone();
                runtime.begin_external_change(adapter).await.map_err(Refusal::of)?;
                self.physical.adapter_on.store(false, Ordering::SeqCst);
                runtime.complete_external_change(ticket, now()).await.map_err(Refusal::of)?;
            }
            // A Later Unconfirmed Write Also Leaves The Earliest Inverse In Charge.
            runtime.begin_external_change(pending(effect_scope, true)).await.map_err(Refusal::of)?;
            self.physical.on.store(false, Ordering::SeqCst);
            self.physical.entered.notify_one();
            if matches!(self.mode, Mode::Cancel) {
                self.physical.release.notified().await;
            }
            if matches!(self.mode, Mode::Fail) {
                return Err(Refusal::of("controlled run failure"));
            }
            let criteria = execution
                .snapshot
                .template()
                .unwrap()
                .criteria
                .iter()
                .map(|criterion| SidecarCriterionOutcome {
                    identifier: criterion.metadata.identifier.to_string(),
                    label: criterion.metadata.label.clone(),
                    status: SidecarCriterionStatus::Passed as i32,
                    message: None,
                })
                .collect();
            Ok(SidecarExecutionOutcome {
                passed: !matches!(self.mode, Mode::Cancel),
                message: None,
                criteria,
                detail_json: None,
                failed_in: None,
            })
        })
    }
    fn signal(&self, signal: ExecutionSignal) -> TransportFuture<'_, Result<SignalReply, Refusal>> {
        if signal.cancel {
            self.physical.release.notify_one();
        }
        let record_ordinal = signal.record.as_ref().map_or(0, |record| record.ordinal);
        Box::pin(async move {
            Ok(SignalReply {
                command: None,
                samples: Vec::new(),
                record_ordinal,
            })
        })
    }
    fn take_local_samples(&self, _: &str) -> Result<Vec<dokime_runtime::telemetry::TelemetrySample>, Refusal> {
        Ok(Vec::new())
    }
    fn interrupt(&self) {
        self.physical.release.notify_one();
    }
    fn shutdown(&self) -> TransportFuture<'_, ()> {
        Box::pin(async {})
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn now() -> Timestamp {
    Timestamp::from_system_time(std::time::SystemTime::now())
}
fn pending(scope: RecordScope, on: bool) -> PendingEffect {
    PendingEffect {
        ticket: uuid::Uuid::now_v7().to_string(),
        scope,
        source: "effect_fixture".into(),
        resource: "fixture.radio".into(),
        inverse_operation: "host.fixture.effect.restore".into(),
        inverse: Inverse { on, resource: "fixture.radio".into() }.encode(),
        description: "Fixture Radio Off".into(),
        prepared_at: now(),
        applied_at: None,
        coalesced: Vec::new(),
    }
}
fn configured(root: &Path, physical: &Arc<Physical>, mode: Mode) -> RuntimeConfig {
    let prepared = physical.clone();
    let initialized = physical.clone();
    let composition = Composition {
        prepare: Arc::new(move |setup, _, _| {
            setup.resource::<Physical>(prepared.clone());
            setup.operations::<Operations>(dokime_runtime::app::Lanes::Open);
            Ok(())
        }),
        initialize: Arc::new(move |runtime| {
            *initialized.runtime.lock().unwrap() = Some(runtime);
            Box::pin(async { Ok(()) })
        }),
        cleanup: Arc::new(|| Box::pin(async { Ok(()) })),
    };
    let host = InstalledPlugin {
        source: "host".into(),
        version: "0.1.0".into(),
        enabled: true,
        run_start: Some(RunStartRegistration {
            operation: "fixture.effect.capture".into(),
            request: Capture {}.encode(),
        }),
        ..Default::default()
    };
    let mut config = config(root).with_installed_plugins([host]).register(Arc::new(Provider { physical: physical.clone(), mode }));
    config.composition.push(composition);
    config
}
async fn planned(handle: &RuntimeHandle) -> (TestCycleId, TestExecutionId) {
    let suite = handle.test_suites().await.into_iter().find(|entry| entry.suite_id == "fixture.effects").unwrap().suite;
    let cycle = start_cycle(handle, suite).await;
    let view = cycle_snapshot(handle, &cycle).await;
    (cycle, view.test_executions[0].test_execution_id.clone())
}

/* === Tests === */

#[tokio::test]
async fn effect_lifecycle() {
    for mode in [Mode::Pass, Mode::Fail, Mode::Cancel] {
        let root = tempdir().unwrap();
        let physical = Arc::new(Physical::new());
        let mut runtime = Runtime::start(configured(root.path(), &physical, mode)).await.unwrap();
        let handle = runtime.handle();
        let (cycle, execution) = planned(&handle).await;
        let receipt = handle.start_next_execution(cycle.clone(), CommandContext::default()).await;
        assert_ne!(receipt.status, CommandStatus::Rejected, "{:?}", receipt.rejection);
        tokio::select! {
            biased;
            _ = physical.entered.notified() => {},
            view = until_state(&handle, |state| state.executions.get(&execution).is_some_and(|execution| execution.finished_at.is_some())) => panic!("run ended before changing state: {:?}", view.state().executions.get(&execution).map(|execution| (&execution.status, &execution.result))),
        }
        if matches!(mode, Mode::Cancel) {
            handle.stop_execution(execution.clone(), CommandContext::default()).await.unwrap();
        }
        let expected = match mode {
            Mode::Pass | Mode::Resources => ResultStatus::Passed,
            Mode::Fail => ResultStatus::Failed,
            Mode::Cancel => ResultStatus::Cancelled,
        };
        let settled =
            until_state(&handle, |state| state.executions.get(&execution).is_some_and(|execution| execution.finished_at.is_some())).await;
        assert_eq!(
            settled.state().executions.get(&execution).unwrap().result.as_ref().unwrap().status,
            expected,
            "steps: {:?}; failures: {:?}",
            settled.state().executions.get(&execution).unwrap().steps,
            settled.state().run_requests.values().map(|request| (&request.failure, &request.publication_failure)).collect::<Vec<_>>()
        );
        wait_for_cycle_outcome(&handle, &cycle, expected).await;
        assert!(physical.on.load(Ordering::SeqCst));
        assert!(handle.watch_state().latest().state().pending_effects.is_empty());
        let evidence = handle.evidence_snapshot().await.unwrap();
        let records = evidence
            .records
            .iter()
            .filter(|record| record.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()) == Some(&execution))
            .collect::<Vec<_>>();
        let snapshots = records.iter().filter(|record| record.record_type.as_str() == "test.execution.start.evidence").collect::<Vec<_>>();
        assert_eq!(snapshots.len(), 1);
        assert!(
            matches!(snapshots[0].details.get("snapshot"), Some(Value::Object(snapshot)) if matches!(snapshot.get("adapters"), Some(Value::Array(adapters)) if adapters.len() == 1))
        );
        let applied_changes =
            records.iter().filter(|record| record.record_type.as_str() == "test.execution.effect.applied").collect::<Vec<_>>();
        let restorations =
            records.iter().filter(|record| record.record_type.as_str() == "test.execution.effect.restored").collect::<Vec<_>>();
        let preparations =
            records.iter().filter(|record| record.record_type.as_str() == "test.execution.effect.prepared").collect::<Vec<_>>();
        assert_eq!(preparations.len(), 102);
        assert_eq!(applied_changes.len(), 100);
        assert_eq!(restorations.len(), 2);
        let original = preparations[0].details.get("ticket").unwrap();
        for (index, prepared) in preparations.iter().enumerate() {
            assert_eq!(prepared.details.get("restoration_ticket"), Some(original));
            assert_eq!(prepared.details.get("coalesced"), Some(&Value::Bool(index != 0)));
        }
        let tickets = preparations
            .iter()
            .map(|record| match record.details.get("ticket").unwrap() {
                Value::Text(ticket) => ticket.as_str(),
                _ => panic!("effect ticket must be text"),
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(tickets.len(), 102);
        for (applied, prepared) in applied_changes.iter().zip(&preparations) {
            assert_eq!(applied.details.get("ticket"), prepared.details.get("ticket"));
            assert_eq!(applied.details.get("prepared_at"), prepared.details.get("prepared_at"));
            assert_eq!(applied.details.get("applied_at"), Some(&Value::from(applied.timestamp.to_string())));
        }
        let applied = applied_changes[0];
        let restored = restorations[1];
        assert!(applied.timestamp.to_datetime().unwrap() <= restored.timestamp.to_datetime().unwrap());
        assert_eq!(applied.details.get("ticket"), restored.details.get("ticket"));
        assert_eq!(restorations[0].details.get("resource_restored"), Some(&Value::Bool(false)));
        assert_eq!(restorations[0].source, "other_fixture");
        assert_eq!(restored.details.get("resource_restored"), Some(&Value::Bool(true)));
        assert_eq!(*physical.restorations.lock().unwrap(), ["fixture.radio"]);
        runtime.shutdown().await.unwrap();
        physical.runtime.lock().unwrap().take();
    }
}

#[tokio::test]
async fn stop_shutdown_joins() {
    use futures_util::FutureExt;

    let root = tempdir().unwrap();
    let physical = Arc::new(Physical::new());
    let mut runtime = Runtime::start(configured(root.path(), &physical, Mode::Cancel)).await.unwrap();
    let handle = runtime.handle();
    let (cycle, execution) = planned(&handle).await;
    let receipt = handle.start_next_execution(cycle, CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted);
    physical.entered.notified().await;
    physical.block_restore.store(true, Ordering::SeqCst);
    let stop = {
        let handle = handle.clone();
        let execution = execution.clone();
        tokio::spawn(async move { handle.stop_execution(execution, CommandContext::default()).await })
    };
    physical.restoring.notified().await;
    let retry = handle.stop_execution(execution.clone(), CommandContext::default());
    tokio::pin!(retry);
    assert!(retry.as_mut().now_or_never().is_none());
    stop.abort();
    assert!(stop.await.unwrap_err().is_cancelled());
    let shutdown = runtime.shutdown();
    tokio::pin!(shutdown);
    assert!(shutdown.as_mut().now_or_never().is_none(), "shutdown must await the execution's original cleanup");
    physical.restore_release.notify_one();
    let (stopped, shutdown) = tokio::join!(retry, shutdown);
    stopped.unwrap();
    shutdown.unwrap();
    #[cfg(feature = "testing")]
    {
        let mut counts = dokime_runtime::testing::drain_counts(&execution);
        let reports = counts.remove("report").unwrap_or_default();
        assert!(reports <= 1, "background publication starts at most once before shutdown");
        assert_eq!(
            counts,
            BTreeMap::from([("effects", 1), ("publications", 1), ("records", 1), ("step_leases", 1), ("capture_leases", 1)]),
            "Stop, retry and shutdown share each physical cleanup stage"
        );
    }
    let snapshot = handle.watch_state().latest();
    assert!(
        snapshot.state().executions[&execution].saved.is_some()
            || snapshot.state().report_finalizations.get(&execution).is_some_and(|intent| intent.lifecycle.is_some()),
        "shutdown retains either the published report or its recoverable finalization intent"
    );
    assert_eq!(*physical.restorations.lock().unwrap(), ["fixture.radio"], "physical restoration runs once across Stop, retry and shutdown");
    physical.runtime.lock().unwrap().take();
}

#[tokio::test]
async fn effect_resource_order() {
    let root = tempdir().unwrap();
    let physical = Arc::new(Physical::new());
    let mut runtime = Runtime::start(configured(root.path(), &physical, Mode::Resources)).await.unwrap();
    let handle = runtime.handle();
    let (cycle, execution) = planned(&handle).await;
    let receipt = handle.start_next_execution(cycle.clone(), CommandContext::default()).await;
    assert_ne!(receipt.status, CommandStatus::Rejected, "{:?}", receipt.rejection);
    let settled =
        until_state(&handle, |state| state.executions.get(&execution).is_some_and(|execution| execution.finished_at.is_some())).await;
    assert_eq!(
        settled.state().executions.get(&execution).unwrap().result.as_ref().unwrap().status,
        ResultStatus::Passed,
        "steps: {:?}; failures: {:?}",
        settled.state().executions.get(&execution).unwrap().steps,
        settled.state().run_requests.values().map(|request| (&request.failure, &request.publication_failure)).collect::<Vec<_>>()
    );
    wait_for_cycle_outcome(&handle, &cycle, ResultStatus::Passed).await;
    assert!(physical.on.load(Ordering::SeqCst));
    assert!(physical.adapter_on.load(Ordering::SeqCst));
    assert_eq!(*physical.restorations.lock().unwrap(), ["fixture.adapter", "fixture.radio"]);
    assert!(handle.watch_state().latest().state().pending_effects.is_empty());
    runtime.shutdown().await.unwrap();
    physical.runtime.lock().unwrap().take();
}

#[tokio::test]
async fn effect_recovery() {
    let root = tempdir().unwrap();
    let physical = Arc::new(Physical::new());
    let config = configured(root.path(), &physical, Mode::Pass);
    let mut runtime = Runtime::start(config.clone()).await.unwrap();
    let handle = runtime.handle();
    let environment = handle.operations().call::<ReadEnvironment>(ReadEnvironment {}).await.unwrap();
    assert_eq!(environment.backend_pid, std::process::id());
    assert!(!environment.in_step);
    let standalone = handle
        .operations()
        .call::<PrepareEffect>(PrepareEffect {
            resource: "fixture.radio".into(),
            inverse_operation: "host.fixture.effect.restore".into(),
            inverse: Inverse {
                on: true,
                resource: "fixture.radio".into(),
            }
            .encode(),
            description: "Standalone Radio".into(),
        })
        .await
        .unwrap();
    assert!(standalone.id.is_empty());
    handle
        .operations()
        .call::<ConfirmEffect>(ConfirmEffect {
            ticket: standalone.id.clone(),
            at: now().to_string(),
            restored: false,
        })
        .await
        .unwrap();
    assert!(handle.watch_state().latest().state().pending_effects.is_empty());
    let (_, execution) = planned(&handle).await;
    let scope = handle.watch_state().latest().state().executions.get(&execution).unwrap().capture_scope();
    let effect = pending(scope.clone(), true);
    let ticket = effect.ticket.clone();
    handle.begin_external_change(effect).await.unwrap();
    physical.on.store(false, Ordering::SeqCst);
    handle.complete_external_change(ticket, now()).await.unwrap();
    let later = pending(scope.clone(), false);
    let ticket = later.ticket.clone();
    handle.begin_external_change(later).await.unwrap();
    physical.on.store(true, Ordering::SeqCst);
    handle.complete_external_change(ticket, now()).await.unwrap();
    let later = pending(scope.clone(), true);
    let ticket = later.ticket.clone();
    handle.begin_external_change(later).await.unwrap();
    physical.on.store(false, Ordering::SeqCst);
    handle.complete_external_change(ticket, now()).await.unwrap();
    handle.begin_external_change(pending(scope, false)).await.unwrap();
    physical.on.store(true, Ordering::SeqCst);
    assert_eq!(handle.watch_state().latest().state().pending_effects.len(), 1);
    runtime.shutdown().await.unwrap();
    drop(handle);
    physical.runtime.lock().unwrap().take();
    drop(runtime);
    physical.fail_restore.store(true, Ordering::SeqCst);
    assert!(Runtime::start(config.clone()).await.is_err());
    physical.runtime.lock().unwrap().take();
    assert!(physical.on.load(Ordering::SeqCst));
    physical.fail_restore.store(false, Ordering::SeqCst);
    let mut recovered = Runtime::start(config).await.unwrap();
    assert!(physical.on.load(Ordering::SeqCst));
    assert_eq!(*physical.restorations.lock().unwrap(), ["fixture.radio"]);
    assert!(recovered.handle().watch_state().latest().state().pending_effects.is_empty());
    // Startup Retires The Fixture's Closed Cycle; Its Records Stay In Native History.
    let handle = recovered.handle();
    let at = handle.storage().flush().await.unwrap();
    let mut reader = handle
        .storage()
        .read_records::<dokime_runtime::storage::records::RecordRow>(harmos::storage::RecordRead {
            at,
            selection: harmos::storage::query::RecordSelection {
                schema: dokime_runtime::storage::records::schema(),
                scope: None,
                item: None,
                source_start: None,
                source_end: None,
            },
            after: 0,
        })
        .await
        .unwrap();
    let mut restorations = Vec::new();
    while let Some(batch) = reader.next().await.unwrap() {
        let page = crate::service_support::native::record_page(batch).unwrap();
        restorations.extend(
            page.iter()
                .filter(|record| {
                    record.record_type.as_str() == "test.execution.effect.restored"
                        && record.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()) == Some(&execution)
                })
                .cloned(),
        );
    }
    assert_eq!(restorations.len(), 1);
    assert!(restorations[0].timestamp.to_datetime().is_ok());
    recovered.shutdown().await.unwrap();
    physical.runtime.lock().unwrap().take();
}
