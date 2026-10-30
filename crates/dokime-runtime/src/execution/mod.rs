/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "sidecar")]
mod callbacks;
pub mod context;
pub mod document;
pub mod environment;
pub mod evidence;
pub(crate) mod evidence_rows;
pub mod lease;
pub mod log;
pub mod progress;
pub mod slot;
pub mod traceability;

#[cfg(feature = "sidecar")]
pub mod listen;

#[cfg(feature = "sidecar")]
mod builtins;

#[cfg(feature = "sidecar")]
mod calls;

#[cfg(feature = "sidecar")]
pub(crate) mod instances;
pub mod scope;

#[cfg(feature = "sidecar")]
mod serving;

#[cfg(feature = "sidecar")]
pub(crate) mod tick;

#[cfg(feature = "sidecar")]
mod run;

#[cfg(feature = "sidecar")]
pub(crate) mod observing;

#[cfg(feature = "sidecar")]
mod scanning;

#[cfg(feature = "sidecar")]
mod template;

#[cfg(feature = "sidecar")]
mod unwind;

#[cfg(feature = "native")]
pub mod instance;

#[cfg(feature = "native")]
pub mod storage;

#[cfg(feature = "sidecar")]
mod workflow;

/* === Re-Exports === */

#[cfg(feature = "sidecar")]
pub use callbacks::{CallbackTask, Callbacks};
pub use context::{
    Called, Cleanup, CleanupContext, Context, CriterionContext, InitializationContext, Initialize, Required, Step, StepContext,
};

#[cfg(feature = "sidecar")]
pub use context::{ObserverContext, Observing};
pub use document::DocumentHandle;
pub use environment::Environment;
pub use evidence::ExecutionEvidence;
pub use lease::{Lease, LeaseHold, LeaseRelease};
pub use progress::{ExecutionProgress, Progress, ProgressSink};
pub use slot::Slot;
pub use traceability::TraceabilityHandle;

#[cfg(feature = "sidecar")]
pub use listen::{Deadlined, Listenable, ListeningRecords, ListeningRows};

#[cfg(feature = "sidecar")]
pub(crate) use listen::{PublishedRow, rows as listening_rows};

#[cfg(feature = "sidecar")]
pub use calls::{Answering, Caller, StepHost};

#[cfg(feature = "sidecar")]
pub use instances::{ExecutionIdentity, StepIdentity, StepInstance};
pub use scope::{ExecutionScope, IdempotencyKey, WatchTicket};

#[cfg(feature = "sidecar")]
pub use serving::{
    EmbeddedRegistry, EmbeddedTemplates, Empty, ExecutionHost, Operations, Run, Runnable, SIGNAL_POLICY, Served, Signal, bind, contracts,
    contracts_with_settings, interrupt, prepare, runner,
};

#[cfg(feature = "sidecar")]
pub use tick::Tick;

#[cfg(feature = "sidecar")]
pub use template::{
    DeclaredObserver, EvaluationFuture, InitializationFuture, SamplePublisher, SidecarTemplate, StampedRow, StepFuture, TemplateDefinition,
};

#[cfg(feature = "native")]
pub use instance::TestExecutionInstance;

#[cfg(feature = "sidecar")]
pub use workflow::{EmbeddedPublisher, SampleOutput, Workflow};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
