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
use std::env::consts::EXE_SUFFIX;
use std::{collections::BTreeMap, fs, path::Path};

// Third Party Imports
use image::{ImageFormat, Rgb, RgbImage};

// Local Imports
use super::{Error, distribution};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn kit(directory: &Path) -> Result<(), Error> {
    fs::create_dir_all(directory.join("bin"))?;
    fs::create_dir_all(directory.join("organization"))?;
    let executable = super::executable();
    let mut files = BTreeMap::new();
    for name in ["dokime", "dokime-gui"] {
        let name = format!("bin/{name}{}", EXE_SUFFIX);
        fs::write(directory.join(&name), &executable)?;
        files.insert(name, distribution::digest(&executable));
    }
    for name in ["LICENSE", "NOTICE", "THIRD-PARTY-NOTICES"] {
        fs::write(directory.join(name), name)?;
        files.insert(name.to_owned(), distribution::digest(name.as_bytes()));
    }
    organization(&directory.join("organization"), "Example Test Hub")?;
    for name in ["organization.toml", "icon.png"] {
        let name = format!("organization/{name}");
        files.insert(name.clone(), distribution::digest(&fs::read(directory.join(name))?));
    }
    let value = serde_json::json!({"schema":1,"version":"1.2.3","target":crate::bundle::native_target(),"files":files});
    fs::write(directory.join("distribution.json"), serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}

pub(super) fn organization(directory: &Path, label: &str) -> Result<(), Error> {
    fs::create_dir_all(directory)?;
    let text = format!("[app]\nlabel={label:?}\n[app.artwork]\napplication='icon.png'\ninstaller='icon.png'\n");
    fs::write(directory.join("organization.toml"), text)?;
    let pixels = RgbImage::from_pixel(2, 2, Rgb([12, 34, 56]));
    pixels.save_with_format(directory.join("icon.png"), ImageFormat::Png).map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(())
}

pub(super) fn manifest() -> super::Manifest {
    let format = if cfg!(target_os = "windows") { "nsis" } else { "deb" };
    toml::from_str(&format!(
        "[installer]\nversion='1.2.3'\nidentifier='com.example.testhub'\nformats=['{format}']\n[dokime]\nversion='1.2.3'\n"
    ))
    .unwrap()
}

/* === Private Functions === */

/* === Tests === */
