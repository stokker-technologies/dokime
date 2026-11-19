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
    #[error("Package Installer: {0}; Correct the Manifest or Supply a Verified Native Kit")]
    Invalid(String),

    #[error(
        "GitHub API rate limit exceeded{limit_message}; setting GITHUB_TOKEN raises the limit{reset_message}",
        limit_message = .limit.map(|limit| format!(" ({limit} requests)")).unwrap_or_default(),
        reset_message = .reset.map(|reset| format!("; resets at {reset} (UTC epoch seconds)")).unwrap_or_default()
    )]
    GithubRateLimit { limit: Option<u64>, reset: Option<u64> },

    #[error("Package Installer: Download a Fresh Kit; Distribution Digest Mismatch: {member}")]
    DistributionDigest { member: String },

    #[error("Package Installer: Re-export the Kit With Every Organization File Indexed: {member}")]
    UnindexedDistribution { member: String },

    #[error("invalid installer [license] {field}: {reason}")]
    License { field: &'static str, reason: String },

    #[error("--sign requires explicit signing configuration")]
    SigningConfiguration,

    #[error("Package Installer: Check Input Paths, File Permissions and Available Tools: {0}")]
    Io(#[from] std::io::Error),
}

/* === Traits === */

/* === Implementations === */

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Invalid(error.to_string())
    }
}

impl From<harmos_runtime::bundle::Error> for Error {
    fn from(error: harmos_runtime::bundle::Error) -> Self {
        Self::Invalid(error.to_string())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
