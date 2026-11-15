/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Desktop Capture Folders: The Directories the Built-In Host Lets Plugins Capture From

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    io::Read,
    path::{Path, PathBuf},
};

// Third Party Imports

// Local Imports

/* === Constants === */

const MAX_BYTES: usize = 1024 * 1024;
const MAX_FOLDERS: usize = 256;
const MAX_PATH_BYTES: usize = 4096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Stored {
    folders: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// The desktop-owned file beside the application's other remembered settings.
pub fn path(root: &Path) -> PathBuf {
    root.join("capture-folders.json")
}

/// The saved folders; an absent file is the empty default, which refuses every capture.
pub fn load(path: &Path) -> std::io::Result<Vec<String>> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut bytes = Vec::new();
    file.take(MAX_BYTES as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES {
        return Err(invalid("Read Capture Folders: Keep the File Under 1 MiB".into()));
    }
    let stored: Stored = serde_json::from_slice(&bytes)
        .map_err(|error| invalid(format!("Read Capture Folders: Correct or Remove {}: {error}", path.display())))?;
    validate(&stored.folders)?;
    Ok(stored.folders)
}

pub fn save(path: &Path, folders: &[String]) -> std::io::Result<()> {
    validate(folders)?;
    let bytes = serde_json::to_vec(&Stored { folders: folders.to_vec() }).map_err(std::io::Error::other)?;
    harmos_runtime::filesystem::publication::write(path, &bytes).map_err(Into::into)
}

/// The saved folders as host capture roots; an unreadable file starts the host with none.
pub fn roots(path: &Path) -> Vec<PathBuf> {
    match load(path) {
        Ok(folders) => folders.into_iter().map(PathBuf::from).collect(),
        Err(error) => {
            tracing::warn!(%error, "Read Capture Folders");
            Vec::new()
        }
    }
}

/* === Private Functions === */

fn validate(folders: &[String]) -> std::io::Result<()> {
    if folders.len() > MAX_FOLDERS {
        return Err(invalid(format!("Save Capture Folders: Keep at Most {MAX_FOLDERS} Folders")));
    }
    for folder in folders {
        if folder.is_empty() || folder.len() > MAX_PATH_BYTES || folder.chars().any(char::is_control) || !Path::new(folder).is_absolute() {
            return Err(invalid(format!("Save Capture Folders: Enter {folder:?} as an Absolute Folder Path")));
        }
    }
    Ok(())
}

fn invalid(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{load, path, save};

    #[test]
    fn folders_default_empty_persist_and_refuse_relative_paths() {
        let root = tempfile::tempdir().unwrap();
        let file = path(root.path());
        assert!(load(&file).unwrap().is_empty());
        let folder = root.path().join("device").display().to_string();
        save(&file, std::slice::from_ref(&folder)).unwrap();
        assert_eq!(load(&file).unwrap(), vec![folder.clone()]);
        assert!(save(&file, &["relative/device".into()]).is_err());
        assert_eq!(load(&file).unwrap(), vec![folder]);
    }
}
