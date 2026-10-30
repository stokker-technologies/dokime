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
use harmos::Context;

// Local Imports
use super::{FileWatch, WatchEvent};
use crate::app::Lanes;
use crate::operations::hosting::{Host, Serve, Served};
use crate::runtime::operations::reports::status;
use crate::{RuntimeHandle, runtime::Error};
use dokime_domain::model::files::wire::FileWatchArmed;
use dokime_domain::model::files::wire::FileWatchGap;
use dokime_domain::model::files::wire::WatchFileOutcome;
use dokime_domain::model::files::wire::WatchedArtifact;
use dokime_operations::files::AwaitFileWatchArmed;
use dokime_operations::files::WatchFile;

/* === Constants === */

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

// The host side of both operations.
struct Files(Arc<Resources>);

// One running watch: its readiness registration, and the watch a cancelled
// attempt still seals when this drops.
struct Watching {
    resources: Arc<Resources>,
    watch: Option<FileWatch>,
    ready: tokio::sync::watch::Sender<Option<Result<FileWatchArmed, String>>>,
    ticket: Option<u64>,
}

/* === Enums === */

#[harmos::catalog]
enum PreparedOperations {
    Operation0(WatchFile),
    Operation1(AwaitFileWatchArmed),
}

/* === Traits === */

/* === Implementations === */

impl Watching {
    fn register(resources: &Arc<Resources>, ctx: &Context<'_, WatchFile>) -> Result<Self, Error> {
        let execution = execution_of(ctx.call());
        let ticket = ctx.call().value::<crate::execution::WatchTicket>().ok().flatten().map(|ticket| ticket.ticket);
        let (ready, _) = tokio::sync::watch::channel(None);
        if let Some(ticket) = ticket {
            let mut entries = resources.readiness.lock().expect("watch readiness lock is healthy");
            if entries.len() >= 256 {
                return Err(super::capped("the configured file-watch readiness capacity is full"));
            }
            entries.insert(ticket, Ready { execution, signal: ready.clone() });
            drop(entries);
            resources.changed.notify_waiters();
        }
        Ok(Self {
            resources: Arc::clone(resources),
            watch: None,
            ready,
            ticket,
        })
    }

    async fn run(&mut self, request: WatchFile, scoped: Option<&str>, ctx: &Context<'_, WatchFile>) -> Result<WatchFileOutcome, Error> {
        let runtime =
            self.resources.runtime.get().ok_or_else(|| Error::unavailable(super::UNREACHABLE, "the runtime is still starting"))?;
        let watch = tokio::select! {
            result = runtime.open_watch_file(&request, scoped) => result?,
            () = ctx.cancelled() => return Err(Error::cancelled("file_watch_cancelled", "file watch was cancelled before it opened")),
        };
        let watch = self.watch.insert(watch);

        // Poll Until Complete, Bounded, Or Sealed By The Execution
        let mut events = Vec::new();
        let mut gaps = Vec::new();
        loop {
            let polled = tokio::select! {
                result = watch.poll(&mut events) => result,
                () = ctx.cancelled() => break,
            };
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
                () = ctx.cancelled() => break,
                () = tokio::time::sleep(pause) => {}
            }
        }

        // Seal, Then Answer With Every Sealed Artifact
        let sealed = watch.seal(&mut events).await;
        collect_gaps(&mut events, &mut gaps);
        sealed?;
        if ctx.cancellation_requested() && self.ready.borrow().is_none() {
            self.watch = None;
            return Err(Error::cancelled("file_watch_cancelled", "file watch was cancelled before capture was armed"));
        }
        let artifacts = watch
            .sealed()
            .iter()
            .map(|record| WatchedArtifact {
                artifact: record.filename.clone(),
                size_bytes: record.size_bytes,
            })
            .collect();
        self.watch = None;
        Ok(WatchFileOutcome { artifacts, gaps })
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        if self.ready.borrow().is_none() {
            self.ready.send_replace(Some(Err("file watch stopped before capture was armed; inspect its outcome".to_owned())));
        }
        if let Some(ticket) = self.ticket {
            let mut entries = self.resources.readiness.lock().expect("watch readiness lock is healthy");
            entries.remove(&ticket);
            drop(entries);
            self.resources.changed.notify_waiters();
        }
        // A cancelled or failed watch still seals what it captured, owned by the runtime so shutdown drains it.
        if let (Some(mut watch), Some(runtime)) = (self.watch.take(), self.resources.runtime.get()) {
            runtime.owned_work.spawn(async move {
                if let Err(error) = watch.seal(&mut Vec::new()).await {
                    tracing::warn!(error = %error, "cancelled file watch did not seal every file");
                }
            });
        }
    }
}

