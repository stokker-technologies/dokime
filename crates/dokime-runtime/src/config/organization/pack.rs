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

use super::{ResolvedBrand, policy};

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

// Third Party Imports

// Local Imports
use super::theme::{Bundled, Source};

/* === Constants === */

const DEFAULT_INSTALLER_SIDEBAR_COLOR: [u8; 3] = [0x1b, 0x19, 0x15];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// A Validated Organization With All Inherited Files Included.
#[derive(Clone, Debug)]
pub struct ResolvedPack {
    pub manifest: toml::Value,
    pub files: BTreeMap<String, Vec<u8>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ResolvedPack {
    pub fn label(&self) -> &str {
        self.manifest.get("app").and_then(|app| app.get("label")).and_then(toml::Value::as_str).unwrap_or("")
    }

    pub fn asset(&self, role: &str) -> Option<(&str, &[u8])> {
        let path = self.manifest.get("app")?.get("artwork")?.get(role)?.as_str()?;
        Some((path, self.files.get(path)?.as_slice()))
    }

    pub fn installer_sidebar_color(&self) -> Result<[u8; 3], String> {
        let Some(installer) = self.manifest.get("installer") else {
            return Ok(DEFAULT_INSTALLER_SIDEBAR_COLOR);
        };
        let installer = installer.as_table().ok_or("organization installer must be a table")?;
        let Some(value) = installer.get("sidebar_color") else {
            return Ok(DEFAULT_INSTALLER_SIDEBAR_COLOR);
        };

        let color = value.as_str().ok_or("installer.sidebar_color must be an opaque #RRGGBB color")?;
        let bytes = color.as_bytes();
        if bytes.len() != 7 || bytes[0] != b'#' || !bytes[1..].iter().all(u8::is_ascii_hexdigit) {
            return Err("installer.sidebar_color must be an opaque #RRGGBB color".into());
        }

        let mut channels = [0; 3];
        for (index, channel) in channels.iter_mut().enumerate() {
            let encoded = &color[1 + index * 2..3 + index * 2];
            *channel = u8::from_str_radix(encoded, 16).map_err(|error| error.to_string())?;
        }
        Ok(channels)
    }

    pub fn installer_sidebar_image(&self) -> Option<(&str, &[u8])> {
        let path = self.manifest.get("installer")?.get("sidebar_image")?.as_str()?;
        let bytes = self.files.get(path)?.as_slice();
        Some((path, bytes))
    }

    pub fn write_to(&self, destination: &Path) -> Result<(), String> {
        validate(self)?;
        for (relative, bytes) in &self.files {
            validate_path(relative)?;
            if relative == "organization.toml" {
                continue;
            }
            let path = destination.join(relative);
            let parent = path.parent().ok_or("organization file has no parent")?;
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            std::fs::write(path, bytes).map_err(|error| error.to_string())?;
        }
        std::fs::create_dir_all(destination).map_err(|error| error.to_string())?;
        let manifest = toml::to_string(&self.manifest).map_err(|error| error.to_string())?;
        std::fs::write(destination.join("organization.toml"), manifest).map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn materialize(&self) -> Result<super::Materialized, String> {
        let text = toml::to_string(&self.manifest).map_err(|error| error.to_string())?;
        super::Materialized::read(&text, self).map_err(|error| error.to_string())
    }

    pub fn brand(&self) -> Result<ResolvedBrand, String> {
        if self.label().trim().is_empty() {
            return Err("organization needs a nonempty app label".into());
        }
        for relative in self.files.keys() {
            validate_path(relative)?;
        }
        super::resolved_pack_brand(self).map_err(|error| error.to_string())
    }
}

impl Source for ResolvedPack {
    fn read(&self, relative: &str) -> std::io::Result<Vec<u8>> {
        self.files.get(relative).cloned().ok_or_else(|| std::io::ErrorKind::NotFound.into())
    }
}

/* === Public Functions === */

pub fn brand() -> Option<ResolvedBrand> {
    super::compiled_brand()
}

/// Overlay Authored Files And Manifest Fields Before Validating The Complete Pack.
pub fn resolve(override_dir: Option<&Path>) -> Result<ResolvedPack, String> {
    let base = installation_directory();
    resolve_with_base(base.as_deref(), override_dir)
}

pub fn resolve_with_base(base_dir: Option<&Path>, override_dir: Option<&Path>) -> Result<ResolvedPack, String> {
    let mut files = match base_dir {
        Some(directory) => {
            let directory = directory.canonicalize().map_err(|error| error.to_string())?;
            let mut files = BTreeMap::new();
            collect_files(&directory, &directory, &mut files)?;
            files
        }
        None => Bundled::files().iter().map(|(name, bytes)| ((*name).to_owned(), bytes.to_vec())).collect(),
    };
    let bytes = files.get("organization.toml").ok_or("the bundled organization has no manifest")?;
    let text = std::str::from_utf8(bytes).map_err(|_| "bundled organization manifest is not UTF-8")?;
    let manifest_path = base_dir.unwrap_or(Path::new("organization")).join("organization.toml");
    let mut manifest = parse_manifest(&manifest_path, text)?;
    if let Some(directory) = override_dir {
        let directory = directory.canonicalize().map_err(|error| error.to_string())?;
        let mut overrides = BTreeMap::new();
        collect_files(&directory, &directory, &mut overrides)?;
        if let Some(bytes) = overrides.remove("organization.toml") {
            let text = std::str::from_utf8(&bytes).map_err(|_| "organization.toml is not UTF-8")?;
            let overlay = parse_manifest(&directory.join("organization.toml"), text)?;
            merge(&mut manifest, overlay);
        }
        files.extend(overrides);
    }
    let text = toml::to_string(&manifest).map_err(|error| error.to_string())?;
    files.insert("organization.toml".into(), text.into_bytes());
    let pack = ResolvedPack { manifest, files };
    validate(&pack)?;
    Ok(pack)
}

pub fn installation_directory() -> Option<std::path::PathBuf> {
    let executable = std::env::current_exe().ok()?;
    let directory = executable.parent()?.join("organization");
    directory.join("organization.toml").is_file().then_some(directory)
}

/* === Private Functions === */

fn parse_manifest(path: &Path, text: &str) -> Result<toml::Value, String> {
    toml::from_str(text).map_err(|error| format!("invalid organization manifest {}: {error}", path.display()))
}

fn validate(pack: &ResolvedPack) -> Result<(), String> {
    let manifest = &pack.manifest;
    let app = manifest.get("app").and_then(toml::Value::as_table).ok_or("organization needs an app table")?;
    if ["identifier", "slug", "deep_link_scheme"].iter().any(|field| app.contains_key(*field)) {
        return Err("compiled application identity is not an organization setting".into());
    }
    if app.get("label").and_then(toml::Value::as_str).is_none_or(|label| label.trim().is_empty()) {
        return Err("organization needs a nonempty app label".into());
    }
    if let Some(artwork) = app.get("artwork") {
        for (role, value) in artwork.as_table().ok_or("app.artwork must be a table")? {
            if !["application", "tray", "installer"].contains(&role.as_str()) {
                return Err(format!("unsupported organization artwork role: {role}"));
            }
            let path = value.as_str().ok_or("artwork paths must be strings")?;
            validate_path(path)?;
            let bytes = pack.files.get(path).ok_or_else(|| format!("organization artwork is missing: {path}"))?;
            image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
                .map_err(|_| format!("organization {role} artwork must be a valid PNG image"))?;
        }
    }
    validate_installer(pack)?;
    if let Some(updates) = manifest.get("updates") {
        let updates: policy::UpdatePolicy = updates.clone().try_into().map_err(|_| "organization update policy is invalid")?;
        updates.validate()?;
    }
    pack.brand()?;
    Ok(())
}

fn validate_installer(pack: &ResolvedPack) -> Result<(), String> {
    pack.installer_sidebar_color()?;
    let Some(installer) = pack.manifest.get("installer") else { return Ok(()) };
    let Some(value) = installer.get("sidebar_image") else { return Ok(()) };
    let path = value.as_str().ok_or("installer.sidebar_image must be a pack-root-relative PNG path")?;
    validate_path(path)?;

    let bytes = pack.files.get(path).ok_or_else(|| format!("organization installer sidebar image is missing: {path}"))?;
    image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|_| "organization installer sidebar image must be a valid PNG image")?;
    Ok(())
}

fn merge(base: &mut toml::Value, overlay: toml::Value) {
    if let (Some(base), Some(overlay)) = (base.as_table_mut(), overlay.as_table()) {
        for (key, value) in overlay {
            match base.get_mut(key) {
                Some(current) => merge(current, value.clone()),
                None => {
                    base.insert(key.clone(), value.clone());
                }
            }
        }
    } else {
        *base = overlay;
    }
}

fn validate_path(relative: &str) -> Result<(), String> {
    if relative == "generated" || relative.starts_with("generated/") {
        return Err("generated files are not authored organization inputs".into());
    }
    let unnormalized = relative.split('/').any(|part| matches!(part, "" | "." | ".."));
    if unnormalized
        || relative.contains(['\\', ':'])
        || Path::new(relative).components().any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe organization path: {relative}"));
    }
    Ok(())
}

