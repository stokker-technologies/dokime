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

// Third Party Imports

// Local Imports
use dokime_runtime::execution::ExecutionScope;

use harmos::operations::OperationDeclaration;

use std::sync::atomic::{AtomicUsize, Ordering};

use std::time::Duration;

/* === Constants === */

static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::operation(resources = [harmos::operations::HostContext], id = "invocation.evidence", response = u64,
    replay = harmos::operations::ReplaySafety::Idempotent,
    policy = harmos::OperationPolicy { timeout: Duration::from_secs(5), attempt_timeout: Duration::from_secs(1),
        cancellation_timeout: Duration::ZERO, retries: harmos::RetryPolicy::Never })]
struct Read {}

/* === Enums === */

#[harmos::catalog]
enum Operations {
    Read(Read),
}

/* === Traits === */

/* === Implementations === */

impl harmos::Operation for Read {
    type Error = std::io::Error;
    async fn run(self, ctx: harmos::Context<'_, Self>) -> Result<harmos::Response<u64>, Self::Error> {
        ATTEMPTS.fetch_add(1, Ordering::SeqCst);
        if ctx.resource::<harmos::operations::HostContext>().borrow().attempt() == 0 {
            return Err(std::io::Error::other("transient read failure"));
        }
        Ok(42.into())
    }
    fn classify(_: &Self::Error) -> harmos::operations::FailureKind {
        harmos::operations::FailureKind::Transient
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn retained_attempts_complete_while_persistence_is_blocked() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../src/report/fixtures/execution-report-input.json")).unwrap();
    let mut cycle: dokime_domain::model::execution::result::CycleView = serde_json::from_value(fixture["input"]["cycle"].clone()).unwrap();
    cycle.test_executions[0].saved = None;
    let execution = &cycle.test_executions[0];
    let root = tempfile::tempdir().unwrap();
    let mut owner = dokime_runtime::Runtime::start(dokime_runtime::RuntimeConfig::new(root.path(), cycle.metadata.clone())).await.unwrap();
    let runtime = owner.handle();
    dokime_runtime::testing::ExecutionOutputs::new(&runtime).acquire(&cycle, execution).await.unwrap();
    runtime.operations().register::<Operations>(dokime_runtime::app::Lanes::Open).await.unwrap();
    let token = execution.test_execution_id.to_string();
    let mut context = runtime.retained_attempt_context(execution.clone(), "invocation.evidence");
    context.insert_portable(ExecutionScope::execution(token.clone()).staged("step/read")).unwrap();
    let configured = harmos::RetryPolicy::OnTransient {
        max_retries: 1,
        backoff: harmos::Backoff::Fixed {
            delay: Duration::from_millis(1),
            jitter: harmos::Jitter::None,
        },
    };
    let operations = runtime.operations().with_context(context.clone());

    // Hold Persistence Beyond the Operation Deadline.
    let release = runtime.hold_retained_attempt_persistence(&token).await;
    tokio::time::pause();
    let answer = tokio::time::timeout(Duration::from_secs(1), operations.operation::<Read>(Read {}).retry(configured)).await;
    assert_eq!(*answer.unwrap().unwrap(), 42);
    tokio::time::advance(Duration::from_secs(6)).await;
    assert!(operations.operation::<Read>(Read {}).await.is_err());
    let before = ATTEMPTS.load(Ordering::SeqCst);
    let tampered = context.with_value(ExecutionScope::execution("another-execution")).unwrap();
    assert!(matches!(runtime.operations().with_context(tampered).operation::<Read>(Read {}).await, Err(harmos::Error::PermissionDenied)));
    assert_eq!(ATTEMPTS.load(Ordering::SeqCst), before);
    let mut unowned = harmos::operations::CallContext::default();
    unowned.insert_portable(ExecutionScope::execution(token.clone()).staged("step/forged")).unwrap();
    assert!(runtime.operations().with_context(unowned).operation::<Read>(Read {}).await.is_err());

    // Resume Persistence Before Reading the Retained Attempts.
    tokio::time::resume();
    release.send(()).unwrap();
    runtime.flush_retained_attempts(&token).await;
    let content = dokime_runtime::testing::ExecutionOutputs::new(&runtime)
        .native_read(&execution.test_execution_id, "artifacts/execution.log", 0, 8192)
        .await
        .unwrap()
        .unwrap();
    let text = std::str::from_utf8(&content.bytes).unwrap();
    let entries =
        text.lines().map(|line| serde_json::from_str::<serde_json::Value>(&line[line.find('{').unwrap()..]).unwrap()).collect::<Vec<_>>();
    let attempts = entries.iter().map(|entry| (entry["attempt"].as_u64().unwrap(), entry["outcome"].as_str().unwrap())).collect::<Vec<_>>();
    assert_eq!(attempts, [(0, "failed"), (1, "succeeded"), (0, "failed")]);
    assert!(entries.iter().all(|entry| entry["stage"] == "step/read" && entry["operation"] == Read::ID));
    assert_eq!(entries[0]["instance"], entries[1]["instance"]);
    assert_ne!(entries[1]["instance"], entries[2]["instance"]);

    // Repeated Flushes Must Not Duplicate Attempts.
    runtime.flush_retained_attempts(&token).await;
    let repeated = dokime_runtime::testing::ExecutionOutputs::new(&runtime)
        .native_read(&execution.test_execution_id, "artifacts/execution.log", 0, 8192)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(content.bytes, repeated.bytes);
    drop(repeated);
    drop(content);
    runtime.remove_retained_attempt(&token);
    drop(runtime);
    owner.shutdown().await.unwrap();
}
