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
};

// Third Party Imports
use serde::Serialize;
use tauri_bundler::{BundleBinary, BundleSettings, NsisSettings, PackageSettings, PackageType, SettingsBuilder};

// Local Imports
use super::{
    artwork,
    distribution::{self, Distribution},
    manifest::{Format, InstallerSigning, Manifest, UpdaterSigning},
};
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Serialize)]
struct DistributionMetadata {
    schema: u32,
    version: String,
    dokime_version: String,
    target: String,
    label: String,
    identity: distribution::Identity,
    files: BTreeMap<String, String>,
    installer_signed: bool,
    updater_signed: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn assemble(
    manifest: &Manifest, distribution: &Distribution, source: &Path, plugins: &BTreeMap<String, PathBuf>, scratch: &Path, output: &Path,
    sign: bool,
) -> Result<Vec<PathBuf>, Error> {
    let terminal = dokime::terminal::Terminal::default();
    terminal.status(format!("Staging distribution with {} plugins", plugins.len()));
    let identity = manifest.identity(&distribution.identity)?;
    let mut pack = resolve_organization(manifest, source)?;
    // Verified Plugin Bundles Have One Installed Location Outside The Organization Pack.
    pack.files.retain(|name, _| !name.starts_with("plugins/"));
    let Some((_, application_bytes)) = pack.asset("application") else {
        return Err(Error::Invalid("organization application artwork is required".into()));
    };
    let application_image = image::load_from_memory(application_bytes).map_err(|error| Error::Invalid(error.to_string()))?;
    #[cfg(windows)]
    let application_icon = artwork::icon(&application_image)?;
    let pack_dir = scratch.join("organization");
    pack.write_to(&pack_dir).map_err(Error::Invalid)?;
    let label = pack.label().to_owned();
    validate_label(&label)?;
    let windows = distribution.target.contains("windows");
    let suffix = if windows { ".exe" } else { "" };
    let installed_resource_prefix = if windows { String::new() } else { format!("usr/lib/{}/", identity.slug) };
    let mut linux_files = HashMap::new();
    let bin_dir = scratch.join("build");
    std::fs::create_dir_all(bin_dir.join("distribution"))?;
    let mut files = BTreeMap::new();
    for name in ["dokime", "dokime-gui"] {
        let file = format!("{name}{suffix}");
        let bytes = std::fs::read(source.join("bin").join(&file))?;
        let destination = if windows { bin_dir.join(&file) } else { bin_dir.join("distribution").join(&file) };
        std::fs::write(&destination, &bytes)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o755))?;
        }
        #[cfg(windows)]
        if windows && name == "dokime-gui" {
            super::windows::replace(&destination, &application_icon)?;
            if sign {
                sign_installer(manifest, &destination)?;
            }
        }
        let installed = format!("{installed_resource_prefix}{file}");
        if !windows {
            linux_files.insert(PathBuf::from(format!("/{installed}")), destination.clone());
            let launcher = if name == "dokime" { identity.slug.clone() } else { format!("{}-gui", identity.slug) };
            let script = format!("#!/bin/sh\nexec /usr/lib/{}/{file} \"$@\"\n", identity.slug);
            let path = bin_dir.join(launcher);
            std::fs::write(&path, &script)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
            }
            let installed = format!("usr/bin/{}", path.file_name().unwrap().to_string_lossy());
            files.insert(installed, distribution::digest(script.as_bytes()));
        }
        let final_bytes = std::fs::read(&destination)?;
        let digest = distribution::digest(&final_bytes);
        files.insert(installed, digest);
    }
    let mut resources = HashMap::new();
    for (name, staged) in plugins {
        let relative = format!("plugins/{name}");
        resources.insert(staged.to_string_lossy().into_owned(), relative.clone());
        let linux_path = PathBuf::from(format!("/{installed_resource_prefix}{relative}"));
        linux_files.insert(linux_path, staged.clone());
        let bytes = std::fs::read(staged)?;
        let installed = format!("{installed_resource_prefix}{relative}");
        files.insert(installed, distribution::digest(&bytes));
    }
    for name in pack.files.keys() {
        let relative = format!("organization/{name}");
        let pack_path = distribution::relative(name)?;
        let staged = pack_dir.join(pack_path);
        let resource = staged.to_string_lossy().into_owned();
        resources.insert(resource, relative.clone());
        let installed = format!("/{installed_resource_prefix}{relative}");
        linux_files.insert(PathBuf::from(installed), staged.clone());
        let bytes = std::fs::read(staged)?;
        let digest = distribution::digest(&bytes);
        let installed = format!("{installed_resource_prefix}{relative}");
        files.insert(installed, digest);
    }
    let license = source.join("LICENSE");
    #[cfg(windows)]
    let license = {
        let staged = scratch.join("LICENSE");
        std::fs::copy(&license, &staged)?;
        staged
    };
    resources.insert(license.to_string_lossy().into_owned(), "LICENSE".into());
    linux_files.insert(PathBuf::from(format!("/{installed_resource_prefix}LICENSE")), license.clone());
    let license_bytes = std::fs::read(&license)?;
    let license_digest = distribution::digest(&license_bytes);
    let installed_license = format!("{installed_resource_prefix}LICENSE");
    files.insert(installed_license, license_digest);

    // Stage Startup Identity In The Verified Inventory
    let installed_identity =
        dokime::paths::DistributionIdentity::new(identity.identifier.clone()).map_err(|error| Error::Invalid(error.to_string()))?;
    let identity_bytes = serde_json::to_vec_pretty(&installed_identity)?;
    let identity_path = scratch.join("distribution.json");
    std::fs::write(&identity_path, &identity_bytes)?;
    resources.insert(identity_path.to_string_lossy().into_owned(), "distribution.json".into());
    linux_files.insert(PathBuf::from(format!("/{installed_resource_prefix}distribution.json")), identity_path);
    files.insert(format!("{installed_resource_prefix}distribution.json"), distribution::digest(&identity_bytes));

    // Describe Distribution
    let (installer_signed, updater_signed) = match &manifest.signing {
        Some(settings) => (sign && settings.installer.is_some(), sign && settings.updater.is_some()),
        None => (false, false),
    };
    let metadata = DistributionMetadata {
        schema: 1,
        version: manifest.installer.version.clone(),
        dokime_version: distribution.version.clone(),
        target: distribution.target.clone(),
        label: label.clone(),
        identity: identity.clone(),
        files,
        installer_signed,
        updater_signed,
    };
    let metadata_path = scratch.join("installer.json");
    let metadata_bytes = serde_json::to_vec_pretty(&metadata)?;
    std::fs::write(&metadata_path, metadata_bytes)?;
    resources.insert(metadata_path.to_string_lossy().into_owned(), "installer.json".into());
    linux_files.insert(PathBuf::from(format!("/{installed_resource_prefix}installer.json")), metadata_path);

    let image_path = scratch.join("icon.png");
    application_image.save_with_format(&image_path, image::ImageFormat::Png).map_err(|error| Error::Invalid(error.to_string()))?;
    let ico_path = scratch.join("icon.ico");
    let Some((_, installer_bytes)) = pack.asset("installer") else {
        let error = Error::Invalid("organization installer artwork is required".into());
        return Err(error);
    };
    let installer_image = image::load_from_memory(installer_bytes).map_err(|error| {
        let message = error.to_string();
        Error::Invalid(message)
    })?;
    artwork::write_ico(&installer_image, &ico_path)?;
    let hooks = scratch.join("installer-hooks.nsh");
    let sidebar = scratch.join("installer-sidebar.bmp");
    let hook_source = if windows {
        artwork::sidebar(&pack, &sidebar)?;
        let sidebar_path = sidebar.to_string_lossy().replace('$', "$$");
        format!("!define MUI_UNWELCOMEFINISHPAGE_BITMAP \"{sidebar_path}\"\n{}", include_str!("hooks.nsh"))
    } else {
        include_str!("hooks.nsh").to_owned()
    };
    std::fs::write(&hooks, hook_source)?;
    let template = scratch.join("installer.nsi");
    std::fs::write(&template, include_str!("installer.nsi"))?;
    let png_icon = image_path.to_string_lossy().into_owned();
    let ico_icon = ico_path.to_string_lossy().into_owned();
    let icons = vec![png_icon, ico_icon];
    #[cfg(windows)]
    let icons = {
        let root = std::env::current_dir()?;
        icons.into_iter().map(|path| resource_path(Path::new(&path), &root)).collect::<Result<Vec<_>, _>>()?
    };
    let mut bundle = BundleSettings {
        identifier: Some(identity.identifier.clone()),
        icon: Some(icons),
        license: Some("Apache-2.0".into()),
        license_file: Some(license),
        ..BundleSettings::default()
    };
    bundle.windows.nsis = Some(NsisSettings {
        template: Some(template),
        installer_hooks: Some(hooks),
        installer_icon: Some(ico_path),
        sidebar_image: windows.then_some(sidebar),
        install_mode: Default::default(), // CurrentUser; Keep Setup And Uninstall Unelevated.
        ..NsisSettings::default()
    });
    if !windows {
        let template = scratch.join("application.desktop");
        let contents = desktop_entry(&identity, &label);
        std::fs::write(&template, contents)?;
        bundle.deb.desktop_template = Some(template.clone());
        bundle.rpm.desktop_template = Some(template);
    }
    if windows {
        #[cfg(windows)]
        let resources = {
            let root = std::env::current_dir()?;
            resources
                .into_iter()
                .map(|(path, target)| resource_path(Path::new(&path), &root).map(|path| (path, target)))
                .collect::<Result<HashMap<_, _>, _>>()?
        };
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
    let package = package_settings(manifest, &identity, &label, windows);
    let application_name = if windows { "dokime-gui".into() } else { format!("{}-gui", identity.slug) };
    let command_name = if windows { "dokime".into() } else { identity.slug.clone() };
    let application = BundleBinary::new(application_name, true);
    let command = BundleBinary::new(command_name, false);
    let binaries = vec![application, command];
    let settings = SettingsBuilder::new()
        .project_out_directory(&bin_dir)
        .target(distribution.target.clone())
        .package_settings(package)
        .bundle_settings(bundle)
        .binaries(binaries)
        .package_types(types)
        .binary_patching(false)
        .no_sign(true)
        .build()
        .map_err(|error| {
            let message = error.to_string();
            Error::Invalid(message)
        })?;

    terminal.status(format!("Building native installer for {}", distribution.target));
    let bundles = tauri_bundler::bundle_project(&settings).map_err(|error| {
        let message = error.to_string();
        Error::Invalid(message)
    })?;
    let paths = bundles.into_iter().flat_map(|bundle| {
        let kind = bundle.package_type;
        bundle.bundle_paths.into_iter().map(move |source| (kind, source))
    });
    let mut outputs = Vec::new();
    for (kind, source) in paths {
        let destination = finish_package(manifest, distribution, kind, &source, output, sign)?;
        outputs.push(destination);
    }
    Ok(outputs)
}

