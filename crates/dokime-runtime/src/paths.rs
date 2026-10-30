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

pub use dokime_domain::model::identity::distribution::DistributionIdentity;

/* === Imports === */

// Standard Library Imports
use std::{
    env, io,
    path::{Path, PathBuf},
};

// Third Party Imports

// Local Imports

/* === Constants === */

pub const DEFAULT_APPLICATION_ID: &str = "com.stokker.dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct ApplicationPaths {
    pub root: PathBuf,
    logs: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ApplicationPaths {
    pub fn resolve(application_id: &str) -> std::io::Result<Self> {
        let executable = env::current_exe()?;
        Self::resolve_for(&executable, application_id, None)
    }

    pub fn resolve_for(executable: &Path, compiled_identifier: &str, root: Option<PathBuf>) -> io::Result<Self> {
        let identity = distribution_identity(executable, compiled_identifier)?;
        Self::resolve_identifier(&identity.identifier, root)
    }

    pub fn resolve_identifier(identifier: &str, root: Option<PathBuf>) -> io::Result<Self> {
        let root = match root {
            Some(root) => Some(root),
            None => env::var_os("DOKIME_APPLICATION_ROOT").filter(|value| !value.is_empty()).map(PathBuf::from),
        };
        let environment = |key: &str| env::var_os(key).map(PathBuf::from);
        Self::resolve_with(identifier, root, cfg!(windows), environment)
    }

    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let logs = root.join("logs");
        Self { root, logs }
    }

    pub fn logs(&self) -> PathBuf {
        self.logs.clone()
    }

    pub fn webview(&self) -> PathBuf {
        self.root.join(if cfg!(windows) { "EBWebView" } else { "webview" })
    }

    pub fn plugins(&self) -> PathBuf {
        self.default_data().join("plugins")
    }

    pub fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }

    pub fn default_data(&self) -> PathBuf {
        self.root.join("data")
    }

    fn resolve_with(application_id: &str, root: Option<PathBuf>, windows: bool, get: impl Fn(&str) -> Option<PathBuf>) -> io::Result<Self> {
        DistributionIdentity::new(application_id).map_err(io::Error::other)?;
        if let Some(root) = root {
            let root = std::path::absolute(root)?;
            return Ok(Self::new(root));
        }
        let absolute = |key| {
            let path = get(key)?;
            path.is_absolute().then_some(path)
        };
        let home = absolute("HOME");
        let data_fallback = home.as_ref().map(|home| home.join(".local/share"));
        let base = if windows { absolute("LOCALAPPDATA") } else { absolute("XDG_DATA_HOME").or(data_fallback) };
        let base = base.ok_or_else(|| io::Error::other("the application data home must be an absolute directory"))?;
        let mut paths = Self::new(base.join(application_id));
        if windows {
            return Ok(paths);
        }

        let state_fallback = home.map(|home| home.join(".local/state"));
        let state = absolute("XDG_STATE_HOME").or(state_fallback);
        let state = state.ok_or_else(|| io::Error::other("XDG_STATE_HOME or HOME must be an absolute directory"))?;
        paths.logs = state.join(application_id).join("logs");
        Ok(paths)
    }
}

/* === Public Functions === */

pub fn distribution_identity(executable: &Path, compiled_identifier: &str) -> io::Result<DistributionIdentity> {
    let directory = executable.parent().ok_or_else(|| io::Error::other("executable has no parent directory"))?;
    let bytes = match std::fs::read(directory.join("distribution.json")) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return DistributionIdentity::new(compiled_identifier).map_err(io::Error::other);
        }
        Err(error) => return Err(error),
    };
    let identity: DistributionIdentity = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    identity.validate().map_err(io::Error::other)?;
    Ok(identity)
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod distribution_tests {
    use super::*;

    #[test]
    fn separate_roots() {
        for windows in [false, true] {
            let get = |key: &str| Some(PathBuf::from(if cfg!(windows) { "C:/user" } else { "/user" }).join(key));
            let plain = ApplicationPaths::resolve_with(DEFAULT_APPLICATION_ID, None, windows, get).unwrap();
            let branded = ApplicationPaths::resolve_with("com.hilti.dokime", None, windows, get).unwrap();
            for (plain, branded) in [
                (plain.root.clone(), branded.root.clone()),
                (plain.default_data(), branded.default_data()),
                (plain.plugins(), branded.plugins()),
                (plain.logs(), branded.logs()),
                (plain.webview(), branded.webview()),
            ] {
                assert_ne!(plain, branded);
                assert!(branded.to_string_lossy().contains("com.hilti.dokime"));
            }
            assert_eq!(branded.logs(), get(if windows { "LOCALAPPDATA" } else { "XDG_STATE_HOME" }).unwrap().join("com.hilti.dokime/logs"));
        }
    }

    #[test]
    fn unix_fallbacks() {
        let home = PathBuf::from(if cfg!(windows) { "C:/user" } else { "/user" });
        let get = |key: &str| if key == "HOME" { Some(home.clone()) } else { Some(PathBuf::from("relative")) };
        let paths = ApplicationPaths::resolve_with(DEFAULT_APPLICATION_ID, None, false, get).unwrap();
        assert_eq!(paths.root, home.join(".local/share").join(DEFAULT_APPLICATION_ID));
        assert_eq!(paths.logs(), home.join(".local/state").join(DEFAULT_APPLICATION_ID).join("logs"));
    }

    #[test]
    fn explicit_root() {
        let root = PathBuf::from(if cfg!(windows) { "C:/isolated" } else { "/isolated" });
        let paths = ApplicationPaths::resolve_with(DEFAULT_APPLICATION_ID, Some(root.clone()), cfg!(windows), |_| None).unwrap();
        assert_eq!(paths.root, root);
        assert_eq!(paths.logs(), root.join("logs"));
        assert_eq!(paths.plugins(), root.join("data/plugins"));
        assert!(ApplicationPaths::resolve_with(DEFAULT_APPLICATION_ID, None, cfg!(windows), |_| Some("relative".into())).is_err());
    }
}
