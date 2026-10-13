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
impl From<dokime_client::Error> for ConnectionError {
    fn from(error: dokime_client::Error) -> Self {
        match error {
            dokime_client::Error::InvalidTarget { message } => invalid_target(message),
            dokime_client::Error::AuthenticationRequired => Self::AuthenticationRequired,
            dokime_client::Error::Transport(_) => Self::Transport,
            dokime_client::Error::Rpc(status) => rpc_error(status),
            dokime_client::Error::IncompatibleApi => Self::IncompatibleApi,
            dokime_client::Error::IdentityChanged => Self::IdentityChanged,
            dokime_client::Error::BackendUnavailable => Self::BackendUnavailable,
            dokime_client::Error::Task(_) => Self::Transport,
            dokime_client::Error::ExecutionRejected { rejection } => Self::ExecutionRejected {
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
            dokime_client::Error::WaitTimedOut { .. } => Self::WaitTimedOut,
            dokime_client::Error::Closed => Self::Superseded,
        }
    }
}

/* === Public Functions === */
pub(super) fn invalid_target(message: &str) -> ConnectionError {
    ConnectionError::InvalidTarget {
        message: message.into(),
    }
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
