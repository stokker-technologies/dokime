/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "native")]
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

#[cfg(feature = "native")]
pub mod listen;

#[cfg(feature = "native")]
mod builtins;

#[cfg(feature = "native")]
mod calls;

#[cfg(feature = "native")]
pub(crate) mod instances;
pub mod scope;

#[cfg(feature = "native")]
mod serving;

#[cfg(feature = "native")]
pub(crate) mod tick;

#[cfg(feature = "native")]
mod run;

#[cfg(feature = "native")]
pub(crate) mod observing;

#[cfg(feature = "native")]
mod scanning;

#[cfg(feature = "native")]
mod template;

#[cfg(feature = "native")]
mod unwind;

#[cfg(feature = "host")]
pub mod instance;

#[cfg(feature = "host")]
pub mod storage;

#[cfg(feature = "native")]
mod workflow;

/* === Re-Exports === */

#[cfg(feature = "native")]
pub use callbacks::{CallbackTask, Callbacks};
pub use context::{
    Called, Cleanup, CleanupContext, Context, CriterionContext, InitializationContext, Initialize, Required, Step, StepContext,
    Traceability, TraceabilityContext,
};

#[cfg(feature = "native")]
pub use context::{ObserverContext, Observing};
pub use document::DocumentHandle;
pub use environment::Environment;
pub use evidence::ExecutionEvidence;
pub use lease::{Lease, LeaseHold, LeaseRelease};
pub use progress::{ExecutionProgress, Progress, ProgressSink};
pub use slot::Slot;

#[cfg(feature = "native")]
pub use listen::{Deadlined, Listenable, ListeningRecords, ListeningRows};

#[cfg(feature = "native")]
pub(crate) use listen::{PublishedRow, rows as listening_rows};

#[cfg(feature = "native")]
pub use calls::{Answering, Caller, StepHost};

#[cfg(feature = "native")]
pub use instances::{ExecutionIdentity, StepIdentity, StepInstance};
pub use scope::{ExecutionScope, IdempotencyKey, WatchTicket};

#[cfg(feature = "native")]
pub use serving::{
    EmbeddedRegistry, EmbeddedTemplates, ExecutionHost, Run, Runnable, SIGNAL_POLICY, Served, Signal, bind, contracts,
    contracts_with_settings, interrupt, runner, serve,
};

#[cfg(feature = "native")]
pub use tick::Tick;

#[cfg(feature = "native")]
pub use template::{
    DeclaredObserver, EvaluationFuture, InitializationFuture, SamplePublisher, SidecarTemplate, StampedRow, StepFuture, TemplateDefinition,
    TraceabilityFuture,
};

#[cfg(feature = "host")]
pub use instance::TestExecutionInstance;

#[cfg(feature = "native")]
#[cfg(feature = "native")]
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
