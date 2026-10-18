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
    #[error("could not read {path}: {source}")]
    Read { path: std::path::PathBuf, source: std::io::Error },
    #[error("{file} contains invalid CSS near {near:?}: {message}")]
    Parse { file: &'static str, near: String, message: String },
    #[error("{file} must contain exactly one {selector} rule (found {count})")]
    Selector { file: &'static str, selector: &'static str, count: usize },
    #[error("{file} {selector} is missing required custom property --{property}")]
    MissingProperty { file: &'static str, selector: &'static str, property: String },
    #[error("{file} {selector} defines custom property --{property} more than once")]
    DuplicateProperty { file: &'static str, selector: &'static str, property: String },
    #[error("{file} {selector} custom property --{property} must not be empty or !important")]
    InvalidValue { file: &'static str, selector: &'static str, property: String },
    #[error("light.css and dark.css must define the same semantic custom-property names")]
    ModePropertyMismatch,
    #[error("{file} {selector} has unsupported chart property --{property}; expected chart-1 through chart-8")]
    UnsupportedChartProperty { file: &'static str, selector: &'static str, property: String },
    #[error("{file} must not use @import; keep the branding stylesheet self-contained")]
    Import { file: &'static str },
    #[error("{file} font URL {url:?} is not a contained local branding asset")]
    UnsafeUrl { file: &'static str, url: String },
    #[error("{file} font URL {url:?} has unsupported media type; use woff2, woff, ttf, or otf")]
    UnsupportedFont { file: &'static str, url: String },
    #[error("could not read font asset {path}: {source}")]
    FontRead { path: std::path::PathBuf, source: std::io::Error },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