#[cfg(any(windows, test))]
fn resource_path(path: &Path, root: &Path) -> Result<String, Error> {
    let relative = path.strip_prefix(root).map_err(|_| Error::Invalid("installer resource is outside its staging root".into()))?;
    if relative.as_os_str().is_empty() || relative.components().any(|component| !matches!(component, std::path::Component::Normal(_))) {
        return Err(Error::Invalid("installer resource has an invalid staging path".into()));
    }
    let relative = relative.to_str().ok_or_else(|| Error::Invalid("installer resource path is not UTF-8".into()))?;
    Ok(relative.to_owned())
}

pub fn validate_signing(manifest: &Manifest, requested: bool) -> Result<(), Error> {
    let rpm_variables = ["TAURI_SIGNING_RPM_KEY", "TAURI_SIGNING_RPM_KEY_PASSPHRASE"];
    let inherited_rpm_key = manifest.installer.formats.contains(&Format::Rpm)
        && rpm_variables.iter().any(|name| {
            let value = std::env::var_os(name);
            value.is_some()
        });
    if inherited_rpm_key {
        let error = Error::Invalid("unset inherited TAURI_SIGNING_RPM_KEY and TAURI_SIGNING_RPM_KEY_PASSPHRASE; RPM signing uses explicit --sign and signing.installer.key_id".into());
        return Err(error);
    }
    if !requested {
        return Ok(());
    }

    let Some(signing) = &manifest.signing else {
        let error = Error::SigningConfiguration;
        return Err(error);
    };
    if signing.installer.is_none() && signing.updater.is_none() {
        let error = Error::Invalid("--sign requires installer or updater signing settings".into());
        return Err(error);
    }
    if let Some(installer) = &signing.installer {
        validate_installer_signing(installer)?;
    }
    if let Some(updater) = &signing.updater {
        validate_updater_signing(manifest, updater)?;
    }
    Ok(())
}

