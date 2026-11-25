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

pub(super) use crate::organization::ResolvedPack;

/* === Imports === */

// Standard Library Imports
use std::{fs, path::Path};

// Third Party Imports

// Local Imports
use super::{Error, distribution};
use crate::organization;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

/// Read the Pack Through the Shared Loader, Refusing Any Appearance the Desktop Could Not Apply.
pub(super) fn read(directory: &Path) -> Result<ResolvedPack, Error> {
    let pack = organization::read(directory).map_err(Error::Invalid)?;
    pack.appearance().map_err(Error::Invalid)?;
    validate_label(pack.label())?;
    Ok(pack)
}

/* === Private Functions === */

// The Label Names Installed Files and Shortcuts, So It Must Be a Safe Native Filename.
fn validate_label(label: &str) -> Result<(), Error> {
    let trimmed = label.trim();
    let invalid_start = trimmed.is_empty() || trimmed != label || label.ends_with('.');
    let invalid = invalid_start
        || [".", ".."].contains(&label)
        || label.chars().any(|character| {
            let control = character.is_control();
            let reserved = "/\\:*?\"<>|".contains(character);
            control || reserved
        });
    if invalid {
        let error = Error::Invalid("distribution label must be a safe native filename; slashes, control characters and reserved filename characters are not allowed".into());
        return Err(error);
    }
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
        for extra in
            ["[app.organization]\nname=5", "[app.organization]\nname='Example Test Hub'\nunit=5", "[theme.corner_radii]\nbase=false"]
        {
            fs::write(root.path().join("organization.toml"), format!("{original}\n{extra}"))?;
            assert!(read(root.path()).is_err());
        }
        fs::write(root.path().join("organization.toml"), &original)?;
        fs::create_dir(root.path().join("theme"))?;
        for css in [
            "@import 'outside.css';",
            "a { src: url(https://example.test/icon.png) }",
            "a { src: url(../icon.png) }",
            "a { src: url(missing.png) }",
            "/* unclosed",
            "a { src: url(icon.png",
        ] {
            fs::write(root.path().join("theme/base.css"), css)?;
            assert!(read(root.path()).is_err());
        }
        fs::write(root.path().join("theme/base.css"), "/* @import and url(missing.png) */ a { background: url(icon.png) }")?;
        read(root.path())?;
        fs::write(root.path().join("icon.png"), b"not a PNG")?;
        assert!(read(root.path()).is_err());
        Ok(())
    }
}
