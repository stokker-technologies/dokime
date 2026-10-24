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
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
    process::Command,
};

// Third Party Imports
use tauri_bundler::{BundleBinary, BundleSettings, NsisSettings, PackageSettings, PackageType, SettingsBuilder};

// Local Imports
use super::{
    inspect::{self, DistributionMetadata},
    manifest::{Format, Manifest},
    payload::{self, Payload},
};
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn assemble(
    manifest: &Manifest, payload: &Payload, source: &Path, scratch: &Path, output: &Path, sign: bool,
) -> Result<Vec<PathBuf>, Error> {
    let mut pack =
        dokime::config::organization::resolve_with_base(Some(&source.join("organization")), manifest.installer.organization.as_deref())
            .map_err(Error::Invalid)?;
    if let Some(folder) = &manifest.installer.organization {
        let override_manifest = folder.join("organization.toml");
        let explicit_updates = if override_manifest.exists() {
            let value: toml::Value =
                toml::from_str(&std::fs::read_to_string(override_manifest)?).map_err(|error| Error::Invalid(error.to_string()))?;
            value.get("updates").and_then(|updates| updates.get("mode")).is_some()
        } else {
            false
        };
        if !explicit_updates {
            let root = pack.manifest.as_table_mut().ok_or_else(|| Error::Invalid("organization manifest must be a table".into()))?;
            let updates = root.entry("updates").or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
            updates
                .as_table_mut()
                .ok_or_else(|| Error::Invalid("organization updates must be a table".into()))?
                .insert("mode".into(), toml::Value::String("disabled".into()));
            let bytes = toml::to_string(&pack.manifest).map_err(|error| Error::Invalid(error.to_string()))?.into_bytes();
            pack.files.insert("organization.toml".into(), bytes);
        }
    }
    let pack_dir = scratch.join("organization");
    pack.write_to(&pack_dir).map_err(Error::Invalid)?;
    let label = pack.label().to_owned();
    validate_label(&label)?;
    let windows = payload.target.contains("windows");
    let suffix = if windows { ".exe" } else { "" };
    let bin_dir = scratch.join("build");
    std::fs::create_dir_all(&bin_dir)?;
    let mut files = BTreeMap::new();
    for name in ["dokime", "dokime-gui"] {
        let file = format!("{name}{suffix}");
        let bytes = std::fs::read(source.join("bin").join(&file))?;
        let destination = bin_dir.join(&file);
        std::fs::write(&destination, &bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o755))?;
        }
        files.insert(if windows { file } else { format!("usr/bin/{file}") }, payload::digest(&bytes));
    }
    let mut resources = HashMap::new();
    let mut linux_files = HashMap::new();
    for name in pack.files.keys() {
        let relative = format!("organization/{name}");
        let staged = pack_dir.join(payload::relative(name)?);
        resources.insert(staged.to_string_lossy().into_owned(), relative.clone());
        linux_files.insert(PathBuf::from(format!("/usr/lib/dokime/{relative}")), staged);
        files.insert(
            if windows { relative } else { format!("usr/lib/dokime/{relative}") },
            payload::digest(&std::fs::read(pack_dir.join(payload::relative(name)?))?),
        );
    }
    let license = source.join("LICENSE");
    resources.insert(license.to_string_lossy().into_owned(), "LICENSE".into());
    linux_files.insert(PathBuf::from("/usr/lib/dokime/LICENSE"), license.clone());
    files.insert(if windows { "LICENSE".into() } else { "usr/lib/dokime/LICENSE".into() }, payload::digest(&std::fs::read(&license)?));
    let metadata = DistributionMetadata {
        schema: 1,
        version: manifest.installer.version.clone(),
        dokime_version: payload.version.clone(),
        target: payload.target.clone(),
        label: label.clone(),
        identity: payload.identity.clone(),
        files,
        installer_signed: sign && manifest.signing.as_ref().is_some_and(|settings| settings.installer.is_some()),
        updater_signed: sign && manifest.signing.as_ref().is_some_and(|settings| settings.updater.is_some()),
    };
    let metadata_path = scratch.join("distribution.json");
    std::fs::write(&metadata_path, serde_json::to_vec_pretty(&metadata)?)?;
    resources.insert(metadata_path.to_string_lossy().into_owned(), "distribution.json".into());
    linux_files.insert(PathBuf::from("/usr/lib/dokime/distribution.json"), metadata_path);

    // Generate Assembly Artwork
    let image_path = scratch.join("icon.png");
    let (_, image_bytes) =
        pack.asset("application").ok_or_else(|| Error::Invalid("organization application artwork is required".into()))?;
    let image = image::load_from_memory(image_bytes).map_err(|e| Error::Invalid(e.to_string()))?;
    image.save_with_format(&image_path, image::ImageFormat::Png).map_err(|e| Error::Invalid(e.to_string()))?;
    let ico_path = scratch.join("icon.ico");
    let (_, installer_bytes) =
        pack.asset("installer").ok_or_else(|| Error::Invalid("organization installer artwork is required".into()))?;
    let installer_image = image::load_from_memory(installer_bytes).map_err(|e| Error::Invalid(e.to_string()))?;
    installer_image
        .resize(256, 256, image::imageops::FilterType::Lanczos3)
        .save_with_format(&ico_path, image::ImageFormat::Ico)
        .map_err(|e| Error::Invalid(e.to_string()))?;
    let hooks = scratch.join("installer-hooks.nsh");
    std::fs::write(&hooks, include_str!("hooks.nsh"))?;
    let mut bundle = BundleSettings {
        identifier: Some(payload.identity.identifier.clone()),
        icon: Some(vec![image_path.to_string_lossy().into_owned(), ico_path.to_string_lossy().into_owned()]),
        license: Some("Apache-2.0".into()),
        license_file: Some(license),
        deep_link_protocols: Some(vec![serde_json::from_value(
            serde_json::json!({"schemes":[payload.identity.deep_link_scheme],"name":label}),
        )?]),
        ..BundleSettings::default()
    };
    bundle.windows.nsis = Some(NsisSettings { installer_hooks: Some(hooks), installer_icon: Some(ico_path), ..NsisSettings::default() });
    if windows {
        bundle.resources_map = Some(resources);
    } else {
        bundle.deb.files = linux_files.clone();
        bundle.rpm.files = linux_files;
        bundle.deb.depends = Some(vec![
            "libwebkit2gtk-4.1-0".into(),
            "libgtk-3-0t64".into(),
            "libayatana-appindicator3-1".into(),
            "libasound2t64".into(),
            "libsoup-3.0-0".into(),
            "libjavascriptcoregtk-4.1-0".into(),
            "librsvg2-2".into(),
            "libssl3t64".into(),
        ]);
        bundle.rpm.depends = Some(vec![
            "webkit2gtk4.1".into(),
            "gtk3".into(),
            "libayatana-appindicator-gtk3".into(),
            "alsa-lib".into(),
            "libsoup3".into(),
            "javascriptcoregtk4.1".into(),
            "librsvg2".into(),
            "openssl-libs".into(),
        ]);
    }
    let types = manifest
        .installer
        .formats
        .iter()
        .map(|format| match format {
            Format::Nsis => PackageType::Nsis,
            Format::Deb => PackageType::Deb,
            Format::Rpm => PackageType::Rpm,
        })
        .collect();
    let settings = SettingsBuilder::new()
        .project_out_directory(&bin_dir)
        .target(payload.target.clone())
        .package_settings(PackageSettings {
            product_name: label,
            version: manifest.installer.version.clone(),
            description: "Dokime Test Hub".into(),
            homepage: None,
            authors: Some(vec!["Stokker Technologies".into()]),
            default_run: Some("dokime-gui".into()),
        })
        .bundle_settings(bundle)
        .binaries(vec![BundleBinary::new("dokime-gui".into(), true), BundleBinary::new("dokime".into(), false)])
        .package_types(types)
        .binary_patching(false)
        .no_sign(true)
        .build()
        .map_err(|e| Error::Invalid(e.to_string()))?;
    let bundles = tauri_bundler::bundle_project(&settings).map_err(|e| Error::Invalid(e.to_string()))?;
    let mut outputs = Vec::new();
    for bundle in bundles {
        for source in bundle.bundle_paths {
            let name = match bundle.package_type {
                PackageType::Nsis => format!("dokime-{}-windows-x86_64-setup.exe", manifest.installer.version),
                PackageType::Deb => format!("dokime_{}_amd64.deb", manifest.installer.version),
                PackageType::Rpm => format!("dokime-{}-1.x86_64.rpm", manifest.installer.version),
                _ => return Err(Error::Invalid("unexpected native package type".into())),
            };
            if sign {
                sign_installer(manifest, &source)?;
            }
            let destination = output.join(name);
            std::fs::copy(&source, &destination)?;
            if sign {
                super::signing::updater(manifest, payload, &destination, output)?;
            }
            inspect::inspect(&destination)?;
            outputs.push(destination);
        }
    }
    Ok(outputs)
}

