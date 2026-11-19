/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Validate Installer Organization Inputs

/* === Imports === */
use super::Error;
use image::DynamicImage;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

/* === Public Functions === */
pub(super) fn scratch() -> Result<tempfile::TempDir, Error> {
    Ok(tempfile::tempdir()?)
}
pub(super) fn stage_executable(path: &Path, bytes: &[u8], _image: Option<&DynamicImage>) -> Result<(), Error> {
    fs::write(path, bytes)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(test)]
pub(super) use tests::executable;

#[cfg(test)]
mod tests {
    pub(crate) fn executable() -> Vec<u8> {
        b"fabricated executable".to_vec()
    }
}
