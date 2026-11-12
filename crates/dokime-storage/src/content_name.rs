/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Portable Content Names

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::{Component, Path};

// Third Party Imports

// Local Imports
use crate::{Error, Result};

/* === Constants === */

const MAX_NAME_BYTES: usize = 255;
const MAX_PATH_BYTES: usize = 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn validate_artifact_name(name: &str) -> Result<()> {
    if name.len() > MAX_NAME_BYTES || !is_relative_path(name) {
        return Err(Error::Invalid("Upload Artifact: Supply a Safe Relative Name of at Most 255 Bytes".into()));
    }
    Ok(())
}
pub fn validate_execution_path(path: &str) -> Result<()> {
    if !is_relative_path(path) {
        return Err(Error::Invalid(
            "Read Execution File: Select a Registered Relative Path Without Parent, Device or Absolute Names".into(),
        ));
    }
    Ok(())
}

/* === Private Functions === */

// Copied Pure Validation; Delete the Old Domain Owner at Cutover.
fn is_relative_path(path: &str) -> bool {
    if path.is_empty() || path.len() > MAX_PATH_BYTES {
        return false;
    }
    if path.chars().any(char::is_control) || path.contains('\\') {
        return false;
    }
    if path.split('/').any(|segment| {
        let stem = segment.split('.').next().unwrap_or_default().to_ascii_uppercase();
        let device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
            || ["COM", "LPT"].iter().any(|prefix| {
                stem.strip_prefix(prefix)
                    .is_some_and(|suffix| matches!(suffix, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"))
            });
        segment.is_empty()
            || segment == "."
            || segment == ".."
            || segment.ends_with(['.', ' '])
            || segment.contains([':', '<', '>', '"', '|', '?', '*'])
            || device
    }) {
        return false;
    }
    let drive = path.split('/').next().is_some_and(|first| first.len() == 2 && first.ends_with(':'));
    let rooted =
        Path::new(path).components().any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)));
    !drive && !rooted
}

/* === Tests === */
