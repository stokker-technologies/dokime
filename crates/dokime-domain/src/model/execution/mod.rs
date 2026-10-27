/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

/* === Modules === */

pub mod checkpoint;
pub mod command;
pub mod cursor;
pub mod health;
pub mod mode;
pub mod notes;
pub mod request;
pub mod result;
pub mod saved;
pub mod scope;
pub mod snapshot;
pub mod termination;
pub mod wire;
pub mod wire_checkpoint;
pub mod wire_cursor;

/* === Re-Exports === */

pub use checkpoint::{ExecutionCheckpoint, SavedEvidenceError};
pub use command::CommandContext;
pub use cursor::PageCursor;
pub use health::{HealthItem, HealthSeverity};
pub use mode::{ExecutionMode, StepCommand};
pub use notes::{NoteAdmission, NoteOwner, OperatorNote, OperatorNoteRequest, note_identity};
pub use request::{AttemptClaim, RunPhase, RunReceipt, RunSummary, StepCommandPhase, StepCommandReceipt};
pub use result::{
    CriterionResult, CriterionStatus, CyclePhase, CycleStatus, CycleView, ExecutionInvalidation, ExecutionStatus, IllegalCycleTransition,
    MeasurementRecord, ResultStatus, ResultSummary, StepAttemptStart, StepRecord, StepStatus, TestCaseStatus, TestCycle, TestExecution,
    TestStepAttempt, effective_attempts, validate_invocation,
};
pub use saved::{
    CycleSeal, EXECUTION_CONTEXT_MEDIA_TYPE, EXECUTION_CONTEXT_SCHEMA_VERSION, ExecutionOutput, ExecutionOutputContent, ExecutionRevision,
    ExecutionSeal, ExecutionSource, SavedStreamState, VerifiedReplicaFile, output_validation_memory, validate_cycle_revisions,
    validate_output_path, validate_outputs, validate_relative_output_path,
};
pub use scope::CapturePhase;
pub use scope::EvidenceOrigin;
pub use scope::RecordScope;
pub use scope::scope_visible_in;
pub use snapshot::{ExecutionImplementation, ExecutionPluginIdentity, ExecutionSnapshot};
pub use termination::TerminationReason;

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
