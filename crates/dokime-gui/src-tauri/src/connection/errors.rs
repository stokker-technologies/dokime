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
use super::ConnectionError;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<super::host::Error> for ConnectionError {
    fn from(error: super::host::Error) -> Self {
        use super::host::Error;
        match error {
            Error::Target(message) => Self::InvalidTarget { message },
            Error::Rpc(status) => rpc_error(status),
            Error::IdentityChanged => Self::IdentityChanged,
            Error::IncompleteIdentity => Self::IncompatibleApi,
            Error::Superseded => Self::Superseded,
            Error::Disconnected => Self::NotConnected,
            Error::Host(_) | Error::Transport(_) => Self::Transport,
        }
    }
}

/* === Public Functions === */

pub(super) fn rpc_error(status: tonic::Status) -> ConnectionError {
    match status.code() {
        tonic::Code::Unauthenticated => ConnectionError::Unauthenticated,
        tonic::Code::PermissionDenied => ConnectionError::PermissionDenied,
        _ => ConnectionError::Transport,
    }
}

/* === Private Functions === */

/* === Tests === */
