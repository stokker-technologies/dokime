/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stage Explicit Installer Plugin Bundles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

// Third Party Imports

// Local Imports
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn stage(paths: &[PathBuf], scratch: &Path) -> Result<BTreeMap<String, PathBuf>, Error> {
    let mut names = BTreeSet::new();
    let mut selected = Vec::new();
    for path in paths {
        let name = path.file_name().and_then(|name| name.to_str());
        let Some(name) = name else {
            return Err(Error::Invalid("plugin bundle has no filename".into()));
        };
        super::payload::relative(name)?;
        if path.extension().is_none_or(|extension| extension != "dokime") {
            return Err(Error::Invalid("installer plugins must be .dokime bundles".into()));
        }
        let folded = name.to_lowercase();
        if !names.insert(folded) {
            let message = format!("duplicate installer plugin filename: {name}");
            return Err(Error::Invalid(message));
        }
        let metadata = std::fs::symlink_metadata(path)?;
        if !metadata.is_file() {
            return Err(Error::Invalid("installer plugin must be a regular file".into()));
        }
        let bytes = std::fs::read(path)?;
        selected.push((name.to_owned(), bytes));
    }

    let mut staged = BTreeMap::new();
    if selected.is_empty() {
        return Ok(staged);
    }
    let directory = scratch.join("plugins");
    std::fs::create_dir(&directory)?;
    for (name, bytes) in selected {
        let destination = directory.join(&name);
        std::fs::write(&destination, bytes)?;
        staged.insert(name, destination);
    }
    Ok(staged)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_plugins_keep_their_bytes_and_implicit_plugins_are_absent() {
        let input = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let host = input.path().join("host.dokime");
        let bench = input.path().join("bench.dokime");
        std::fs::write(&host, b"host bundle").unwrap();
        std::fs::write(&bench, b"bench bundle").unwrap();

        assert!(stage(&[], scratch.path()).unwrap().is_empty());
        assert!(!scratch.path().join("plugins").exists());
        let staged = stage(std::slice::from_ref(&host), scratch.path()).unwrap();
        assert_eq!(staged.len(), 1);
        assert_eq!(std::fs::read(&staged["host.dokime"]).unwrap(), std::fs::read(host).unwrap());
        assert!(!scratch.path().join("plugins/bench.dokime").exists());

        let repeated = tempfile::tempdir().unwrap();
        let staged = stage(&[bench.clone(), bench.with_file_name("host.dokime")], repeated.path()).unwrap();
        assert_eq!(staged.len(), 2);
        assert_eq!(std::fs::read(&staged["bench.dokime"]).unwrap(), std::fs::read(bench).unwrap());
    }

    #[test]
    fn invalid_plugins_fail_before_staging() {
        let input = tempfile::tempdir().unwrap();
        let scratch = tempfile::tempdir().unwrap();
        let host = input.path().join("host.dokime");
        std::fs::write(&host, b"bundle").unwrap();

        assert!(stage(&[host.clone(), host.clone()], scratch.path()).is_err());
        assert!(stage(&[host.clone(), host.with_file_name("HOST.dokime")], scratch.path()).is_err());
        assert!(stage(&[input.path().to_owned()], scratch.path()).is_err());
        assert!(stage(&[host.with_extension("zip")], scratch.path()).is_err());
        assert!(stage(&[host.with_file_name("missing.dokime")], scratch.path()).is_err());
        assert!(!scratch.path().join("plugins").exists());
    }
}
