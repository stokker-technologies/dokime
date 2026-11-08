/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Hosted Captures Settle Under Their Execution Barrier

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::plugin::Declared;

// Local Imports
use crate::domains::files::{Bench, append, write};
use crate::service_support::prelude::*;
use dokime_domain::model::files::wire::{FileRule, FileSource, MirrorCapture};
use dokime_runtime::execution::{Run, Signal};
use dokime_runtime::operations::files::StartCapture;
use dokime_runtime::prelude::{InitializationContext, StepContext};
use dokime_runtime::transport::ExecutionSignal;

/* === Constants === */

const TEMPLATE: &str = "capture_stop.checks.capture";
static STEP_WAITING: tokio::sync::Notify = tokio::sync::Notify::const_new();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Plugin;

struct Checks;

struct Capture;

#[dokime_runtime::parameters]
struct Parameters {
    #[parameter(label = "Path", default = "")]
    path: String,
}

/* === Enums === */

#[dokime_runtime::catalog]
enum Suites {
    Checks(Checks),
}

#[dokime_runtime::catalog]
enum Templates {
    Capture(Capture),
}

/* === Traits === */

/* === Implementations === */

impl harmos::builtins::lifecycle::Policy for Plugin {
    const INITIALIZATION_POLICY: harmos::operations::OperationPolicy = super::native::POLICY;
    const SHUTDOWN_POLICY: harmos::operations::OperationPolicy = super::native::POLICY;
}

#[dokime_runtime::plugin(id = "capture_stop", requires(), contributions(dokime_runtime::Suites<Suites>), restart = "never", description = "Stops a hosted watch mid-step.")]
impl dokime_runtime::Plugin for Plugin {
    const CATALOGS: &'static [harmos::Catalog] = &[];
}

impl harmos::Lifecycle for Plugin {
    type Config = dokime_runtime::plugin::Configuration;
    type Error = dokime_runtime::Refusal;
    type Context = harmos::sidecar::Supervisor;
    async fn prepare(_: Self::Config, setup: &mut harmos::Setup<Self>) -> Result<Self, Self::Error> {
        dokime_runtime::execution::prepare::<Self, Suites>(setup)?;
        Ok(Self)
    }
}

#[dokime_runtime::suite(id = "capture_stop.checks", label = "Capture Across a Stop", templates = Templates)]
impl Checks {}

#[dokime_runtime::template(id = TEMPLATE, label = "Capture", parameters = Parameters)]
impl Capture {
    async fn initialize(_: &Parameters, _: &mut InitializationContext<'_>) -> Result<Self, dokime_runtime::Error> {
        Ok(Self)
    }

    #[dokime_runtime::step(label = "Capture Files")]
    async fn capture(&mut self, ctx: &mut StepContext<'_>) -> Result<(), dokime_runtime::Error> {
        // Start Capture
        let source = FileSource::tree(ctx.parameters().path.as_str(), [FileRule::include("**/*.log")]);
        let request = StartCapture::new(source, "many", MirrorCapture::every(3600));
        ctx.operation(request).await?;

        STEP_WAITING.notify_one();
        std::future::pending().await
    }
}

/* === Public Functions === */

/* === Private Functions === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn captured_stop_settles() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    for index in 0..64 {
        write(&tree.path().join(format!("nested/{index}.log")), b"baseline\n");
    }
    let native = super::native::Native::initialize(
        super::native::attach::<Plugin>(),
        Plugin::VERSION,
        serde_json::to_vec(&dokime_runtime::plugin::Configuration::default()).unwrap(),
        |_, _| {},
        vec![],
    )
    .await;
    *native.host.watch_runtime.lock().unwrap() = Some(bench.handle.clone());
    let request = super::native::run_request(
        &bench.execution,
        "capture_stop.checks",
        TEMPLATE,
        serde_json::to_string(&BTreeMap::from([("path", Value::from(tree.path().to_str().unwrap()))])).unwrap(),
    );
    let run = {
        let mut context = harmos::operations::CallContext::default();
        context.insert_portable(dokime_runtime::execution::ExecutionScope::execution(bench.execution.clone())).unwrap();
        let operations = native.runtime.operations.with_context(context);
        tokio::spawn(async move {
            let outcome = super::native::call_wire::<Run<Capture>>(&operations, "capture_stop", Run::new(request)).await.unwrap();
            (outcome, std::time::Instant::now())
        })
    };
    STEP_WAITING.notified().await;
    for index in 0..64 {
        append(&tree.path().join(format!("nested/{index}.log")), b"changed\n");
    }
    let execution = TestExecutionId::from_uuid(bench.execution.clone()).unwrap();
    let gate = dokime_runtime::testing::hold_capture_seal(&execution);
    let stop = {
        let handle = bench.handle.clone();
        let execution = execution.clone();
        tokio::spawn(async move { handle.stop_execution(execution, CommandContext::default()).await.unwrap() })
    };
    until_state(&bench.handle, |state| {
        state.run_requests.values().any(|request| request.summary.stop_requested && request.selected_execution_ids.contains(&execution))
    })
    .await;
    let started = std::time::Instant::now();
    super::native::call_wire::<Signal<Capture>>(
        &native.runtime.operations,
        "capture_stop",
        Signal::new(ExecutionSignal {
            execution_token: bench.execution.clone(),
            cancel: true,
            ..Default::default()
        }),
    )
    .await
    .unwrap();
    gate.entered().await;
    let (outcome, settled_at) = run.await.unwrap();
    eprintln!("captured_stop_settles: run stop reply {:?}", settled_at.duration_since(started));
    assert!(!outcome.passed);
    let attempts = native
        .host
        .evidence()
        .into_iter()
        .flat_map(|batch| batch.step_attempts)
        .map(|encoded| serde_json::from_str::<dokime_domain::model::execution::result::TestStepAttempt>(&encoded).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].termination, dokime_domain::model::execution::termination::TerminationReason::Cancelled);
    assert!(!stop.is_finished());
    gate.release();
    stop.await.unwrap();
    let artifacts = bench.handle.artifacts().await.unwrap();
    assert_eq!(artifacts.iter().filter(|record| record.filename.starts_with("many/")).count(), 64);
    for index in 0..64 {
        let artifact = artifacts.iter().find(|record| record.filename == format!("many/nested/{index}.log")).unwrap();
        assert_eq!(artifact.scope.test_execution_id.as_ref(), Some(&execution));
        assert_eq!(artifact.status, dokime_runtime::artifacts::ArtifactStatus::Complete);
        let content = bench.handle.read_artifact_content(&artifact.artifact_id, 1024).await.unwrap().unwrap();
        assert_eq!(content.bytes, b"baseline\nchanged\n");
    }
    *native.host.watch_runtime.lock().unwrap() = None;
    native.shutdown().await;
    bench.stop().await;
}

/* === Tests === */
