/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Validate Installer Organization Inputs

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::str;
use std::{collections::BTreeMap, fs, path::Path};

// Third Party Imports
use image::ImageFormat;

// Local Imports
use super::policy::UpdatePolicy;
use super::{Error, distribution};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct ResolvedPack {
    pub manifest: toml::Value,
    pub files: BTreeMap<String, Vec<u8>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ResolvedPack {
    pub fn label(&self) -> &str {
        self.manifest["app"]["label"].as_str().expect("validated label")
    }
    pub fn organization_name(&self) -> Option<&str> {
        self.manifest.get("app")?.get("organization")?.get("name")?.as_str()
    }
    pub fn asset(&self, role: &str) -> Option<(&str, &[u8])> {
        let path = self.manifest.get("app")?.get("artwork")?.get(role)?.as_str()?;
        Some((path, self.files.get(path)?.as_slice()))
    }
    pub fn installer_sidebar_color(&self) -> Result<[u8; 3], Error> {
        let Some(value) = self.manifest.get("installer").and_then(|value| value.get("sidebar_color")) else {
            return Ok([0x1b, 0x19, 0x15]);
        };
        let color = value.as_str().ok_or_else(|| Error::Invalid("Set installer.sidebar_color to #RRGGBB".into()))?;
        if color.len() != 7 || !color.starts_with('#') || !color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit) {
            return Err(Error::Invalid("Set installer.sidebar_color to #RRGGBB".into()));
        }
        let mut channels = [0; 3];
        for (index, channel) in channels.iter_mut().enumerate() {
            *channel = u8::from_str_radix(&color[1 + index * 2..3 + index * 2], 16).map_err(|error| Error::Invalid(error.to_string()))?;
        }
        Ok(channels)
    }
    pub fn installer_sidebar_image(&self) -> Option<(&str, &[u8])> {
        let path = self.manifest.get("installer")?.get("sidebar_image")?.as_str()?;
        Some((path, self.files.get(path)?.as_slice()))
    }
    pub fn write_to(&self, destination: &Path) -> Result<(), Error> {
        for (name, bytes) in &self.files {
            let path = destination.join(distribution::relative(name)?);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, bytes)?;
        }
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn read(directory: &Path) -> Result<ResolvedPack, Error> {
    let directory = directory.canonicalize()?;
    let mut files = BTreeMap::new();
    let mut pending = vec![directory.clone()];
    while let Some(folder) = pending.pop() {
        for entry in fs::read_dir(folder)? {
            let entry = entry?;
            let path = entry.path();
            let kind = entry.file_type()?;
            if kind.is_symlink() || !path.canonicalize()?.starts_with(&directory) {
                return Err(Error::Invalid("Replace Organization Links With Regular Files Inside the Pack".into()));
            }
            let name = path
                .strip_prefix(&directory)
                .map_err(|error| Error::Invalid(error.to_string()))?
                .to_str()
                .ok_or_else(|| Error::Invalid("Use UTF-8 Organization Paths".into()))?
                .replace('\\', "/");
            distribution::relative(&name)?;
            if name == "generated" || name.starts_with("generated/") {
                return Err(Error::Invalid("Remove Generated Files From the Authored Organization Pack".into()));
            }
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            if !kind.is_file() {
                return Err(Error::Invalid("Use Only Regular Organization Files".into()));
            }
            files.insert(name, fs::read(path)?);
        }
    }
    let bytes = files.get("organization.toml").ok_or_else(|| Error::Invalid("Add organization.toml to the Organization Pack".into()))?;
    let text = str::from_utf8(bytes).map_err(|_| Error::Invalid("Save organization.toml as UTF-8".into()))?;
    let manifest: toml::Value = toml::from_str(text)
        .map_err(|error| Error::Invalid(format!("Correct {}: {error}", directory.join("organization.toml").display())))?;
    let pack = ResolvedPack { manifest, files };
    let app = pack
        .manifest
        .get("app")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| Error::Invalid("Add an [app] Table to organization.toml".into()))?;
    if ["identifier", "slug", "deep_link_scheme"].iter().any(|field| app.contains_key(*field)) {
        return Err(Error::Invalid("Set Distribution Identity in installer.toml, Not the Organization Pack".into()));
    }
    let label =
        app.get("label").and_then(toml::Value::as_str).ok_or_else(|| Error::Invalid("Set app.label in organization.toml".into()))?;
    super::native::validate_label(label)?;
    if let Some(artwork) = app.get("artwork") {
        let artwork = artwork.as_table().ok_or_else(|| Error::Invalid("Use an app.artwork Table".into()))?;
        for (role, value) in artwork {
            if !["application", "tray", "installer"].contains(&role.as_str()) {
                return Err(Error::Invalid(format!("Remove Unsupported Artwork Role {role}")));
            }
            image_asset(&pack, value)?;
        }
    }
    if let Some(installer) = pack.manifest.get("installer") {
        let installer = installer.as_table().ok_or_else(|| Error::Invalid("Use an installer Table in organization.toml".into()))?;
        pack.installer_sidebar_color()?;
        if let Some(value) = installer.get("sidebar_image") {
            image_asset(&pack, value)?;
        }
    }
    if let Some(updates) = pack.manifest.get("updates") {
        validate_updates(updates)?;
    }
    // Validate Every Referenced Logo Without Inheriting Default Assets.
    if let Some(logos) = app.get("logos") {
        let logos = logos.as_table().ok_or_else(|| Error::Invalid("Use an app.logos Table".into()))?;
        for group in logos.values() {
            for value in group.as_table().ok_or_else(|| Error::Invalid("Use Logo Variant Tables".into()))?.values() {
                asset(&pack, value)?;
            }
        }
    }
    Ok(pack)
}

