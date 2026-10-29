/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cancelled")]
    Cancelled,

    #[error("{0}")]
    Invalid(String),

    #[error("payload digest mismatch: {member}")]
    PayloadDigest { member: String },

    #[error("payload organization member is not integrity indexed: {member}")]
    UnindexedPayload { member: String },

    #[error("--sign requires explicit signing configuration")]
    SigningConfiguration,

    #[error("{0} exceeded its execution timeout")]
    Timeout(String),

    #[error("{0} exceeded the {1} byte limit")]
    Output(String, usize),

    #[error("{program} failed ({status}): {stderr}")]
    Process { program: String, status: std::process::ExitStatus, stderr: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Invalid(error.to_string())
    }
}

impl From<harmos_packaging::bundle::Error> for Error {
    fn from(error: harmos_packaging::bundle::Error) -> Self {
        Self::Invalid(error.to_string())
    }
}
