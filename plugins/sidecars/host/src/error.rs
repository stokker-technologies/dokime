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

pub type Result<T, E = Error> = std::result::Result<T, E>;

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "runtime")]
    #[error("host system collection failed: {0}")]
    Collection(#[from] sysinfo::Error),

    #[error("unknown {kind}: {value}")]
    UnknownValue { kind: &'static str, value: String },

    #[error("host probe failed: {0}")]
    Probe(String),

    #[error("host command failed: {0}")]
    Command(String),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
