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
pub mod error;
pub mod evidence;
pub mod health;
pub mod lease;
pub mod log;
pub mod notes;
pub mod progress;
pub mod request;
pub mod result;
pub mod saved;
pub mod slot;
pub mod snapshot;

#[cfg(feature = "native")]
pub mod listen;

#[cfg(feature = "native")]
mod calls;

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

#[cfg(feature = "native")]
mod workflow;

/* === Re-Exports === */

#[cfg(feature = "native")]
pub use callbacks::{CallbackTask, Callbacks};
pub use context::{
    Calls, Cleanup, CleanupContext, Context, CriterionContext, InitializationContext, Initialize,
    Required, Step, StepContext, Traceability, TraceabilityContext,
};

#[cfg(feature = "native")]
pub use context::{ObserverContext, Observing};
pub use document::DocumentHandle;
pub use environment::Environment;
pub use error::TerminationReason;
pub use evidence::ExecutionEvidence;
pub use health::{HealthItem, HealthSeverity};
pub use lease::{Lease, LeaseHold, LeaseRelease};
pub use notes::{OperatorNote, OperatorNoteRequest};
pub use progress::{ExecutionProgress, Progress, ProgressSink};
pub use result::{
    CriterionResult, CriterionStatus, CycleStatus, CycleView, ExecutionMode, ExecutionStatus,
    MeasurementRecord, ResultStatus, ResultSummary, StepAttemptStart, StepRecord, StepStatus,
    TestCaseStatus, TestCycle, TestExecution, TestStepAttempt,
};
pub use slot::Slot;
pub use snapshot::{ExecutionImplementation, ExecutionPluginIdentity, ExecutionSnapshot};

#[cfg(feature = "native")]
pub use listen::{Deadlined, Listenable, ListeningRecords, ListeningRows};

#[cfg(feature = "native")]
pub(crate) use listen::{PublishedRow, rows as listening_rows};

#[cfg(feature = "native")]
pub use calls::{Answering, Caller, StepHost};
pub use scope::{ExecutionScope, IdempotencyKey};
#[cfg(feature = "native")]
pub use serving::{
    ExecutionHost, Run, Runnable, Signal, bind, contracts, interrupt, runner, serve,
};
#[cfg(feature = "native")]
pub use tick::Tick;

#[cfg(feature = "native")]
pub use template::{
    DeclaredObserver, EvaluationFuture, InitializationFuture, SamplePublisher, SidecarTemplate,
    StampedRow, StepFuture, TemplateDefinition, TraceabilityFuture,
};

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
