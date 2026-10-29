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
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
// Third Party Imports
use tokio::sync::Notify;
// Local Imports
use super::*;
use dokime_runtime::execution::{CallbackTask, Callbacks, TemplateDefinition};

/* === Constants === */
static DRIVEN: Mutex<Option<Arc<Loop>>> = Mutex::new(None);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Foreign;

struct Resource(Arc<Loop>);

struct Loop {
    stage: &'static str,
    registered: Notify,
    cancelled: Notify,
    release: Notify,
    cancelling: tokio::sync::watch::Sender<bool>,
    settled: AtomicBool,
    events: Mutex<Vec<&'static str>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Loop {
    fn new(stage: &'static str) -> Self {
        Self {
            stage,
            registered: Notify::new(),
            cancelled: Notify::new(),
            release: Notify::new(),
            cancelling: tokio::sync::watch::channel(false).0,
            settled: AtomicBool::new(false),
            events: Mutex::new(Vec::new()),
        }
    }

    async fn callback(self: &Arc<Self>, scope: &Callbacks) -> Result<(), Diagnostic> {
        let mut cancelling = self.cancelling.subscribe();
        scope.register(self.clone());
        self.registered.notify_one();

        cancelling.wait_for(|cancelled| *cancelled).await.unwrap();
        Err(Diagnostic::of("foreign callback cancelled; finalizer remains owned"))
    }
}

impl CallbackTask for Loop {
    fn is_settled(&self) -> bool {
        self.settled.load(Ordering::Acquire)
    }

    fn cancel(&self) {
        self.cancelling.send_replace(true);
        self.cancelled.notify_one();
    }

    fn join(&self) -> Pin<Box<dyn Future<Output = Result<(), Diagnostic>> + Send + '_>> {
        Box::pin(async move {
            self.release.notified().await;
            self.events.lock().unwrap().push("joined");
            self.settled.store(true, Ordering::Release);

            if self.stage == "failure" { Err(Diagnostic::of("foreign finalizer failed")) } else { Ok(()) }
        })
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        self.0.events.lock().unwrap().push("released");
    }
}

impl SidecarTemplate for Foreign {
    const RUN: &'static str = "probe.checks.pulse.run";
    const SIGNAL: &'static str = "probe.checks.pulse.signal";

    type Own = Resource;
    type Parameters = ();

    fn definition() -> TemplateDefinition {
        TemplateDefinition {
            template: Probe::declaration(),
            records: Vec::new(),
            streams: Vec::new(),
        }
    }

    fn initialize<'a>(ctx: &'a mut InitializationContext<'_, Self, Self::Own>) -> InitializationFuture<'a, (Self, Self::Own)> {
        Box::pin(async move {
            if driven().stage == "initialization" {
                driven().callback(ctx.environment().callbacks()).await?;
            }
            Ok((Self, Resource(driven())))
        })
    }

    fn run_step<'a>(&'a mut self, _: &'a str, ctx: &'a mut StepContext<'_, Self, Self::Own>) -> StepFuture<'a> {
        Box::pin(async move {
            if driven().stage == "step" {
                driven().callback(ctx.environment().callbacks()).await?;
            }

            Ok(())
        })
    }