pub(crate) fn validate_label(label: &str) -> Result<(), Error> {
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

/* === Private Functions === */

fn package_settings(manifest: &Manifest, identity: &distribution::Identity, label: &str, windows: bool) -> PackageSettings {
    // Tauri Uses The Linux Product Name As The Native Package Name.
    // The Description Supplies DEB's Display Description And RPM's Summary.
    PackageSettings {
        product_name: if windows { label.to_owned() } else { identity.slug.clone() },
        version: manifest.installer.version.clone(),
        description: label.to_owned(),
        homepage: None,
        authors: Some(vec!["Stokker Technologies".into()]),
        default_run: Some(if windows { "dokime-gui".into() } else { format!("{}-gui", identity.slug) }),
    }
}

fn desktop_entry(identity: &distribution::Identity, label: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={label}\nExec={}-gui\nIcon={}-gui\nCategories=Utility;\n",
        identity.slug, identity.slug
    )
}

fn resolve_organization(manifest: &Manifest, source: &Path) -> Result<dokime::config::organization::ResolvedPack, Error> {
    let base = source.join("organization");
    let overrides = manifest.installer.organization.as_deref();
    let resolved = dokime::config::organization::resolve_with_base(Some(&base), overrides);
    let mut pack = resolved.map_err(Error::Invalid)?;
    let Some(folder) = overrides else {
        return Ok(pack);
    };
    if explicit_update_mode(folder)? {
        return Ok(pack);
    }

    let Some(root) = pack.manifest.as_table_mut() else {
        let error = Error::Invalid("organization manifest must be a table".into());
        return Err(error);
    };
    let updates = root.entry("updates").or_insert_with(|| {
        let table = toml::map::Map::new();
        toml::Value::Table(table)
    });
    let Some(updates) = updates.as_table_mut() else {
        let error = Error::Invalid("organization updates must be a table".into());
        return Err(error);
    };
    let disabled = toml::Value::String("disabled".into());
    updates.insert("mode".into(), disabled);
    let serialized = toml::to_string(&pack.manifest).map_err(|error| {
        let message = error.to_string();
        Error::Invalid(message)
    })?;
    let bytes = serialized.into_bytes();
    pack.files.insert("organization.toml".into(), bytes);
    Ok(pack)
}

