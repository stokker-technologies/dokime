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
    execution::{Caller, ExecutionHost, Runnable, SampleOutput},
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
    fail_restore: AtomicBool,
    captured: AtomicUsize,
    runtime: Mutex<Option<RuntimeHandle>>,
    entered: tokio::sync::Notify,
    release: tokio::sync::Notify,
}

#[harmos::operation(id = "fixture.effect.restore", response = EffectResult, policy = dokime_runtime::operations::ORDINARY, visibility = Host, resources = [Physical])]
#[derive(Clone, Default, PartialEq)]
struct Inverse {
    pub on: bool,
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
            fail_restore: AtomicBool::new(false),
            captured: AtomicUsize::new(0),
            runtime: Mutex::new(None),
            entered: tokio::sync::Notify::new(),
            release: tokio::sync::Notify::new(),
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
        physical.on.store(self.on, Ordering::SeqCst);
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
            for on in [false, true] {
                let pending = pending(execution.capture_scope(), self.physical.on.load(Ordering::SeqCst));
                let ticket = pending.ticket.clone();
                runtime.begin_external_change(pending).await.map_err(Refusal::of)?;
                assert!(runtime.watch_state().latest().state().pending_effects.iter().any(|effect| effect.ticket == ticket));
                self.physical.on.store(on, Ordering::SeqCst);
                runtime.complete_external_change(ticket, now()).await.map_err(Refusal::of)?;
            }
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
        Box::pin(async {
            Ok(SignalReply {
                command: None,
                samples: Vec::new(),
                record_ordinal: 0,
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
        inverse: Inverse { on }.encode(),
        description: "Fixture Radio Off".into(),
        prepared_at: now(),
        applied_at: None,
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
            Mode::Pass => ResultStatus::Passed,
            Mode::Fail => ResultStatus::Failed,
            Mode::Cancel => ResultStatus::Cancelled,
        };
        let settled =
            until_state(&handle, |state| state.executions.get(&execution).is_some_and(|execution| execution.finished_at.is_some())).await;
        assert_eq!(settled.state().executions.get(&execution).unwrap().result.as_ref().unwrap().status, expected);
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
        assert_eq!(applied_changes.len(), 2);
        assert_eq!(restorations.len(), 2);
        for (applied, restored) in applied_changes.iter().zip(restorations.iter().rev()) {
            assert_eq!(applied.details.get("ticket"), restored.details.get("ticket"));
            assert_eq!(applied.details.get("applied_at"), Some(&Value::from(applied.timestamp.to_string())));
        }
        let applied = applied_changes[0];
        let restored = restorations[1];
        assert!(applied.timestamp.to_datetime().unwrap() <= restored.timestamp.to_datetime().unwrap());
        assert_eq!(applied.details.get("ticket"), restored.details.get("ticket"));
        runtime.shutdown().await.unwrap();
        physical.runtime.lock().unwrap().take();
    }
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
            inverse: Inverse { on: true }.encode(),
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
    let effect = pending(scope, true);
    let ticket = effect.ticket.clone();
    handle.begin_external_change(effect).await.unwrap();
    physical.on.store(false, Ordering::SeqCst);
    handle.complete_external_change(ticket, now()).await.unwrap();
    runtime.shutdown().await.unwrap();
    drop(handle);
    physical.runtime.lock().unwrap().take();
    drop(runtime);
    physical.fail_restore.store(true, Ordering::SeqCst);
    assert!(Runtime::start(config.clone()).await.is_err());
    physical.runtime.lock().unwrap().take();
    assert!(!physical.on.load(Ordering::SeqCst));
    physical.fail_restore.store(false, Ordering::SeqCst);
    let mut recovered = Runtime::start(config).await.unwrap();
    assert!(physical.on.load(Ordering::SeqCst));
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