/* === Private Functions === */

fn validate_updates(value: &toml::Value) -> Result<(), Error> {
    let policy: UpdatePolicy =
        value.clone().try_into().map_err(|_| Error::Invalid("Correct the Organization Update Policy Schema".into()))?;
    policy.validate().map_err(Error::Invalid)
}

fn asset<'a>(pack: &'a ResolvedPack, value: &toml::Value) -> Result<&'a [u8], Error> {
    let name = value.as_str().ok_or_else(|| Error::Invalid("Set Asset Paths to Pack-Relative Strings".into()))?;
    distribution::relative(name)?;
    pack.files.get(name).map(Vec::as_slice).ok_or_else(|| Error::Invalid(format!("Add Referenced Organization Asset {name}")))
}
fn image_asset(pack: &ResolvedPack, value: &toml::Value) -> Result<(), Error> {
    let bytes = asset(pack, value)?;
    image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .map_err(|error| Error::Invalid(format!("Provide Valid PNG Organization Artwork: {error}")))?;
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installer::fixtures;

    #[test]
    fn complete_pack_refuses_invalid_identity_assets_sidebar_and_update_policy() -> Result<(), Error> {
        let root = tempfile::tempdir()?;
        fixtures::organization(root.path(), "Example Test Hub")?;
        let original = fs::read_to_string(root.path().join("organization.toml"))?;
        assert_eq!(read(root.path())?.label(), "Example Test Hub");
        for extra in [
            "[installer]\nsidebar_color='red'",
            "[installer]\nsidebar_image='../outside.png'",
            "[updates]\nmode='manual'",
            "[updates]\nmode='disabled'\nunknown=true",
        ] {
            fs::write(root.path().join("organization.toml"), format!("{original}\n{extra}"))?;
            assert!(read(root.path()).is_err());
        }
        for invalid in [
            original.replace("label=\"Example Test Hub\"", "identifier='com.example.testhub'\nlabel='Example Test Hub'"),
            original.replace("icon.png", "missing.png"),
            original.replace("icon.png", "../icon.png"),
            original.replace("Example Test Hub", "../example-pack"),
        ] {
            fs::write(root.path().join("organization.toml"), invalid)?;
            assert!(read(root.path()).is_err());
        }
        fs::write(root.path().join("organization.toml"), &original)?;
        fs::write(root.path().join("icon.png"), b"not a PNG")?;
        assert!(read(root.path()).is_err());
        Ok(())
    }
}
