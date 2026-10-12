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
use crate::connection::ConnectionError;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Clone, Debug, serde::Serialize, thiserror::Error)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum Error {
    #[error("connection: {0:?}")]
    Session(ConnectionError),
    #[error("invalid native RPC request")]
    InvalidRequest,
    #[error("incompatible RPC descriptor")]
    IncompatibleDescriptor,
    #[error("native RPC limit exceeded")]
    LimitExceeded,
    #[error("native RPC capacity exhausted")]
    Busy,
    #[error("unknown RPC stream")]
    UnknownStream,
    #[error("invalid stream acknowledgement")]
    InvalidAck,
    #[error("native RPC cancelled")]
    Cancelled,
    #[error("gRPC {code}: {message}")]
    Grpc {
        code: i32,
        message: String,
        details: Vec<u8>,
    },
}

/* === Traits === */

/* === Implementations === */
impl From<ConnectionError> for Error {
    fn from(error: ConnectionError) -> Self {
        Self::Session(error)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
