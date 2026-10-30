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
use std::{
    io,
    path::{Path, PathBuf},
};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct Directory {
    root: PathBuf,
}

/* === Enums === */

/* === Traits === */

pub trait Source {
    fn read(&self, relative: &str) -> io::Result<Vec<u8>>;
}

/* === Implementations === */

impl Directory {
    pub fn new(root: &Path) -> io::Result<Self> {
        Ok(Self { root: root.canonicalize()? })
    }
}

impl Source for Directory {
    fn read(&self, relative: &str) -> io::Result<Vec<u8>> {
        let path = self.root.join(relative).canonicalize()?;
        if !path.starts_with(&self.root) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "path leaves the brand bundle"));
        }
        std::fs::read(path)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
