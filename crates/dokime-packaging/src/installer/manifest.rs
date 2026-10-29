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
    pub identifier: Option<String>,
    pub organization: Option<PathBuf>,
    pub formats: Vec<Format>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub version: Option<String>,

    #[serde(default = "native_target")]
    pub target: String,

    #[serde(default)]
    pub plugins: Vec<String>,

    #[serde(default)]
    pub external_plugins: Vec<ExternalPlugin>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalPlugin {
    pub path: PathBuf,
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

impl Artifact {
    pub fn version(&self) -> &str {
        self.version.as_deref().unwrap_or(env!("CARGO_PKG_VERSION"))
    }
}

impl Manifest {
    pub(super) fn identity(&self, compiled: &super::distribution::Identity) -> Result<super::distribution::Identity, Error> {
        let mut identity = compiled.clone();
        if let Some(identifier) = &self.installer.identifier {
            let distribution =
                dokime::paths::DistributionIdentity::new(identifier.clone()).map_err(|error| Error::Invalid(error.to_string()))?;
            identity.slug = distribution.slug();
            identity.identifier = distribution.identifier;
        }
        Ok(identity)
    }

    pub fn read(path: &Path) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)?;
        let mut manifest: Self =
            toml::from_str(&text).map_err(|_| Error::Invalid("invalid installer manifest; check TOML syntax and supported keys".into()))?;
        let base = path.parent().unwrap_or(Path::new("."));
        for plugin in &mut manifest.dokime.external_plugins {
            plugin.path = base.join(&plugin.path);
        }
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
        if let Some(identifier) = &self.installer.identifier {
            dokime::paths::DistributionIdentity::new(identifier.clone()).map_err(|error| Error::Invalid(error.to_string()))?;
        }
        let installer_version = cargo_metadata::semver::Version::parse(&self.installer.version);
        let version = self.dokime.version();
        if installer_version.is_err() || (version != "dev-channel" && cargo_metadata::semver::Version::parse(version).is_err()) {
            return Err(Error::Invalid("versions must be exact semantic versions or Dokime dev-channel".into()));
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

fn native_target() -> String {
    dokime::plugins::NATIVE_TARGET.into()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_manifest() {
        let text = "[installer]\nversion='1.2.3'\nformats=['nsis']\n[dokime]\nplugins=['opaque-name']\n[[dokime.external_plugins]]\npath='plugin.dokime'";
        let manifest: Manifest = toml::from_str(text).unwrap();
        assert_eq!(manifest.dokime.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(manifest.dokime.plugins, ["opaque-name"]);
        assert_eq!(manifest.dokime.external_plugins[0].path, Path::new("plugin.dokime"));

        for retired in ["release_url='https://example.test/archive.zip'", "sha256='digest'"] {
            let text = format!("[installer]\nversion='1.2.3'\nformats=['nsis']\n[dokime]\n{retired}");
            assert!(toml::from_str::<Manifest>(&text).is_err());
        }
    }
    #[test]
    fn distribution_identifier() {
        let text = "[installer]\nversion='1.2.3'\nidentifier='com.hilti.dokime'\nformats=['nsis']\n[dokime]\nversion='1.2.3'\ntarget='x86_64-pc-windows-msvc'";
        let mut manifest: Manifest = toml::from_str(text).unwrap();
        let compiled = super::super::distribution::Identity {
            identifier: "com.stokker.dokime".into(),
            slug: "dokime".into(),
            updater_public_key: "trust".into(),
        };
        let identity = manifest.identity(&compiled).unwrap();
        assert_eq!(identity.identifier, "com.hilti.dokime");
        assert_eq!(identity.slug, "com-hilti-dokime");
        assert_eq!(identity.updater_public_key, compiled.updater_public_key);
        manifest.installer.identifier = None;
        assert_eq!(manifest.identity(&compiled).unwrap().slug, "dokime");
        manifest.installer.identifier = Some("../outside".into());
        assert!(manifest.identity(&compiled).is_err());
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn native_formats() {
        let format = if cfg!(target_os = "windows") { "nsis" } else { "deb" };
        let target = if cfg!(target_os = "windows") { "x86_64-pc-windows-msvc" } else { "x86_64-unknown-linux-gnu" };
        let text = format!("[installer]\nversion='1.2.3'\nformats=['{format}','{format}']\n[dokime]\nversion='1.2.3'\ntarget='{target}'");
        let manifest: Manifest = toml::from_str(&text).unwrap();
        assert!(manifest.validate().is_err());
    }
}
