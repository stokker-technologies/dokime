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
use std::sync::Arc;
use std::time::Instant;

// Third Party Imports
use harmos::operations::{Context, Operation, Response};

// Local Imports
use super::{FileWatch, WatchEvent};
use crate::{Error, RuntimeHandle};
use dokime_contract::operations::files::{
    FileWatchGap, WatchFile as Declaration, WatchFileOutcome, WatchFileRequest, WatchedArtifact,
};

/* === Constants === */
// The owner the step built-ins are registered under.
const OWNER: &str = "dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Resources {
    runtime: RuntimeHandle,
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

    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> crate::Result<Self> {
        Ok(Self {
            resources,
            watch: None,
        })
    }

    async fn run<'a>(
        &mut self,
        context: Self::Context<'a>,
        request: &mut WatchFileRequest,
        _: &mut (),
    ) -> crate::Result<Response<WatchFileOutcome>> {
        // A Step's Call Carries Its Execution
        let scoped = context
            .context()
            .value::<dokime::execution::ExecutionScope>()
            .ok()
            .flatten()
            .map(|scope| scope.execution);
        let watch = self
            .resources
            .runtime
            .open_watch_file(request, scoped.as_deref())
            .await?;
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
            let remaining = watch
                .deadline()
                .map(|deadline| deadline.saturating_sub(started.elapsed()));
            if remaining.is_some_and(|remaining| remaining.is_zero()) {
                break;
            }
            let pause = remaining.map_or(watch.poll_interval(), |remaining| {
                remaining.min(watch.poll_interval())
            });
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
            .map(|record| WatchedArtifact {
                artifact: record.filename.clone(),
                size_bytes: record.size_bytes,
            })
            .collect();
        self.watch = None;
        Ok(WatchFileOutcome { artifacts, gaps }.into())
    }

    // A cancelled or failed watch still seals what it captured.
    async fn stop(&mut self) -> crate::Result<()> {
        let Some(mut watch) = self.watch.take() else {
            return Ok(());
        };
        watch.seal(&mut Vec::new()).await
    }
}

/* === Public Functions === */
// Registers the step built-ins: the Harmos instance controls and `WatchFile`.
pub fn register(runtime: &RuntimeHandle) -> crate::Result<()> {
    harmos::builtin::Catalog::register(runtime.operations(), OWNER, crate::app::Lanes::Open)?;
    runtime.operations().register_wire_operation::<WatchFile>(
        OWNER,
        crate::app::Lanes::Open,
        Arc::new(Resources {
            runtime: runtime.clone(),
        }),
    )?;
    Ok(())
}

/* === Private Functions === */
fn collect_gaps(events: &mut Vec<WatchEvent>, gaps: &mut Vec<FileWatchGap>) {
    for event in events.drain(..) {
        if let WatchEvent::Gap {
            path,
            offset,
            reason,
        } = event
        {
            gaps.push(FileWatchGap {
                path,
                offset,
                reason,
            });
        }
    }
}

/* === Tests === */
