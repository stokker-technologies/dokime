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
use std::process::ExitCode;
// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Input(String),
    #[error("{0}")]
    Setup(String),
    #[error("Credential store deadline elapsed; an already-started store operation may still complete.")]
    CredentialTimeout,
    #[error(transparent)]
    Credentials(#[from] dokime_client::credentials::Error),
    #[error("{0}")]
    Check(#[source] Box<Error>),
    #[error("I/O failed: {0}")]
    Io(#[source] std::io::Error),
    #[error("JSON output failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Client(#[from] dokime_client::Error),
    #[error(transparent)]
    Operation(#[from] dokime_client::operations::Error),

    #[error(
        "The device outcome may be unknown. Check device state before manually resending; this command is never retried automatically. {source}"
    )]
    OperationUncertain { source: Box<Error> },

    #[error(
        "Operation response output failed. The device command will not be replayed; check device state before manually resending. {source}"
    )]
    OperationOutput { source: Box<Error> },
}

/* === Traits === */

/* === Implementations === */
impl From<dokime_client::profiles::Error> for Error {
    fn from(error: dokime_client::profiles::Error) -> Self {
        match error {
            dokime_client::profiles::Error::Invalid(message) => Self::Input(message),
            dokime_client::profiles::Error::Io(error) => error.into(),
            dokime_client::profiles::Error::Filesystem(error) => std::io::Error::from(error).into(),
            dokime_client::profiles::Error::Credentials(error) => error.into(),
            dokime_client::profiles::Error::Busy => Self::Setup(error.to_string()),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        if error.kind() == std::io::ErrorKind::InvalidInput { Self::Input(error.to_string()) } else { Self::Io(error) }
    }
}

impl Error {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Check(_) => ExitCode::from(1),
            Self::Operation(dokime_client::operations::Error::CatalogTooLarge) => ExitCode::from(4),
            Self::Input(_) => ExitCode::from(2),
            Self::Client(dokime_client::Error::Rpc(status))
            | Self::Operation(dokime_client::operations::Error::Client(dokime_client::Error::Rpc(status)))
                if status.code() == dokime_client::RpcCode::ResourceExhausted =>
            {
                ExitCode::from(4)
            }
            Self::Client(dokime_client::Error::Rpc(status))
            | Self::Operation(dokime_client::operations::Error::Client(dokime_client::Error::Rpc(status)))
                if status.code() == dokime_client::RpcCode::InvalidArgument =>
            {
                ExitCode::from(2)
            }
            Self::Client(dokime_client::Error::InvalidTarget { .. } | dokime_client::Error::AuthenticationRequired) => ExitCode::from(2),
            Self::Operation(
                dokime_client::operations::Error::Unknown(_)
                | dokime_client::operations::Error::Ambiguous(_)
                | dokime_client::operations::Error::InputStream(_)
                | dokime_client::operations::Error::InputTooLarge,
            ) => ExitCode::from(2),
            _ => ExitCode::from(3),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