fn explicit_update_mode(folder: &Path) -> Result<bool, Error> {
    let path = folder.join("organization.toml");
    if !path.exists() {
        return Ok(false);
    }
    let text = std::fs::read_to_string(&path)?;
    let value: toml::Value = toml::from_str(&text).map_err(|error| {
        let message = format!("invalid organization manifest {}: {error}", path.display());
        Error::Invalid(message)
    })?;
    let Some(updates) = value.get("updates") else {
        return Ok(false);
    };
    let explicit = updates.get("mode").is_some();
    Ok(explicit)
}

fn finish_package(
    manifest: &Manifest, distribution: &Distribution, kind: PackageType, source: &Path, output: &Path, sign: bool,
) -> Result<PathBuf, Error> {
    let identity = manifest.identity(&distribution.identity)?;
    let slug = &identity.slug;
    let name = match kind {
        PackageType::Nsis => format!("{slug}-{}-{}-setup.exe", manifest.installer.version, manifest.dokime.target),
        PackageType::Deb => format!("{slug}_{}_amd64.deb", manifest.installer.version),
        PackageType::Rpm => format!("{slug}-{}-1.x86_64.rpm", manifest.installer.version),
        _ => {
            let error = Error::Invalid("unexpected native package type".into());
            return Err(error);
        }
    };
    if sign {
        sign_installer(manifest, source)?;
    }
    let destination = output.join(name);
    std::fs::copy(source, &destination)?;
    if sign {
        super::signing::updater(manifest, distribution, &destination, output)?;
    }
    Ok(destination)
}

