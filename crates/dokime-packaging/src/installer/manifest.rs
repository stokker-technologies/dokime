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
use std::path::{Path, PathBuf};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub installer: Distribution,
    pub dokime: Artifact,
    pub signing: Option<Signing>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Distribution {
    pub version: String,
    pub organization: Option<PathBuf>,
    pub formats: Vec<Format>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub version: String,
    pub target: String,
    pub release_url: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Signing {
    pub installer: Option<InstallerSigning>,
    pub updater: Option<UpdaterSigning>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InstallerSigning {
    pub certificate_thumbprint: Option<String>,
    pub key_id: Option<String>,
    pub timestamp_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdaterSigning {
    pub private_key_file: Option<PathBuf>,
    pub private_key_env: Option<String>,
    pub private_key_path_env: Option<String>,
    pub password_env: String,
    pub artifact_base_url: String,
}

/* === Enums === */

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Nsis,
    Deb,
    Rpm,
}

/* === Traits === */

/* === Implementations === */

impl Manifest {
    pub fn read(path: &Path) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)?;
        let mut manifest: Self =
            toml::from_str(&text).map_err(|_| Error::Invalid("invalid installer manifest; check TOML syntax and supported keys".into()))?;
        let base = path.parent().unwrap_or(Path::new("."));
        if let Some(folder) = &mut manifest.installer.organization {
            *folder = base.join(&*folder);
        }
        let Some(signing) = &mut manifest.signing else {
            return Ok(manifest);
        };
        if let Some(updater) = &mut signing.updater
            && let Some(path) = &mut updater.private_key_file
        {
            *path = base.join(&*path);
        }
        Ok(manifest)
    }
    pub fn validate(&self) -> Result<(), Error> {
        for version in [&self.installer.version, &self.dokime.version] {
            if cargo_metadata::semver::Version::parse(version).is_err() {
                return Err(Error::Invalid("versions must be exact semantic versions".into()));
            }
        }
        let windows = self.dokime.target == "x86_64-pc-windows-msvc";
        let linux = self.dokime.target == "x86_64-unknown-linux-gnu";
        if !(windows || linux) {
            return Err(Error::Invalid("unsupported installer target".into()));
        }
        if (windows && !cfg!(target_os = "windows")) || (linux && !cfg!(target_os = "linux")) {
            return Err(Error::Invalid("installer assembly requires the matching native platform".into()));
        }
        if self.installer.formats.is_empty() {
            return Err(Error::Invalid("at least one installer format is required".into()));
        }
        for (index, format) in self.installer.formats.iter().enumerate() {
            if self.installer.formats[..index].contains(format) || (windows != (*format == Format::Nsis)) {
                return Err(Error::Invalid("duplicate or non-native installer format".into()));
            }
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_formats() {
        let format = if cfg!(target_os = "windows") { "nsis" } else { "deb" };
        let target = if cfg!(target_os = "windows") { "x86_64-pc-windows-msvc" } else { "x86_64-unknown-linux-gnu" };
        let text = format!("[installer]\nversion='1.2.3'\nformats=['{format}','{format}']\n[dokime]\nversion='1.2.3'\ntarget='{target}'");
        let manifest: Manifest = toml::from_str(&text).unwrap();
        assert!(manifest.validate().is_err());
    }
}
