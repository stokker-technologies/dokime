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
    #[error(transparent)]
    Library(#[from] harmos_runtime::Error),

    #[error(transparent)]
    Storage(#[from] dokime_storage::Error),

    #[error(transparent)]
    Operation(#[from] dokime_operations_next::Error),

    #[error("Reconnect the Plugin and Retry: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("Plugin Transport Failed; Reconnect the Plugin and Retry: {0}")]
    Status(#[from] tonic::Status),

    #[error("Rebuild the Plugin Against the Current Protocol: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("Check the Plugin Executable and Its Permissions: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Invalid(String),

    #[error("Execution Cancelled by the Operator")]
    Cancelled,

    #[error("Plugin Operation Failed: {0:?}")]
    Plugin(Box<harmos_protocol::v1::Failure>),

    #[error("Host Worker Failed; Restart the Host and Inspect Interrupted Evidence: {0}")]
    Join(#[from] tokio::task::JoinError),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
