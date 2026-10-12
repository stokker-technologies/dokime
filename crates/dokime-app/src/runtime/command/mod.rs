/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod handle;

/* === Re-Exports === */
pub use dokime::core::CommandContext;
pub use dokime::core::{CommandId, JobRunId};

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::Error;
use dokime::core::{CorrelationId, PromptId, Timestamp, ValidationIssue};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(crate) type CommandResult<T> = std::result::Result<T, Box<CommandReceipt>>;

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandReceipt {
    pub command_id: CommandId,
    pub correlation_id: Option<CorrelationId>,
    pub status: CommandStatus,
    pub runtime_revision: u64,
    pub accepted_at: Timestamp,
    pub duration_ms: Option<u64>,
    pub job_id: Option<JobRunId>,
    pub prompt_id: Option<PromptId>,
    pub rejection: Option<CommandRejection>,
    pub idempotency_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandRejection {
    pub code: String,
    pub message: String,
    pub validation_issues: Vec<ValidationIssue>,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandStatus {
    Accepted,
    Rejected,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn accepted_receipt(
    command_id: CommandId,
    context: &CommandContext,
    accepted_at: Timestamp,
    job_id: Option<JobRunId>,
) -> CommandReceipt {
    CommandReceipt {
        command_id,
        correlation_id: context.correlation_id.clone(),
        status: CommandStatus::Accepted,
        runtime_revision: 0,
        accepted_at,
        duration_ms: None,
        job_id,
        prompt_id: None,
        rejection: None,
        idempotency_key: context.idempotency_key.clone(),
    }
}

pub fn rejected_receipt<E>(
    command_id: CommandId,
    context: &CommandContext,
    accepted_at: Timestamp,
    rejection: E,
) -> CommandReceipt
where
    E: Into<Error>,
{
    CommandReceipt {
        command_id,
        correlation_id: context.correlation_id.clone(),
        status: CommandStatus::Rejected,
        runtime_revision: 0,
        accepted_at,
        duration_ms: None,
        job_id: None,
        prompt_id: None,
        rejection: Some(command_rejection(rejection.into())),
        idempotency_key: context.idempotency_key.clone(),
    }
}

pub fn unavailable_rejection(message: impl Into<String>) -> Error {
    Error::unavailable("runtime_unavailable", message)
}

pub fn internal_rejection(message: impl Into<String>) -> Error {
    Error::internal("runtime_internal", message)
}

/* === Private Functions === */
fn command_rejection(error: Error) -> CommandRejection {
    match error {
        Error::Validation {
            code,
            message,
            issues,
        } => CommandRejection {
            code,
            message,
            validation_issues: issues,
        },
        Error::Conflict { code, message }
        | Error::Overload { code, message }
        | Error::Unavailable { code, message }
        | Error::Forbidden { code, message }
        | Error::NotFound { code, message }
        | Error::Timeout { code, message }
        | Error::Cancelled { code, message }
        | Error::Internal { code, message } => CommandRejection {
            code,
            message,
            validation_issues: Vec::new(),
        },
        error => CommandRejection {
            code: "runtime_internal".to_owned(),
            message: error.to_string(),
            validation_issues: Vec::new(),
        },
    }
}

/* === Tests === */