pub fn validate_signing(manifest: &Manifest, requested: bool) -> Result<(), Error> {
    if manifest.installer.formats.contains(&Format::Rpm)
        && ["TAURI_SIGNING_RPM_KEY", "TAURI_SIGNING_RPM_KEY_PASSPHRASE"].iter().any(|name| std::env::var_os(name).is_some())
    {
        return Err(Error::Invalid("unset inherited TAURI_SIGNING_RPM_KEY and TAURI_SIGNING_RPM_KEY_PASSPHRASE; RPM signing uses explicit --sign and signing.installer.key_id".into()));
    }
    if !requested {
        return Ok(());
    }
    let signing = manifest.signing.as_ref().ok_or_else(|| Error::Invalid("--sign requires explicit signing configuration".into()))?;
    if signing.installer.is_none() && signing.updater.is_none() {
        return Err(Error::Invalid("--sign requires installer or updater signing settings".into()));
    }
    if let Some(installer) = &signing.installer {
        if cfg!(windows) {
            let valid_thumbprint = installer
                .certificate_thumbprint
                .as_deref()
                .is_some_and(|value| value.len() == 40 && value.bytes().all(|c| c.is_ascii_hexdigit()));
            if !valid_thumbprint
                || !installer.timestamp_url.as_deref().is_some_and(|value| value.starts_with("https://"))
                || installer.key_id.is_some()
            {
                return Err(Error::Invalid(
                    "Windows installer signing requires a certificate-store SHA-1 thumbprint and HTTPS timestamp URL".into(),
                ));
            }
        } else if !installer.key_id.as_deref().is_some_and(|value| !value.is_empty() && value.bytes().all(|c| c.is_ascii_hexdigit()))
            || installer.certificate_thumbprint.is_some()
            || installer.timestamp_url.is_some()
        {
            return Err(Error::Invalid(
                "Linux installer signing requires a GPG key_id; credentials are supplied by the external GPG agent".into(),
            ));
        }
    }
    if let Some(updater) = &signing.updater {
        if manifest.installer.version != manifest.dokime.version {
            return Err(Error::Invalid(
                "updater signing requires matching installer and Dokime versions; distinct company versions require manual deployment"
                    .into(),
            ));
        }
        let references = usize::from(updater.private_key_file.is_some())
            + usize::from(updater.private_key_env.is_some())
            + usize::from(updater.private_key_path_env.is_some());
        let key_available = updater.private_key_file.as_ref().is_some_and(|path| path.is_file())
            || updater.private_key_env.as_ref().is_some_and(|name| std::env::var(name).is_ok_and(|value| !value.trim().is_empty()))
            || updater
                .private_key_path_env
                .as_ref()
                .is_some_and(|name| std::env::var_os(name).is_some_and(|path| Path::new(&path).is_file()));
        if references != 1 || !key_available {
            return Err(Error::Invalid("updater signing requires exactly one available private key file/environment reference".into()));
        }

        if !cfg!(windows) || std::env::var(&updater.password_env).is_err() || !updater.artifact_base_url.starts_with("https://") {
            return Err(Error::Invalid(
                "updater signing requires native Windows, a key file, external password environment and HTTPS artifact URL".into(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_label(label: &str) -> Result<(), Error> {
    if label.trim().is_empty()
        || label.trim() != label
        || label.ends_with('.')
        || [".", ".."].contains(&label)
        || label.chars().any(|character| character.is_control() || "/\\:*?\"<>|".contains(character))
    {
        return Err(Error::Invalid("distribution label must be a safe native filename; slashes, control characters and reserved filename characters are not allowed".into()));
    }
    Ok(())
}

/* === Private Functions === */

fn sign_installer(manifest: &Manifest, path: &Path) -> Result<(), Error> {
    let Some(settings) = manifest.signing.as_ref().and_then(|s| s.installer.as_ref()) else {
        return Ok(());
    };
    if cfg!(windows) {
        let thumbprint = settings.certificate_thumbprint.as_deref().ok_or_else(|| Error::Invalid("missing signing certificate".into()))?;
        let timestamp = settings.timestamp_url.as_deref().ok_or_else(|| Error::Invalid("missing signing timestamp URL".into()))?;
        inspect::run(
            Command::new("signtool").args(["sign", "/sha1", thumbprint, "/fd", "SHA256", "/tr", timestamp, "/td", "SHA256"]).arg(path),
            "installer signing requires signtool and the configured certificate in the current user store",
        )?;
        return inspect::run(Command::new("signtool").args(["verify", "/pa"]).arg(path), "installer Authenticode verification failed");
    }
    let key = settings.key_id.as_deref().ok_or_else(|| Error::Invalid("missing Linux signing key".into()))?;
    match path.extension().and_then(|s| s.to_str()) {
        Some("deb") => {
            inspect::run(
                Command::new("debsigs").args(["--sign=origin", "--default-key", key]).arg(path),
                "DEB signing requires debsigs and an available GPG agent",
            )?;
            inspect::run(
                Command::new("debsig-verify").arg(path),
                "DEB signature verification requires debsig-verify with the signing policy installed",
            )
        }
        Some("rpm") => {
            inspect::run(
                Command::new("rpmsign").arg("--addsign").arg("--define").arg(format!("_gpg_name {key}")).arg(path),
                "RPM signing requires rpmsign and an available GPG agent",
            )?;
            inspect::verify_rpm_signature(path)
        }
        _ => Err(Error::Invalid("unsupported signing package format".into())),
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updater_versions() {
        let text = "[installer]\nversion='1.2.4'\nformats=['nsis']\n[dokime]\nversion='1.2.3'\ntarget='x86_64-pc-windows-msvc'\n[signing.updater]\nprivate_key_env='FIXTURE_KEY'\npassword_env='FIXTURE_PASSWORD'\nartifact_base_url='https://example.com'";
        let manifest: Manifest = toml::from_str(text).unwrap();
        assert!(validate_signing(&manifest, false).is_ok());
        let error = validate_signing(&manifest, true).unwrap_err();
        assert!(error.to_string().contains("matching installer and Dokime versions"));
    }

    #[test]
    fn native_labels() {
        for label in ["../outside", "..", ".. ", "Company/Outside", "Company\\Outside", "Company\nOutside", "Company."] {
            assert!(validate_label(label).is_err());
        }
        assert!(validate_label("Stokker Technologies").is_ok());
    }
}
