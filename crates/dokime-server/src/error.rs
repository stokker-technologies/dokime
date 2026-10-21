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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T, E = Error> = std::result::Result<T, E>;

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("filesystem publication failed: {0}")]
    Filesystem(#[from] dokime::harmos::filesystem::Error),

    #[error("remote security storage failed: {0}")]
    RemoteStorage(#[source] std::io::Error),

    #[error("remote listener change failed: {change}; restoring the previous state also failed: {restoration}")]
    RemoteRollback { change: Box<Error>, restoration: Box<Error> },

    #[error("invalid gRPC configuration: {0}")]
    Configuration(&'static str),

    #[error("cannot configure generated operation service: {0}")]
    OperationService(#[source] tonic::Status),

    #[error("runtime operation failed: {0}")]
    Operation(#[from] dokime::harmos::Error),

    #[error("invalid declared operation contract: {0}")]
    Contract(#[from] dokime::harmos::operations::ContractError),

    #[error("cannot bind gRPC listener: {0}")]
    Bind(#[from] std::io::Error),

    #[error("gRPC listener failed: {0}")]
    Transport(#[from] tonic::transport::Error),

    #[error("gRPC listener task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