fn validate_installer_signing(settings: &InstallerSigning) -> Result<(), Error> {
    if cfg!(windows) {
        return validate_windows_signing(settings);
    }
    let valid_key = match settings.key_id.as_deref() {
        Some(value) => !value.is_empty() && value.as_bytes().iter().all(u8::is_ascii_hexdigit),
        None => false,
    };
    let windows_settings = settings.certificate_thumbprint.is_some() || settings.timestamp_url.is_some();
    if !valid_key || windows_settings {
        let error =
            Error::Invalid("Linux installer signing requires a GPG key_id; credentials are supplied by the external GPG agent".into());
        return Err(error);
    }
    Ok(())
}

fn validate_windows_signing(settings: &InstallerSigning) -> Result<(), Error> {
    let valid_thumbprint = match settings.certificate_thumbprint.as_deref() {
        Some(value) => value.len() == 40 && value.as_bytes().iter().all(u8::is_ascii_hexdigit),
        None => false,
    };
    let valid_timestamp = match settings.timestamp_url.as_deref() {
        Some(value) => value.starts_with("https://"),
        None => false,
    };
    if !valid_thumbprint || !valid_timestamp || settings.key_id.is_some() {
        let error =
            Error::Invalid("Windows installer signing requires a certificate-store SHA-1 thumbprint and HTTPS timestamp URL".into());
        return Err(error);
    }
    Ok(())
}

fn validate_updater_signing(manifest: &Manifest, settings: &UpdaterSigning) -> Result<(), Error> {
    if manifest.installer.version != manifest.dokime.version() {
        let error = Error::Invalid(
            "updater signing requires matching installer and Dokime versions; distinct company versions require manual deployment".into(),
        );
        return Err(error);
    }
    let references = usize::from(settings.private_key_file.is_some())
        + usize::from(settings.private_key_env.is_some())
        + usize::from(settings.private_key_path_env.is_some());
    let key_available = settings.private_key_file.as_deref().is_some_and(Path::is_file)
        || environment_key_available(settings.private_key_env.as_deref())
        || environment_file_available(settings.private_key_path_env.as_deref());
    if references != 1 || !key_available {
        let error = Error::Invalid("updater signing requires exactly one available private key file/environment reference".into());
        return Err(error);
    }

    let unavailable =
        !cfg!(windows) || std::env::var(&settings.password_env).is_err() || !settings.artifact_base_url.starts_with("https://");
    if unavailable {
        let error = Error::Invalid(
            "updater signing requires native Windows, a key file, external password environment and HTTPS artifact URL".into(),
        );
        return Err(error);
    }
    Ok(())
}

fn environment_key_available(name: Option<&str>) -> bool {
    let Some(name) = name else {
        return false;
    };
    let Ok(value) = std::env::var(name) else {
        return false;
    };
    let trimmed = value.trim();
    !trimmed.is_empty()
}

fn environment_file_available(name: Option<&str>) -> bool {
    let Some(name) = name else {
        return false;
    };
    let Some(value) = std::env::var_os(name) else {
        return false;
    };
    let path = Path::new(&value);
    path.is_file()
}