    fn cleanup<'a>(&'a mut self, ctx: &'a mut CleanupContext<'_, Self, Self::Own>) -> StepFuture<'a> {
        Box::pin(async move {
            driven().events.lock().unwrap().push("cleanup");
            if driven().stage == "cleanup" {
                driven().callback(ctx.environment().callbacks()).await?;
            }

            Ok(())
        })
    }

    fn evaluate<'a>(
        _: &'a str, ctx: &'a CriterionContext<'_, Self, Self::Parameters>,
    ) -> dokime_runtime::execution::EvaluationFuture<'a, bool> {
        Box::pin(async move {
            let driving = driven();
            if driving.stage == "criterion" {
                driving.callback(ctx.callbacks()).await?;
            }

            Ok(Some(true))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn driven() -> Arc<Loop> {
    DRIVEN.lock().unwrap().clone().expect("a case names the loop it drives before it starts the workflow")
}

pub(crate) async fn shutdown_joins_owners() {
    for (stage, expected) in [
        ("initialization", vec!["joined"]),
        ("step", vec!["joined", "cleanup", "released"]),
        ("cleanup", vec!["cleanup", "joined", "released"]),
        ("criterion", vec!["cleanup", "released", "joined"]),
    ] {
        let foreign = Arc::new(Loop::new(stage));
        *DRIVEN.lock().unwrap() = Some(Arc::clone(&foreign));
        let workflow = Workflow::<Foreign>::new(WORKFLOW, SOURCE);
        workflow.start(start_request(stage, serde_json::json!({}))).unwrap();
        let mut cursor = 0;
        // Drains between waits admit the steps that register the callback.
        loop {
            tokio::select! {
                _ = foreign.registered.notified() => break,
                batch = next(&workflow, stage, cursor) => cursor = batch.cursor,
            }
        }

        let stopping = workflow.clone();
        let shutdown = tokio::spawn(async move { stopping.shutdown().await });
        foreign.cancelled.notified().await;
        // Advance past the old whole-run abort deadline, without allowing the
        // foreign finalizer to finish. Neither shutdown nor resource release
        // may overtake that finalizer.
        tokio::time::advance(Duration::from_secs(3)).await;
        assert!(!shutdown.is_finished(), "{stage}: shutdown abandoned its callback");
        // A cursor past everything buffered reads settlement without taking
        // the retained outcome.
        let pending = workflow.drain(stage, u64::MAX).unwrap();
        assert!(!pending.cleanup_settled, "{stage}: finalizer is still held");
        assert!(pending.outcome.is_none());
        assert_eq!(pending.cursor, u64::MAX);

        let before_join = foreign.events.lock().unwrap().clone();
        assert_eq!(
            before_join,
            match stage {
                "cleanup" => vec!["cleanup"],
                "criterion" => vec!["cleanup", "released"],
                _ => vec![],
            },
            "{stage}: resources released before callback joined",
        );

        foreign.release.notify_one();
        shutdown.await.unwrap();
        assert_eq!(*foreign.events.lock().unwrap(), expected, "{stage}");
        // The terminal notification already happened before this waiter was
        // created. Cleanup state must still make the wait immediately readable,
        // even when its cursor deliberately skips every buffered event.
        let settled = workflow.next(stage, u64::MAX).await.unwrap();
        assert!(settled.cleanup_settled, "{stage}: original owners joined");
        assert!(settled.outcome.is_none());
        assert_eq!(settled.cursor, u64::MAX);
        // Status reads do not acknowledge the retained outcome below.
        let mut cursor = 0;
        let terminal = loop {
            let reply = workflow.drain(stage, cursor).unwrap();
            if let Some(outcome) = reply.outcome {
                break outcome;
            }
            assert!(reply.cursor > cursor, "{stage}: shutdown lost the terminal outcome");
            cursor = reply.cursor;
        };
        assert!(!terminal.passed, "{stage}: cancellation or timeout must not pass");
    }
}

pub(crate) async fn interrupted_join_teardown() {
    let foreign = Arc::new(Loop::new("failure"));
    let other = Arc::new(Loop::new("partial"));
    let callbacks = Callbacks::default();
    callbacks.register(foreign.clone());
    callbacks.register(other.clone());
    let mut settling = Box::pin(callbacks.settle());
    tokio::select! {
        _ = &mut settling => panic!("the foreign callback is still running"),
        _ = foreign.cancelled.notified() => {},
    }
    drop(settling);
    assert!(!foreign.is_settled());

    foreign.release.notify_one();
    other.release.notify_one();
    let failures = callbacks.settle().await;
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].message, "foreign finalizer failed");
    assert_eq!(*foreign.events.lock().unwrap(), ["joined"]);
    assert_eq!(*other.events.lock().unwrap(), ["joined"]);
}

/* === Tests === */
