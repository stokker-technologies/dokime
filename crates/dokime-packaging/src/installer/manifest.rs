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

    #[serde(default)]
    pub plugin: Vec<Plugin>,

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
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plugin {
    pub name: Option<String>,
    pub path: Option<PathBuf>,
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

impl Plugin {
    fn validate(&self, index: usize) -> Result<(), Error> {
        let error = |reason| {
            let message = format!("plugin entry {index}: {reason}");
            Error::Invalid(message)
        };
        match (&self.name, &self.path) {
            (Some(name), None) if !name.trim().is_empty() => Ok(()),
            (None, Some(path)) => {
                let Some(path) = path.to_str() else { return Err(error("path must be UTF-8")) };
                let checked = super::distribution::relative(path);
                checked.map_err(|_| error("path must be relative to and stay inside the organization pack"))?;
                Ok(())
            }
            (Some(_), None) => Err(error("name must not be empty")),
            _ => Err(error("set exactly one of name or path")),
        }
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
        let mut manifest: Self = toml::from_str(&text).map_err(|error| parse_error(path, &error))?;
        let base = path.parent().unwrap_or(Path::new("."));
        manifest.validate_plugins()?;
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
        self.validate_plugins()?;
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

    pub(super) fn plugin_paths(&self, default_organization: &Path) -> Result<Vec<PathBuf>, Error> {
        self.validate_plugins()?;
        let organization = self.installer.organization.as_deref().unwrap_or(default_organization);
        let mut paths = Vec::new();
        for (index, plugin) in self.plugin.iter().enumerate() {
            let Some(relative) = &plugin.path else { continue };
            let root = organization.canonicalize()?;
            let path = root.join(relative);
            if !path.canonicalize()?.starts_with(&root) {
                let message = format!("plugin entry {} escapes the organization pack", index + 1);
                return Err(Error::Invalid(message));
            }
            paths.push(path);
        }
        Ok(paths)
    }

    fn validate_plugins(&self) -> Result<(), Error> {
        for (index, plugin) in self.plugin.iter().enumerate() {
            plugin.validate(index + 1)?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn parse_error(path: &Path, error: &toml::de::Error) -> Error {
    let key = error.message().strip_prefix("unknown field `").and_then(|message| message.split('`').next());
    let hint = match key {
        Some("release_url" | "sha256") => "\nhint: select the release with [dokime] version instead of release_url and sha256.",
        Some("external_plugins") => "\nhint: use top-level [[plugin]] entries with path instead of [[dokime.external_plugins]].",
        Some("plugins") => "\nhint: use top-level [[plugin]] entries with name or path instead of [dokime] plugins.",
        _ => "",
    };
    Error::Invalid(format!("invalid installer manifest {}: {error}{hint}", path.display()))
}

fn native_target() -> String {
    dokime::plugins::NATIVE_TARGET.into()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_parse_diagnostics() {
        let path = Path::new("company/installer.toml");
        for (entry, key, replacements) in [
            ("release_url='https://example.test/archive.zip'", "release_url", &["[dokime]", "version"][..]),
            ("sha256='digest'", "sha256", &["[dokime]", "version"][..]),
            ("plugins=['opaque-name']", "plugins", &["[[plugin]]", "name", "path"][..]),
            ("[[dokime.external_plugins]]\npath='plugin.dokime'", "external_plugins", &["[[plugin]]", "path"][..]),
            ("unexpected=true", "unexpected", &[][..]),
            ("target=42", "target", &[][..]),
            ("target=[", "target", &[][..]),
        ] {
            let text = format!("[installer]\nversion='1.2.3'\nformats=['nsis']\n[dokime]\n{entry}");
            let source = toml::from_str::<Manifest>(&text).unwrap_err();
            assert!(source.span().is_some());
            let error = parse_error(path, &source).to_string();
            assert!(error.contains(&path.display().to_string()), "{error}");
            assert!(error.contains(key), "{error}");
            assert!(error.contains("line 5, column"), "{error}");
            assert!(error.contains(&source.to_string()), "{error}");
            for replacement in replacements {
                assert!(error.contains(replacement), "{error}");
            }
        }
    }

    #[test]
    fn release_manifest() {
        let text = "[installer]\nversion='1.2.3'\nformats=['nsis']\n[dokime]\n[[plugin]]\nname='opaque-name'\n[[plugin]]\npath='plugins/plugin.dokime'";
        let manifest: Manifest = toml::from_str(text).unwrap();
        manifest.validate_plugins().unwrap();
        assert_eq!(manifest.dokime.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(manifest.plugin[0].name.as_deref(), Some("opaque-name"));
        assert_eq!(manifest.plugin[1].path.as_deref(), Some(Path::new("plugins/plugin.dokime")));
    }

    #[test]
    fn plugin_entry_validation() {
        let base = "[installer]\nversion='1.2.3'\nformats=['nsis']\n[dokime]\n";
        let empty: Manifest = toml::from_str(base).unwrap();
        empty.validate_plugins().unwrap();
        assert!(empty.plugin.is_empty());

        for entry in [
            "",
            "name='plugin-host'\npath='plugins/plugin-host.dokime'",
            "name=''",
            "path='../outside.dokime'",
            "path='/plugin-host.dokime'",
            "path='C:/plugin-host.dokime'",
            "path='plugins\\plugin-host.dokime'",
        ] {
            let text = format!("{base}[[plugin]]\nname='valid'\n[[plugin]]\n{entry}");
            let manifest: Manifest = toml::from_str(&text).unwrap();
            let error = manifest.validate_plugins().unwrap_err();
            assert!(error.to_string().contains("entry 2"), "{error}");
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