fn sign_installer(manifest: &Manifest, path: &Path) -> Result<(), Error> {
    let Some(signing) = &manifest.signing else {
        return Ok(());
    };
    let Some(settings) = &signing.installer else {
        return Ok(());
    };
    if cfg!(windows) {
        return sign_windows_installer(settings, path);
    }
    let Some(key) = settings.key_id.as_deref() else {
        let error = Error::Invalid("missing Linux signing key".into());
        return Err(error);
    };
    let extension = path.extension().and_then(std::ffi::OsStr::to_str);
    match extension {
        Some("deb") => {
            let mut command = dokime::domain::process::command("debsigs");
            command.args(["--sign=origin", "--default-key", key]).arg(path);
            super::signing::run(&mut command, "DEB signing requires debsigs and an available GPG agent")?;
            let mut command = dokime::domain::process::command("debsig-verify");
            command.arg(path);
            super::signing::run(&mut command, "DEB signature verification requires debsig-verify with the signing policy installed")
        }
        Some("rpm") => {
            let key_definition = format!("_gpg_name {key}");
            let mut command = dokime::domain::process::command("rpmsign");
            command.arg("--addsign").arg("--define").arg(key_definition).arg(path);
            super::signing::run(&mut command, "RPM signing requires rpmsign and an available GPG agent")?;
            super::signing::verify_rpm_signature(path)
        }
        _ => {
            let error = Error::Invalid("unsupported signing package format".into());
            Err(error)
        }
    }
}