impl Serve<WatchFile> for Files {
    fn serve<'a>(&'a self, request: WatchFile, ctx: Context<'a, WatchFile>) -> Served<'a, WatchFileOutcome> {
        Box::pin(async move {
            // A Step's Call Carries Its Execution
            let scoped = execution_of(ctx.call());
            let mut watching = Watching::register(&self.0, &ctx).map_err(status)?;
            watching.run(request, scoped.as_deref(), &ctx).await.map(Into::into).map_err(status)
        })
    }
}

impl Serve<AwaitFileWatchArmed> for Files {
    fn serve<'a>(&'a self, request: AwaitFileWatchArmed, ctx: Context<'a, AwaitFileWatchArmed>) -> Served<'a, FileWatchArmed> {
        Box::pin(async move {
            let execution = execution_of(ctx.call());
            self.0.armed(request.instance_id, execution).await.map(Into::into).map_err(status)
        })
    }
}

impl Resources {
    pub fn bind(&self, runtime: &RuntimeHandle) {
        let _ = self.runtime.set(runtime.clone());
    }

    async fn armed(&self, ticket: u64, execution: Option<String>) -> Result<FileWatchArmed, Error> {
        let readiness = async {
            let registration = async {
                loop {
                    let changed = self.changed.notified();
                    {
                        let entries = self.readiness.lock().expect("watch readiness lock is healthy");
                        if let Some(entry) = entries.get(&ticket) {
                            if entry.execution != execution {
                                return Err(super::invalid("file watch belongs to another execution"));
                            }
                            return Ok(entry.signal.subscribe());
                        }
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
                    return value.map_err(|message| Error::conflict(super::UNREACHABLE, message));
                }
                signal.changed().await.map_err(|_| Error::conflict(super::UNREACHABLE, "file watch closed before arming"))?;
            }
        };
        tokio::time::timeout(std::time::Duration::from_secs(86_400), readiness)
            .await
            .map_err(|_| Error::conflict(super::UNREACHABLE, "file watch readiness deadline elapsed"))?
    }
}

/* === Public Functions === */

// Registers the step built-ins, the Harmos instance controls, `WatchFile` and
// `AwaitFileWatchArmed`, before any plugin that requires them attaches.
pub fn prepare(setup: &mut harmos::Setup<crate::app::Dokime>) -> harmos::Result<Arc<Resources>> {
    setup.operations::<harmos::builtins::Lifecycle>(Lanes::Open);
    let resources = Arc::new(Resources {
        runtime: OnceLock::new(),
        readiness: Mutex::new(BTreeMap::new()),
        changed: tokio::sync::Notify::new(),
    });
    let files = Arc::new(Files(Arc::clone(&resources)));

    setup.resource::<Host<WatchFile>>(Host::<WatchFile>::new(files.clone()));
    setup.resource::<Host<AwaitFileWatchArmed>>(Host::<AwaitFileWatchArmed>::new(files));

    setup.operations::<PreparedOperations>(Lanes::Open);
    Ok(resources)
}

/* === Private Functions === */

fn execution_of(call: &harmos::operations::CallContext) -> Option<String> {
    call.value::<crate::execution::ExecutionScope>().ok().flatten().map(|scope| scope.execution)
}

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
        let state = Resources {
            runtime: OnceLock::new(),
            readiness: Mutex::new(BTreeMap::new()),
            changed: tokio::sync::Notify::new(),
        };
        let resources = Arc::new(state);
        let outcomes = [None, Some(Ok(FileWatchArmed { baseline: Vec::new() })), Some(Err("source refused".to_owned()))];
        for outcome in outcomes {
            let (ready, signal) = tokio::sync::watch::channel(outcome.clone());
            let entry = Ready {
                execution: Some("execution".to_owned()),
                signal: ready.clone(),
            };
            resources.readiness.lock().unwrap().insert(1, entry);
            let watching = Watching {
                resources: resources.clone(),
                watch: None,
                ready,
                ticket: Some(1),
            };

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
