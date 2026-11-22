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
use std::error;

// Third Party Imports
use harmos_protocol::model::Failure;

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
    Library(#[from] harmos_runtime::Error),

    #[error("Start Composed Services: {0}")]
    ComposedStartup(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error(transparent)]
    Storage(#[from] dokime_storage::Error),

    #[error(transparent)]
    Operation(#[from] dokime_operations::Error),

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

    #[error("{}", plugin_failure(.0))]
    Plugin(Box<harmos_protocol::model::Failure>),

    #[error("Host Worker Failed; Restart the Host and Inspect Interrupted Evidence: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("{run}; Then {close}")]
    Unsettled {
        #[source]
        run: Box<Error>,
        close: Box<Error>,
    },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn library_failure(error: &harmos_runtime::Error) -> String {
    let mut message = error.to_string();
    let mut cause = error::Error::source(error);
    while let Some(source) = cause {
        message.push_str(": ");
        message.push_str(&source.to_string());
        cause = source.source();
    }
    message
}

fn plugin_failure(failure: &Failure) -> String {
    let mut message = String::new();
    let mut cause = Some(failure);
    while let Some(failure) = cause {
        if !message.is_empty() {
            message.push_str(": ");
        }
        message.push_str(&failure.message);
        if !failure.action.is_empty() {
            message.push_str("; ");
            message.push_str(&failure.action);
        }
        cause = failure.cause.as_deref();
        if cause.is_none() {
            message.push_str(&format!(" [{:?}, {}]", failure.classification(), failure.code));
        }
    }
    message.split_whitespace().collect::<Vec<_>>().join(" ")
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
        let runtime = harmos_runtime::Error::OperationFailed {
            operation: "Access.Configure".into(),
            source: Arc::new(cause),
        };
        let error = Error::from(runtime);

        assert!(error.to_string().contains(&diagnostic));
        let runtime = error::Error::source(&error).unwrap().downcast_ref::<harmos_runtime::Error>().unwrap();
        let harmos_runtime::Error::OperationFailed { source, .. } = runtime else { panic!("expected an operation failure") };
        let native = source.downcast_ref::<io::Error>().unwrap();
        assert_eq!(native.raw_os_error(), Some(5));
    }

    #[test]
    fn plugin_failures_keep_actionable_causes_and_classification() {
        let cause = Failure {
            classification: harmos_protocol::model::Classification::Refused as i32,
            code: "NOT_FOUND".into(),
            message: "Missing api_token".into(),
            action: "Add api_token in Secret Storage".into(),
            ..Default::default()
        };
        let failure = Failure {
            message: "Initialize Plugin".into(),
            cause: Some(Box::new(cause.clone())),
            ..Default::default()
        };
        let error = Error::Plugin(Box::new(failure));
        let message = error.to_string();
        assert!(message.contains(&cause.message));
        assert!(message.contains(&cause.action));
        assert!(message.contains(&cause.code));
        assert_eq!(message.lines().count(), 1);
        let Error::Plugin(failure) = error else { unreachable!() };
        assert_eq!(failure.cause.unwrap().classification, cause.classification);
    }
}
