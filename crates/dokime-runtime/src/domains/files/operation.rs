/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// `WatchFile`, the step's file watch: one file watched for the calling step's
// execution until the instance is cancelled, the bound elapses or the
// execution seals. It settles with what it sealed and the gaps it saw; a
// cancelled watch seals what it captured through `stop`.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

// Third Party Imports
use harmos::operations::{Context, Operation, OperationManager, Response};

// Local Imports
use super::{FileWatch, WatchEvent};
use crate::{RuntimeHandle, runtime::Error};
use dokime_domain::model::files::wire::FileWatchArmed;
use dokime_domain::model::files::wire::FileWatchGap;
use dokime_domain::model::files::wire::WatchFileOutcome;
use dokime_domain::model::files::wire::WatchedArtifact;
use dokime_operations::files::AwaitFileWatchArmed;
use dokime_operations::files::WatchFile;

/* === Constants === */

// The owner the step built-ins are registered under.
const OWNER: &str = "dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Registered before any plugin attaches, bound to the runtime once it exists.
pub struct Resources {
    runtime: OnceLock<RuntimeHandle>,
    readiness: Mutex<BTreeMap<u64, Ready>>,
    changed: tokio::sync::Notify,
}

struct Ready {
    execution: Option<String>,
    signal: tokio::sync::watch::Sender<Option<Result<FileWatchArmed, String>>>,
}

struct Awaiting {
    resources: Arc<Resources>,
}

struct Watching {
    resources: Arc<Resources>,
    watch: Option<FileWatch>,
    ready: tokio::sync::watch::Sender<Option<Result<FileWatchArmed, String>>>,
    id: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Operation for Watching {
    type Declaration = WatchFile;
    type Resources = Resources;
    type Error = Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, context: Self::Context<'ctx>) -> crate::runtime::Result<Self> {
        let execution = context.context().value::<crate::execution::ExecutionScope>().ok().flatten().map(|scope| scope.execution);
        let (ready, _) = tokio::sync::watch::channel(None);
        {
            let mut entries = resources.readiness.lock().expect("watch readiness lock is healthy");
            if entries.len() >= 256 {
                return Err(super::capped("the configured file-watch readiness capacity is full"));
            }
            entries.insert(context.id().value(), Ready { execution, signal: ready.clone() });
        }
        resources.changed.notify_waiters();
        Ok(Self { resources, watch: None, ready, id: context.id().value() })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut WatchFile, _: &mut (),
    ) -> crate::runtime::Result<Response<WatchFileOutcome>> {
        // A Step's Call Carries Its Execution
        let scoped = context.context().value::<crate::execution::ExecutionScope>().ok().flatten().map(|scope| scope.execution);
        let runtime =
            self.resources.runtime.get().ok_or_else(|| Error::unavailable(super::UNREACHABLE, "the runtime is still starting"))?;
        let watch = runtime.open_watch_file(&*request, scoped.as_deref()).await?;
        let watch = self.watch.insert(watch);

        // Poll Until Complete, Bounded, Or Sealed By The Execution
        let mut events = Vec::new();
        let mut gaps = Vec::new();
        loop {
            let polled = watch.poll(&mut events).await;
            if let Err(source) = polled {
                self.ready.send_replace(Some(Err(source.to_string())));
                let sealed = watch.seal(&mut events).await;
                self.watch = None;
                if let Err(seal) = sealed {
                    tracing::warn!(error = %seal, "failed file watch did not seal every file");
                }
                return Err(source);
            }
            for event in &events {
                if let WatchEvent::Armed { baseline } = event {
                    self.ready.send_replace(Some(Ok(FileWatchArmed { baseline: baseline.clone() })));
                }
            }
            collect_gaps(&mut events, &mut gaps);
            if watch.is_complete() {
                break;
            }
            let remaining = watch.remaining();
            if remaining.is_some_and(|remaining| remaining.is_zero()) {
                break;
            }
            let pause = remaining.map_or(watch.poll_interval(), |remaining| remaining.min(watch.poll_interval()));
            tokio::select! {
                () = watch.seal_requested() => break,
                () = tokio::time::sleep(pause) => {}
            }
        }

        // Seal, Then Answer With Every Sealed Artifact
        let sealed = watch.seal(&mut events).await;
        collect_gaps(&mut events, &mut gaps);
        sealed?;
        let artifacts = watch
            .sealed()
            .iter()
            .map(|record| WatchedArtifact { artifact: record.filename.clone(), size_bytes: record.size_bytes })
            .collect();
        self.watch = None;
        Ok(WatchFileOutcome { artifacts, gaps }.into())
    }

