/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Validate Installer Organization Inputs

/* === Modules === */

/* === Re-Exports === */

#[cfg(test)]
pub(crate) use tests::executable;

/* === Imports === */

// Standard Library Imports
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

// Third Party Imports
use image::DynamicImage;

// Local Imports
use super::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn scratch() -> Result<tempfile::TempDir, Error> {
    Ok(tempfile::tempdir()?)
}
pub(crate) fn stage_executable(path: &Path, bytes: &[u8], _image: Option<&DynamicImage>) -> Result<(), Error> {
    fs::write(path, bytes)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    pub(crate) fn executable() -> Vec<u8> {
        b"fabricated executable".to_vec()
    }
}
