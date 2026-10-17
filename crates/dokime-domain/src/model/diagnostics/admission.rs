/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Typed Refusals At The Domain Admission Boundary.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use thiserror::Error;

// Local Imports
use super::ValidationIssue;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Debug, Error)]
pub enum AdmissionFailure {
    #[error("{message}")]
    Validation {
        code: String,
        message: String,
        issues: Vec<ValidationIssue>,
    },
    #[error("{message}")]
    Conflict { code: String, message: String },
    #[error("{message}")]
    Overload { code: String, message: String },
    #[error("{message}")]
    Unavailable { code: String, message: String },
    #[error("{message}")]
    Forbidden { code: String, message: String },
    #[error("{message}")]
    NotFound { code: String, message: String },
    #[error("{message}")]
    Timeout { code: String, message: String },
    #[error("{message}")]
    Cancelled { code: String, message: String },
    #[error("{message}")]
    Internal { code: String, message: String },

}

/* === Traits === */

/* === Implementations === */
impl AdmissionFailure {
    pub fn code(&self) -> &str {
        match self {
            Self::Validation { code, .. }
            | Self::Conflict { code, .. }
            | Self::Overload { code, .. }
            | Self::Unavailable { code, .. }
            | Self::Forbidden { code, .. }
            | Self::NotFound { code, .. }
            | Self::Timeout { code, .. }
            | Self::Cancelled { code, .. }
            | Self::Internal { code, .. } => code,
        }
    }

    pub fn validation(
        code: impl Into<String>,
        message: impl Into<String>,
        issues: Vec<ValidationIssue>,
    ) -> Self {
        Self::Validation {
            code: code.into(),
            message: message.into(),
            issues,
        }
    }

    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Conflict {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn overload(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Overload {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn unavailable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Unavailable {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn forbidden(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Forbidden {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn not_found(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::NotFound {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn timeout(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Timeout {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn cancelled(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Cancelled {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn internal(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Internal {
            code: code.into(),
            message: message.into(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
