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

// Third Party Imports

// Local Imports
use super::{AdmissionIssue, ConnectionError};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<dokime::client::Error> for ConnectionError {
    fn from(error: dokime::client::Error) -> Self {
        match error {
            dokime::client::Error::InvalidTarget { message } => invalid_target(message),
            dokime::client::Error::AuthenticationRequired => Self::AuthenticationRequired,
            dokime::client::Error::Transport(_) => Self::Transport,
            dokime::client::Error::Rpc(status) => rpc_error(status),
            dokime::client::Error::IncompatibleApi => Self::IncompatibleApi,
            dokime::client::Error::IdentityChanged => Self::IdentityChanged,
            dokime::client::Error::BackendUnavailable => Self::BackendUnavailable,
            dokime::client::Error::Task(_) => Self::Transport,
            dokime::client::Error::ExecutionRejected { rejection } => Self::ExecutionRejected {
                code: rejection.code,
                message: rejection.message,
                issues: rejection
                    .issues
                    .into_iter()
                    .map(|issue| AdmissionIssue {
                        path: issue.path,
                        message: issue.message,
                        expected: issue.expected,
                    })
                    .collect(),
            },
            dokime::client::Error::WaitTimedOut { .. } => Self::WaitTimedOut,
            dokime::client::Error::Closed => Self::Superseded,
        }
    }
}

/* === Public Functions === */

pub(super) fn invalid_target(message: &str) -> ConnectionError {
    ConnectionError::InvalidTarget { message: message.into() }
}

pub(super) fn rpc_error(status: tonic::Status) -> ConnectionError {
    match status.code() {
        tonic::Code::Unauthenticated => ConnectionError::Unauthenticated,
        tonic::Code::PermissionDenied => ConnectionError::PermissionDenied,
        _ => ConnectionError::Transport,
    }
}

/* === Private Functions === */

/* === Tests === */
