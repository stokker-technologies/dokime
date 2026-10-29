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
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};

// Third Party Imports
use tokio::sync::{oneshot, watch};
use tracing::{Instrument, instrument::WithSubscriber};

// Local Imports
use super::log::Level;
use super::tick::Cadence;
use super::unwind::caught;
use super::{DeclaredObserver, Environment, SidecarTemplate};
use crate::artifacts::Artifacts;
use crate::error::PublishFailure;
use crate::plugin::Diagnostic;
use crate::records::Deliveries;
use crate::telemetry::Streams;
use crate::transport::buffer::{BufferedFrame, Executions, Tasks, lock};
use crate::transport::frame::FrameTransport;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Scope<O, I> {
    pub executions: Executions,
    pub token: String,
    pub source: String,
    pub caller: super::Caller,
    pub binding: Option<Arc<dyn std::any::Any + Send + Sync>>,
    pub own: Arc<O>,
    pub parameters: Arc<I>,
    pub raw: Arc<BTreeMap<String, Value>>,
    pub bindings: Arc<crate::plugin::Configuration>,
    pub artifacts: Artifacts,
    pub streams: Streams,
    pub deliveries: Deliveries,
    pub document: Arc<Mutex<super::document::DocumentIndex>>,
}

pub(super) struct Observers {
    stop: watch::Sender<bool>,
    completions: Vec<oneshot::Receiver<()>>,
    publisher: Arc<ExecutionPublisher>,
}

pub(crate) struct ExecutionPublisher {
    executions: Executions,
    token: String,
    source: String,
    schemas: BTreeMap<String, u32>,
    // A publication and closing the grant serialize, so even a retained clone
    // cannot publish between settling the evidence and deciding criteria.
    closed: Mutex<bool>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionPublisher {
    pub(crate) fn record(&self, request: RecordPublishRequest) -> crate::Result<()> {
        let closed = self.closed.lock().unwrap_or_else(PoisonError::into_inner);
        if *closed {
            return Err(crate::Error::Publish(PublishFailure::Closed));
        }
        // Read rather than asserted: the stage's own `publish` reaches this
        // backend directly, so a record type nothing declared has to be refused
        // here rather than assumed away by whoever called.
        let Some(&version) = self.schemas.get(request.record_type.as_str()) else {
            return Err(crate::Error::Publish(PublishFailure::Undeclared {
                declared_as: request.record_type.to_string(),
            }));
        };
        let accepted =
            lock(&self.executions).get_mut(&self.token).is_some_and(|state| state.observer_record(request, &self.source, version));
        if accepted { Ok(()) } else { Err(crate::Error::Publish(PublishFailure::Closed)) }
    }

    fn close(&self) {
        *self.closed.lock().unwrap_or_else(PoisonError::into_inner) = true;
    }
}

impl Drop for Observers {
    fn drop(&mut self) {
        self.publisher.close();
        self.stop.send_replace(true);
    }
}

/* === Public Functions === */

pub(super) fn start<T: SidecarTemplate>(
    scope: Scope<T::Own, T::Parameters>, declared: Vec<DeclaredObserver>, records: &[crate::records::RecordContract], tasks: &Tasks,
) -> Observers {
    let publisher = Arc::new(ExecutionPublisher {
        executions: Arc::clone(&scope.executions),
        token: scope.token.clone(),
        source: scope.source.clone(),
        // The declared record set, as its schema versions: an observer's stage
        // reaches this backend directly, so the admission a plugin publisher
        // performs against its `Declarations` is performed here instead.
        schemas: records.iter().map(|contract| (contract.record_type.to_string(), contract.schema_version)).collect(),
        closed: Mutex::new(false),
    });
    let (stop, listener) = watch::channel(false);
    let scope = Arc::new(scope);
    let mut tasks = tasks.lock().unwrap_or_else(PoisonError::into_inner);
    tasks.retain(|task| !task.is_finished());
    let completions = declared
        .into_iter()
        .map(|declared| {
            let (completed, completion) = oneshot::channel();
            let name = declared.identifier.to_string();
            // Opened here, inside the execution's own span, so a diagnostic
            // captured on the observer's task still names the run it belongs to.
            let span = tracing::info_span!("observer", observer = %declared.identifier);
            let work = watching::<T>(Arc::clone(&scope), declared, Arc::clone(&publisher), listener.clone())
                .instrument(span)
                .with_current_subscriber();
            match harmos::sidecar::__private::spawn_observer(name.clone(), async move {
                work.await;
                let _ = completed.send(());
            }) {
                Ok(task) => tasks.push(task),
                Err(error) => failed(
                    &scope.executions,
                    &scope.token,
                    &name,
                    Diagnostic::of(format!("could not start observer runtime thread: {error}")),
                ),
            }
            completion
        })
        .collect();

    Observers { stop, completions, publisher }
}

pub(super) async fn settle(mut observers: Observers) {
    observers.publisher.close();
    observers.stop.send_replace(true);
    for completion in observers.completions.drain(..) {
        let _ = completion.await;
    }
}

/* === Private Functions === */

async fn watching<T: SidecarTemplate>(
    scope: Arc<Scope<T::Own, T::Parameters>>, declared: DeclaredObserver, publisher: Arc<ExecutionPublisher>,
    mut stop: watch::Receiver<bool>,
) {
    if declared.interval.is_zero() {
        failed(&scope.executions, &scope.token, &declared.identifier, Diagnostic::of("an observer interval must be positive"));
        return;
    }

    let mut cadence = Cadence::every(declared.interval, stop.clone());
    while let Some(tick) = cadence.next().await {
        // One grant per tick, so what an observer holds never outlives the call
        // it was handed for — and the environment it reads is rebuilt from the
        // same shared values every stage was given.
        let environment = Environment::new(
            scope.parameters.as_ref(),
            scope.raw.as_ref(),
            scope.bindings.as_ref(),
            scope.artifacts.clone(),
            scope.streams.clone(),
            scope.deliveries.clone(),
        )
        .calling(scope.caller.clone())
        .declaring(Arc::clone(&scope.document))
        .holding(Arc::clone(&scope.own));
        let environment = match scope.binding.clone() {
            Some(binding) => environment.bound(binding),
            None => environment,
        };
        let mut transport = FrameTransport::new(Arc::clone(&scope.executions), scope.token.clone());
        let mut ctx = environment.observing(&mut transport, tick, Arc::clone(&publisher)).calling_stage(format!(
            "observer/{}/{}",
            declared.identifier,
            tick.index()
        ));

        // Construct the author future inside the caught future too: a manual
        // trait implementation can panic before returning its boxed future.
        let work = Box::pin(async { T::observe(&declared.identifier, &mut ctx).await });
        let result = tokio::select! {
            biased;
            _ = stop.wait_for(|stopped| *stopped) => return,
            result = caught(work) => result,
        };

        if let Err(diagnostic) = result {
            failed(&scope.executions, &scope.token, &declared.identifier, diagnostic);
            return;
        }
    }
}

fn failed(executions: &Executions, token: &str, observer: &str, diagnostic: Diagnostic) {
    if let Some(state) = lock(executions).get_mut(token) {
        state.push(BufferedFrame::log(Level::Error, diagnostic.context(format!("observer `{observer}` failed")).redacted().to_string()));
    }
}

/* === Tests === */
