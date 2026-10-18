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
use std::{env, path::PathBuf};
// Third Party Imports

// Local Imports

/* === Constants === */
pub const DEFAULT_APPLICATION_ID: &str = "com.stokker.dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationPaths {
    pub root: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ApplicationPaths {
    pub fn resolve(application_id: &str) -> std::io::Result<Self> {
        Self::resolve_with(application_id, cfg!(windows), |key| env::var_os(key).map(PathBuf::from))
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn settings(&self) -> PathBuf {
        self.root.join("settings.json")
    }

    pub fn logs(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn plugins(&self) -> PathBuf {
        self.root.join("plugins")
    }

    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }

    pub fn default_data(&self) -> PathBuf {
        self.root.join("data")
    }

    fn resolve_with(application_id: &str, windows: bool, get: impl Fn(&str) -> Option<PathBuf>) -> std::io::Result<Self> {
        if application_id.is_empty()
            || !application_id.bytes().all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
            || application_id == "."
            || application_id == ".."
        {
            return Err(std::io::Error::other("invalid application identifier"));
        }
        let absolute = |key| get(key).filter(|path| path.is_absolute());
        let base = if windows {
            absolute("LOCALAPPDATA").ok_or_else(|| std::io::Error::other("LOCALAPPDATA must be an absolute directory"))?
        } else {
            absolute("XDG_DATA_HOME")
                .or_else(|| absolute("HOME").map(|home| home.join(".local/share")))
                .ok_or_else(|| std::io::Error::other("XDG_DATA_HOME or HOME must be an absolute directory"))?
        };
        Ok(Self::new(base.join(application_id)))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
