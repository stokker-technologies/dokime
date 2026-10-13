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
    #[error("I/O failed: {0}")]
    Io(#[source] std::io::Error),
    #[error("JSON output failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Runtime(#[from] dokime_app::Error),
    #[error("server failed: {0}")]
    Server(#[from] crate::grpc::Error),
    #[error("server failed: {server}; runtime shutdown also failed: {runtime}")]
    Cleanup {
        server: Box<crate::grpc::Error>,
        runtime: Box<dokime_app::Error>,
    },
}

/* === Traits === */

/* === Implementations === */
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        if error.kind() == std::io::ErrorKind::InvalidInput {
            Self::Input(error.to_string())
        } else {
            Self::Io(error)
        }
    }
}

impl From<dokime_filesystem::Error> for Error {
    fn from(error: dokime_filesystem::Error) -> Self {
        std::io::Error::from(error).into()
    }
}

impl Error {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            Self::Input(_) => ExitCode::from(2),
            _ => ExitCode::from(3),
        }
    }
    pub(crate) fn startup(server: crate::grpc::Error, cleanup: dokime_app::Result<()>) -> Self {
        match cleanup {
            Ok(()) => Self::Server(server),
            Err(runtime) => Self::Cleanup {
                server: Box::new(server),
                runtime: Box::new(runtime),
            },
        }
    }
    pub(crate) fn shutdown(
        server: std::result::Result<(), crate::grpc::Error>,
        runtime: dokime_app::Result<()>,
    ) -> Result<()> {
        match (server, runtime) {
            (Err(server), Err(runtime)) => Err(Self::Cleanup {
                server: Box::new(server),
                runtime: Box::new(runtime),
            }),
            (Err(server), _) => Err(server.into()),
            (_, Err(runtime)) => Err(runtime.into()),
            _ => Ok(()),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
