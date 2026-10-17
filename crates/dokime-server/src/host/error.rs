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
use std::{path::PathBuf, process::ExitCode};
// Third Party Imports

// Local Imports

/* === Constants === */
// Process Exit Codes
// 0 Clean stop.
// 2 Invalid input: arguments, environment file, plugin bundles, development commands.
// 3 Any other failure: I/O, runtime, transport, cleanup.
// 4 The data root holds storage this build cannot open; moving it aside lets the next launch start fresh.
const EXIT_INPUT: u8 = 2;
const EXIT_FAILURE: u8 = 3;
const EXIT_INCOMPATIBLE_STORAGE: u8 = 4;

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
    #[error(
        "Dokime cannot open the data root {}.\n\
         The stored data is not usable by this build: {reason}.\n\
         Move or delete that directory, or point DOKIME_OUTPUT_ROOT or the settings.json data_root at a fresh directory.",
        .root.display()
    )]
    IncompatibleStorage { root: PathBuf, reason: String },
    #[error("I/O failed: {0}")]
    Io(#[source] std::io::Error),
    #[error("JSON output failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Runtime(#[from] dokime_runtime::runtime::Error),
    #[error("server failed: {0}")]
    Server(#[from] crate::grpc::Error),
    #[error("server failed: {server}; runtime shutdown also failed: {runtime}")]
    Cleanup {
        server: Box<crate::grpc::Error>,
        runtime: Box<dokime_runtime::runtime::Error>,
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
        ExitCode::from(match self {
            Self::Input(_) => EXIT_INPUT,
            Self::IncompatibleStorage { .. } => EXIT_INCOMPATIBLE_STORAGE,
            _ => EXIT_FAILURE,
        })
    }

    /// Classifies a failed launch on `root`.
    ///
    /// Only refusals that name the stored data's own shape become
    /// `IncompatibleStorage`; lease contention, I/O and replay failures stay
    /// runtime failures so the remedy never suggests discarding usable data.
    pub fn launch(error: dokime_runtime::runtime::Error, root: PathBuf) -> Self {
        let reason = match storage_refusal(&error) {
            Some(refusal) => refusal.to_string(),
            None => return Self::Runtime(error),
        };
        Self::IncompatibleStorage { root, reason }
    }
    pub(crate) fn startup(server: crate::grpc::Error, cleanup: dokime_runtime::runtime::Result<()>) -> Self {
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
        runtime: dokime_runtime::runtime::Result<()>,
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
fn storage_refusal(error: &dokime_runtime::runtime::Error) -> Option<&dokime_runtime::storage::sqlite::Error> {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(cause) = source {
        if let Some(storage) = cause.downcast_ref::<dokime_runtime::storage::sqlite::Error>() {
            return incompatible(storage).then_some(storage);
        }
        source = cause.source();
    }
    None
}

fn incompatible(error: &dokime_runtime::storage::sqlite::Error) -> bool {
    use dokime_runtime::storage::sqlite::{Error, rusqlite::ErrorCode};
    match error {
        Error::Incompatible(_) | Error::Conflict(_) | Error::Corrupt(_) => true,
        Error::Sqlite(error) => matches!(
            error.sqlite_error_code(),
            Some(ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt)
        ),
        _ => false,
    }
}

/* === Tests === */