fn sign_windows_installer(settings: &InstallerSigning, path: &Path) -> Result<(), Error> {
    let Some(thumbprint) = settings.certificate_thumbprint.as_deref() else {
        let error = Error::Invalid("missing signing certificate".into());
        return Err(error);
    };
    let Some(timestamp) = settings.timestamp_url.as_deref() else {
        let error = Error::Invalid("missing signing timestamp URL".into());
        return Err(error);
    };
    let mut command = dokime::domain::process::command("signtool");
    command.args(["sign", "/sha1", thumbprint, "/fd", "SHA256", "/tr", timestamp, "/td", "SHA256"]).arg(path);
    super::signing::run(&mut command, "installer signing requires signtool and the configured certificate in the current user store")?;
    let mut command = dokime::domain::process::command("signtool");
    command.args(["verify", "/pa"]).arg(path);
    super::signing::run(&mut command, "installer Authenticode verification failed")
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_setup_keeps_firewall_and_elevation_out_of_hooks() {
        let nsis = NsisSettings::default();
        let mode = serde_json::to_value(nsis.install_mode).unwrap();
        let renderer = handlebars::Handlebars::new();
        for manifest in [
            "[installer]\nversion='1.2.3'\nformats=['nsis']\n[dokime]\ntarget='x86_64-pc-windows-msvc'",
            "[installer]\nversion='1.2.3'\nidentifier='com.example.test'\nformats=['nsis']\n[dokime]\ntarget='x86_64-pc-windows-msvc'",
        ] {
            let manifest: Manifest = toml::from_str(manifest).unwrap();
            let data = serde_json::json!({ "install_mode": mode, "version": manifest.installer.version });
            let script = renderer.render_template(include_str!("installer.nsi"), &data).unwrap();
            let hooks = include_str!("hooks.nsh");
            let definition = format!("!define INSTALLMODE \"{}\"", mode.as_str().unwrap());
            assert!(script.contains(&definition));
            let execution: Vec<_> = script.lines().map(str::trim).filter(|line| line.starts_with("RequestExecutionLevel")).collect();
            assert_eq!(execution, ["RequestExecutionLevel user"]);
            for forbidden in [
                "FIREWALL",
                "firewall.ps1",
                "New-NetFirewallRule",
                "-Verb RunAs",
                "UAC::",
                "RequestExecutionLevel admin",
                "MULTIUSER_EXECUTIONLEVEL Highest",
            ] {
                assert!(!script.contains(forbidden), "script contains {forbidden}");
                assert!(!hooks.contains(forbidden), "hooks contain {forbidden}");
            }
            assert!(hooks.contains("GetEnvironmentVariable('Path', 'User')"));
        }
    }

    #[test]
    fn distribution_names() {
        let label = "Example Test Hub";
        let compiled = distribution::Identity {
            identifier: "com.stokker.dokime".into(),
            slug: "dokime".into(),
            updater_public_key: "fixture".into(),
        };
        let text = "[installer]\nversion='1.2.3'\nformats=['nsis']\n[dokime]\nversion='1.2.3'\ntarget='x86_64-pc-windows-msvc'";
        let mut manifest: Manifest = toml::from_str(text).unwrap();
        let renderer = handlebars::Handlebars::new();

        for identifier in [None, Some("com.example.testhub")] {
            manifest.installer.identifier = identifier.map(str::to_owned);
            let identity = manifest.identity(&compiled).unwrap();
            let windows = package_settings(&manifest, &identity, label, true);
            assert_eq!(windows.product_name, label);
            let data =
                serde_json::json!({ "product_name": windows.product_name, "bundle_id": identity.identifier, "install_mode": "both" });
            let template = renderer.render_template(include_str!("installer.nsi"), &data).unwrap();
            assert!(template.contains(&format!("!define PRODUCTNAME \"{label}\"")));
            assert!(template.contains(&format!("!define BUNDLEID \"{}\"", identity.identifier)));

            // Expand The Identity Defines To Verify Their Native Uses.
            let script = template.replace("${BUNDLEID}", &identity.identifier).replace("${PRODUCTNAME}", label);
            assert!(script.contains(&format!("Name \"{label}\"")));
            assert!(script.contains(&format!("\"DisplayName\" \"{label}\"")));
            assert!(script.contains(&format!("$DESKTOP\\{label}.lnk")));
            assert!(script.contains(&format!("$SMPROGRAMS\\{label}.lnk")));
            assert!(script.contains(&format!("placeholder\\{}", identity.identifier)));
            assert!(script.contains(&format!("$LOCALAPPDATA\\{}", identity.identifier)));
            assert!(script.contains(&format!("$PROGRAMFILES\\{}", identity.identifier)));
            assert!(script.contains(&format!("$PROGRAMFILES64\\{}", identity.identifier)));
            assert!(script.contains(&format!("MULTIUSER_INSTALLMODE_INSTDIR \"{}\"", identity.identifier)));
            assert!(script.contains(&format!("CurrentVersion\\Uninstall\\{}", identity.identifier)));
            assert!(script.contains(&format!("${{MANUKEY}}\\{}", identity.identifier)));
            assert!(!script.contains(&format!("$LOCALAPPDATA\\{label}")));
            assert!(!script.contains(&format!("$PROGRAMFILES\\{label}")));
            assert!(!script.contains(&format!("$PROGRAMFILES64\\{label}")));

            let linux = package_settings(&manifest, &identity, label, false);
            assert_eq!(linux.product_name, identity.slug);
            assert_eq!(linux.description, label);
            let desktop = desktop_entry(&identity, label);
            assert!(desktop.contains(&format!("\nName={label}\n")));
            assert!(desktop.contains(&format!("\nExec={}-gui\n", identity.slug)));
        }
    }

    #[cfg(windows)]
    #[test]
    fn resource_drives() {
        let root = Path::new(r"D:\checkout");
        for resource in ["organization/fonts/font.woff2", "icon.png", "icon.ico", "LICENSE", "distribution.json"] {
            let staged = root.join("dokime-installer-fixture").join(resource);
            let mapped = resource_path(&staged, root).unwrap();
            assert_eq!(Path::new(&mapped), Path::new("dokime-installer-fixture").join(resource));
            assert!(Path::new(&mapped).is_relative());
            assert_eq!(root.join(mapped), staged);
            let other_drive = Path::new(r"C:\Temp").join(resource);
            assert!(resource_path(&other_drive, root).is_err());
        }
        assert!(resource_path(&root.join("../outside"), root).is_err());
    }

    #[test]
    fn resource_boundaries() {
        let root = Path::new("checkout");
        assert_eq!(Path::new(&resource_path(&root.join("staging/LICENSE"), root).unwrap()), Path::new("staging/LICENSE"));
        assert!(resource_path(root, root).is_err());
        assert!(resource_path(Path::new("outside/LICENSE"), root).is_err());
        assert!(resource_path(&root.join("staging/../outside"), root).is_err());
    }

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
