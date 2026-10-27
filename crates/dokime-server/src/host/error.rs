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
// 5 The stop outlasted its budget; the next start seals what it left unfinished.
const EXIT_INPUT: u8 = 2;
const EXIT_FAILURE: u8 = 3;
const EXIT_INCOMPATIBLE_STORAGE: u8 = 4;
const EXIT_STOP_INCOMPLETE: u8 = 5;

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
         Move or delete that directory, or point DOKIME_OUTPUT_ROOT at a fresh directory.",
        .root.display()
    )]
    IncompatibleStorage { root: PathBuf, reason: String },

    #[error("I/O failed: {0}")]
    Io(#[source] std::io::Error),

    #[error("JSON output failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Runtime(#[from] dokime::runtime::Error),

    #[error("server failed: {0}")]
    Server(#[from] crate::grpc::Error),

    #[error("the stop did not finish within {budget_s} seconds while {stage}; the next start seals what it left unfinished")]
    StopIncomplete { stage: &'static str, budget_s: u64 },

    #[error("startup failed: {source}; cleanup failed: {cleanup}")]
    StartupCleanup {
        #[source]
        source: Box<Error>,
        cleanup: Box<Error>,
    },

    #[error("server failed: {server}; runtime shutdown also failed: {runtime}")]
    Cleanup { server: Box<crate::grpc::Error>, runtime: Box<dokime::runtime::Error> },
}

/* === Traits === */

/* === Implementations === */

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        if error.kind() == std::io::ErrorKind::InvalidInput { Self::Input(error.to_string()) } else { Self::Io(error) }
    }
}

impl Error {
    pub fn exit_code(&self) -> ExitCode {
        ExitCode::from(match self {
            Self::Input(_) => EXIT_INPUT,
            Self::IncompatibleStorage { .. } => EXIT_INCOMPATIBLE_STORAGE,
            Self::StopIncomplete { .. } => EXIT_STOP_INCOMPLETE,
            _ => EXIT_FAILURE,
        })
    }

    pub fn launch(error: dokime::runtime::Error, root: PathBuf) -> Self {
        let reason = match storage_refusal(&error) {
            Some(refusal) => refusal.to_string(),
            None => return Self::Runtime(error),
        };
        Self::IncompatibleStorage { root, reason }
    }
    pub(crate) fn shutdown(server: std::result::Result<(), crate::grpc::Error>, runtime: dokime::runtime::Result<()>) -> Result<()> {
        match (server, runtime) {
            (Err(server), Err(runtime)) => Err(Self::Cleanup { server: Box::new(server), runtime: Box::new(runtime) }),
            (Err(server), _) => Err(server.into()),
            (_, Err(runtime)) => Err(runtime.into()),
            _ => Ok(()),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn storage_refusal(error: &dokime::runtime::Error) -> Option<&dokime::storage::sqlite::Error> {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(cause) = source {
        if let Some(storage) = cause.downcast_ref::<dokime::storage::sqlite::Error>() {
            return incompatible(storage).then_some(storage);
        }
        source = cause.source();
    }
    None
}

fn incompatible(error: &dokime::storage::sqlite::Error) -> bool {
    use dokime::storage::sqlite::{Error, rusqlite::ErrorCode};
    match error {
        Error::Incompatible(_) | Error::Conflict(_) | Error::Corrupt(_) => true,
        Error::Sqlite(error) => matches!(error.sqlite_error_code(), Some(ErrorCode::NotADatabase | ErrorCode::DatabaseCorrupt)),
        _ => false,
    }
}

/* === Tests === */
