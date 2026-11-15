/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::error::Error as StdError;

// Third Party Imports
use harmos_runtime::Error as NativeError;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Unimplemented(String),
    #[error("{0}")]
    Unavailable(String),

    #[error("{0}")]
    Cancelled(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Stale(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    Precondition(String),

    #[error("{0}: Operation Deadline Expired")]
    Deadline(String),

    #[error(transparent)]
    Storage(#[from] dokime_storage::Error),

    #[error("Rebuild the Plugin Against the Host Protocol: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("{0}")]
    Invalid(String),

    #[error("{0}")]
    Permission(String),

    #[error("{0}")]
    Bound(String),

    #[error("{action}: {}", native_failure(.source))]
    Native {
        action: &'static str,

        #[source]
        source: NativeError,
    },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn native_failure(error: &NativeError) -> String {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(source) = cause {
        message.push_str(": ");
        message.push_str(&source.to_string());
        cause = source.source();
    }
    message
}

/* === Tests === */
