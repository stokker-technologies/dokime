/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Command-Line Errors and Exit Codes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_runtime::Failure;
use tonic::Status;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Operational(String),

    #[error("{0}")]
    Invalid(String),

    #[error("{0}")]
    Capacity(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("{}", status_message(.0))]
    Status(#[from] Status),

    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),

    #[error(transparent)]
    Runtime(#[from] dokime_runtime::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Decode(#[from] prost::DecodeError),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error(transparent)]
    Text(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    Endpoint(#[from] std::net::AddrParseError),

    #[error(transparent)]
    Download(#[from] reqwest::Error),

    #[error(transparent)]
    Persist(#[from] tempfile::PersistError),

    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),

    #[error(transparent)]
    Background(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/* === Traits === */

/* === Implementations === */

impl Error {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Invalid(_) | Self::Endpoint(_) => 2,
            Self::Capacity(_) => 4,
            Self::Status(status) if status.code() == tonic::Code::ResourceExhausted => 4,
            Self::Status(status) if status.code() == tonic::Code::InvalidArgument => 2,
            _ => 3,
        }
    }
}
impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Operational(message.into())
    }
}
impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::Operational(message)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn status_message(status: &Status) -> String {
    match Failure::decode(status.details()) {
        Ok(failure) => format!("{status}: {}", dokime::harmos::model::Failure::from(&failure).message),
        Err(_) => status.to_string(),
    }
}

/* === Tests === */
