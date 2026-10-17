/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod termination;
pub mod saved;
pub mod snapshot;
pub mod result;
pub mod request;
pub mod health;
pub mod notes;
pub mod wire_cursor;
pub mod wire_checkpoint;
pub mod wire;
pub mod command;
pub mod scope;

/* === Re-Exports === */
pub use termination::{TerminationReason};
pub use saved::{EXECUTION_CONTEXT_MEDIA_TYPE, EXECUTION_CONTEXT_SCHEMA_VERSION, ExecutionRevision, ExecutionSource, ExecutionOutput, ExecutionSeal, VerifiedReplicaFile, CycleSeal, ExecutionOutputContent, SavedStreamState, validate_cycle_revisions, output_validation_memory, validate_outputs, validate_output_path, validate_relative_output_path};
pub use snapshot::{ExecutionPluginIdentity, ExecutionSnapshot, ExecutionImplementation};
pub use result::{TestCycle, CycleView, TestExecution, StepAttemptStart, TestStepAttempt, StepRecord, MeasurementRecord, CriterionResult, ResultSummary, IllegalCycleTransition, ExecutionInvalidation, CycleStatus, TestCaseStatus, ExecutionStatus, StepStatus, CriterionStatus, ResultStatus, validate_invocation, effective_attempts};
pub use request::{RunSummary, RunReceipt, AttemptClaim, StepCommandReceipt, RunPhase, StepCommandPhase};
pub use health::{HealthItem, HealthSeverity};
pub use notes::{OperatorNote, OperatorNoteRequest, NoteAdmission, NoteOwner, note_identity, NOTE_SCAN_LIMIT};
pub use command::CommandContext;
pub use scope::CapturePhase;
pub use scope::EvidenceOrigin;
pub use scope::RecordScope;
pub use scope::scope_visible_in;

/* === Public Functions === */

/* === Private Functions === */