fn collect_files(root: &Path, directory: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    for entry in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        if kind.is_symlink() {
            return Err("organization packs may not contain links".into());
        }
        let relative = path.strip_prefix(root).map_err(|error| error.to_string())?.to_string_lossy().replace('\\', "/");
        validate_path(&relative)?;
        if kind.is_dir() {
            collect_files(root, &path, files)?;
        } else if kind.is_file() {
            files.insert(relative, std::fs::read(path).map_err(|error| error.to_string())?);
        }
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_parse_diagnostics() {
        let path = Path::new("company/organization.toml");
        let error = parse_manifest(path, "[app]\nlabel = ?\n").unwrap_err();
        assert!(error.contains(&path.display().to_string()), "{error}");
        assert!(error.contains("label"), "{error}");
        assert!(error.contains("line 2, column"), "{error}");
    }

    #[test]
    fn overlay_values() {
        let mut base: toml::Value = toml::from_str("keep = 1\narray = [1, 2]\n[table]\nleft = 1\nright = 2").unwrap();
        let overlay = toml::from_str("array = [3]\n[table]\nleft = 9").unwrap();
        merge(&mut base, overlay);
        assert_eq!(base["keep"].as_integer(), Some(1));
        assert_eq!(base["array"].as_array().unwrap(), &[toml::Value::Integer(3)]);
        assert_eq!(base["table"]["left"].as_integer(), Some(9));
        assert_eq!(base["table"]["right"].as_integer(), Some(2));
    }
}
