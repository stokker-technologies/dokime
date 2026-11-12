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

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    Bound(String),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
