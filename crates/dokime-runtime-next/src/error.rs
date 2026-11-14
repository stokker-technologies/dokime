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
use harmos_runtime::Error as RuntimeError;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Open Evidence Root: {0} is occupied by another host; stop that host or choose a different root")]
    OccupiedRoot(std::path::PathBuf),

    #[error("Open Evidence Root {path}: Check Its Permissions and Retry: {source}")]
    RootIo {
        path: std::path::PathBuf,

        #[source]
        source: std::io::Error,
    },

    #[error("{}", library_failure(.0))]
    Library(#[from] RuntimeError),

    #[error("Start Composed Services: {0}")]
    ComposedStartup(#[source] Box<dyn std::error::Error + Send + Sync>),

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

fn library_failure(error: &RuntimeError) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io, sync::Arc};

    #[test]
    fn library_failures_preserve_native_causes() {
        let cause = io::Error::from_raw_os_error(5);
        let diagnostic = cause.to_string();
        let runtime = RuntimeError::OperationFailed {
            operation: "Access.Configure".into(),
            source: Arc::new(cause),
        };
        let error = Error::from(runtime);

        assert!(error.to_string().contains(&diagnostic));
        let runtime = error.source().unwrap().downcast_ref::<RuntimeError>().unwrap();
        let RuntimeError::OperationFailed { source, .. } = runtime else { panic!("expected an operation failure") };
        let native = source.downcast_ref::<io::Error>().unwrap();
        assert_eq!(native.raw_os_error(), Some(5));
    }
}
