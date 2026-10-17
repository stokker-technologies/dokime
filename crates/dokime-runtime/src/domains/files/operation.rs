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
use std::sync::{Arc, OnceLock};
use std::time::Instant;

// Third Party Imports
use harmos::operations::{Context, Operation, OperationManager, Response};

// Local Imports
use super::{FileWatch, WatchEvent};
use crate::{RuntimeHandle, runtime::Error};
use dokime_domain::model::files::wire::FileWatchGap;
use dokime_domain::model::files::wire::WatchFileOutcome;
use dokime_domain::model::files::wire::WatchedArtifact;
use dokime_domain::operations::files::WatchFile as Declaration;
use dokime_domain::operations::files::WatchFile;

/* === Constants === */
// The owner the step built-ins are registered under.
const OWNER: &str = "dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Registered before any plugin attaches, bound to the runtime once it exists.
pub struct Resources {
    runtime: OnceLock<RuntimeHandle>,
}

struct WatchFile {
    resources: Arc<Resources>,
    watch: Option<FileWatch>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Operation for WatchFile {
    type Declaration = Declaration;
    type Resources = Resources;
    type Error = Error;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> crate::runtime::Result<Self> {
        Ok(Self { resources, watch: None })
    }

    async fn run<'a>(
        &mut self, context: Self::Context<'a>, request: &mut WatchFile, _: &mut (),
    ) -> crate::runtime::Result<Response<WatchFileOutcome>> {
        // A Step's Call Carries Its Execution
        let scoped = context.context().value::<crate::execution::ExecutionScope>().ok().flatten().map(|scope| scope.execution);
        let runtime =
            self.resources.runtime.get().ok_or_else(|| Error::unavailable(super::UNREACHABLE, "the runtime is still starting"))?;
        let watch = runtime.open_watch_file(request, scoped.as_deref()).await?;
        let watch = self.watch.insert(watch);
        let started = Instant::now();

        // Poll Until Complete, Bounded, Or Sealed By The Execution
        let mut events = Vec::new();
        let mut gaps = Vec::new();
        loop {
            let polled = watch.poll(&mut events).await;
            collect_gaps(&mut events, &mut gaps);
            if let Err(source) = polled {
                let sealed = watch.seal(&mut events).await;
                self.watch = None;
                if let Err(seal) = sealed {
                    tracing::warn!(error = %seal, "failed file watch did not seal every file");
                }
                return Err(source);
            }
            if watch.is_complete() {
                break;
            }
            let remaining = watch.deadline().map(|deadline| deadline.saturating_sub(started.elapsed()));
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
    let resources = Arc::new(Resources { runtime: OnceLock::new() });
    operations.register_wire_operation::<WatchFile>(OWNER, crate::app::Lanes::Open, Arc::clone(&resources))?;
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