    // A cancelled or failed watch still seals what it captured.
    async fn stop(&mut self) -> crate::runtime::Result<()> {
        let Some(mut watch) = self.watch.take() else {
            return Ok(());
        };
        watch.seal(&mut Vec::new()).await
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        if self.ready.borrow().is_none() {
            self.ready.send_replace(Some(Err("file watch stopped before capture was armed; inspect its outcome".to_owned())));
        }
        let mut entries = self.resources.readiness.lock().expect("watch readiness lock is healthy");
        entries.remove(&self.id);
        drop(entries);
        self.resources.changed.notify_waiters();
    }
}

impl Operation for Awaiting {
    type Declaration = AwaitFileWatchArmed;
    type Resources = Resources;
    type Error = Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> crate::runtime::Result<Self> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut AwaitFileWatchArmed, _: &mut (),
    ) -> crate::runtime::Result<Response<FileWatchArmed>> {
        let execution = context.context().value::<crate::execution::ExecutionScope>().ok().flatten().map(|scope| scope.execution);
        let readiness = async {
            let registration = async {
                loop {
                    let changed = self.resources.changed.notified();
                    {
                        let entries = self.resources.readiness.lock().expect("watch readiness lock is healthy");
                        if let Some(entry) = entries.get(&request.instance_id) {
                            if entry.execution != execution {
                                return Err(super::invalid("file watch belongs to another execution"));
                            }
                            return Ok(entry.signal.subscribe());
                        }
                    }
                    let runtime = self.resources.runtime.get().ok_or_else(|| super::invalid("runtime is not ready"))?;
                    let id = harmos::operations::OperationInstanceId::new(request.instance_id);
                    let status = runtime.operations().status(id);
                    if status.is_none_or(|status| status.phase().is_terminal() || status.cancellation_requested()) {
                        return Err(Error::conflict(super::UNREACHABLE, "file watch is no longer active; inspect its outcome"));
                    }
                    changed.await;
                }
            };
            let mut signal = tokio::time::timeout(std::time::Duration::from_secs(30), registration)
                .await
                .map_err(|_| super::invalid("file watch instance was not registered"))??;
            loop {
                let value = signal.borrow_and_update().clone();
                if let Some(value) = value {
                    return value.map(Response::new).map_err(|message| Error::conflict(super::UNREACHABLE, message));
                }
                signal.changed().await.map_err(|_| Error::conflict(super::UNREACHABLE, "file watch closed before arming"))?;
            }
        };
        tokio::time::timeout(std::time::Duration::from_secs(86_400), readiness)
            .await
            .map_err(|_| Error::conflict(super::UNREACHABLE, "file watch readiness deadline elapsed"))?
    }
}

impl Resources {
    pub fn bind(&self, runtime: &RuntimeHandle) {
        let _ = self.runtime.set(runtime.clone());
    }
}

/* === Public Functions === */

// Registers the step built-ins, the Harmos instance controls and `WatchFile`,
// before any plugin that requires them attaches.
pub fn register(operations: &OperationManager) -> harmos::Result<Arc<Resources>> {
    harmos::builtin::Catalog::register(operations, OWNER, crate::app::Lanes::Open)?;
    let resources =
        Arc::new(Resources { runtime: OnceLock::new(), readiness: Mutex::new(BTreeMap::new()), changed: tokio::sync::Notify::new() });
    operations.register_wire_operation::<Watching>(OWNER, crate::app::Lanes::Open, Arc::clone(&resources))?;
    operations.register_wire_operation::<Awaiting>(OWNER, crate::app::Lanes::Open, Arc::clone(&resources))?;
    Ok(resources)
}

/* === Private Functions === */

fn collect_gaps(events: &mut Vec<WatchEvent>, gaps: &mut Vec<FileWatchGap>) {
    for event in events.drain(..) {
        if let WatchEvent::Gap { path, offset, reason } = event {
            gaps.push(FileWatchGap { path, offset, reason });
        }
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settling_releases_registration_and_preserves_the_subscribed_outcome() {
        let state = Resources { runtime: OnceLock::new(), readiness: Mutex::new(BTreeMap::new()), changed: tokio::sync::Notify::new() };
        let resources = Arc::new(state);
        let outcomes = [None, Some(Ok(FileWatchArmed { baseline: Vec::new() })), Some(Err("source refused".to_owned()))];
        for outcome in outcomes {
            let (ready, signal) = tokio::sync::watch::channel(outcome.clone());
            let entry = Ready { execution: Some("execution".to_owned()), signal: ready.clone() };
            resources.readiness.lock().unwrap().insert(1, entry);
            let watching = Watching { resources: resources.clone(), watch: None, ready, id: 1 };

            drop(watching);

            assert!(resources.readiness.lock().unwrap().is_empty());
            let received = signal.borrow().clone().expect("every settled watcher has a readiness outcome");
            match outcome {
                Some(expected) => assert_eq!(received, expected),
                None => assert!(received.unwrap_err().contains("stopped before capture was armed")),
            }
            assert!(signal.has_changed().is_err(), "settlement releases every readiness sender");
        }
    }
}
