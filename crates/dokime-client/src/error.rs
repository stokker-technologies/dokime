/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Transport And Request Failures.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid client target: {message}")]
    InvalidTarget { message: &'static str },

    #[error("a valid bearer token is required")]
    AuthenticationRequired,

    #[error("backend transport failed")]
    Transport(#[source] tonic::transport::Error),

    #[error("backend RPC failed")]
    Rpc(#[source] tonic::Status),

    #[error("backend API is incompatible")]
    IncompatibleApi,

    #[error("backend identity changed")]
    IdentityChanged,

    #[error("backend is not ready")]
    BackendUnavailable,

    #[error("client worker failed")]
    Task(#[source] tokio::task::JoinError),

    #[error("execution admission was rejected: {rejection:?}")]
    ExecutionRejected { rejection: dokime_domain::model::execution::wire::Rejection },

    #[error("request wait deadline elapsed")]
    WaitTimedOut {
        #[source]
        source: tokio::time::error::Elapsed,
    },

    #[error("client is closed")]
    Closed,
}
