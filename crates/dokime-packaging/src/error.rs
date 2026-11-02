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

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cancelled")]
    Cancelled,

    #[error("{0}")]
    Invalid(String),

    #[error(
        "GitHub API rate limit exceeded{limit_message}; setting GITHUB_TOKEN raises the limit{reset_message}",
        limit_message = .limit.map(|limit| format!(" ({limit} requests)")).unwrap_or_default(),
        reset_message = .reset.map(|reset| format!("; resets at {reset} (UTC epoch seconds)")).unwrap_or_default()
    )]
    GithubRateLimit { limit: Option<u64>, reset: Option<u64> },

    #[error("distribution digest mismatch: {member}")]
    DistributionDigest { member: String },

    #[error("distribution organization member is not integrity indexed: {member}")]
    UnindexedDistribution { member: String },

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

/* === Traits === */

/* === Implementations === */

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

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
