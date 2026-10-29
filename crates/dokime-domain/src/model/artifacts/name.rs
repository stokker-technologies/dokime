/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Artifact names are relative paths: their folders are implicit, never created.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Component;
use std::path::Path;

// Third Party Imports

// Local Imports

/* === Constants === */

// The wire and storage bound of one artifact name, in UTF-8 bytes.
pub const MAX_NAME_BYTES: usize = 255;

// The bound of any relative execution path, in UTF-8 bytes.
pub const MAX_PATH_BYTES: usize = 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidName {
    #[error("artifact name must contain 1–255 bytes")]
    Length,

    #[error("artifact name must be a relative path with forward slashes and no empty, '.' or '..' segments")]
    Path,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// One relative path below an execution folder: forward slashes, no empty, `.`
// or `..` segment, no root, drive or control character.
pub fn is_relative_path(path: &str) -> bool {
    if path.is_empty() || path.len() > MAX_PATH_BYTES {
        return false;
    }
    if path.chars().any(char::is_control) || path.contains('\\') {
        return false;
    }
    if path.split('/').any(|segment| segment.is_empty() || segment == "." || segment == "..") {
        return false;
    }
    let drive = path.split('/').next().is_some_and(|first| first.len() == 2 && first.ends_with(':'));
    let rooted =
        Path::new(path).components().any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)));
    !drive && !rooted
}

// An artifact name: a relative path that fits the 255-byte name bound.
pub fn validate(name: &str) -> Result<(), InvalidName> {
    if name.is_empty() || name.len() > MAX_NAME_BYTES {
        return Err(InvalidName::Length);
    }
    if !is_relative_path(name) {
        return Err(InvalidName::Path);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
