/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Root-Owned Desktop Backend Choice

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::{self, Read},
    path::Path,
};

// Third Party Imports

// Local Imports
use super::BackendChoice;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn load(path: &Path) -> io::Result<BackendChoice> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(BackendChoice::OwnedLocal),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(4097).read_to_end(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "desktop backend choice exceeds 4 KiB"));
    }
    serde_json::from_slice(&bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub fn save(path: &Path, selection: &BackendChoice) -> io::Result<()> {
    let bytes = serde_json::to_vec(selection).map_err(io::Error::other)?;
    if bytes.len() > 4096 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "desktop backend choice exceeds 4 KiB"));
    }
    if let Some(parent) = path.parent() {
        dokime::harmos::filesystem::publication::create_dir_all(parent)?;
    }
    dokime::harmos::filesystem::publication::write(path, &bytes).map_err(Into::into)
}

/* === Private Functions === */

/* === Tests === */
