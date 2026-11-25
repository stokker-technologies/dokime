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
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Component;
use std::process::Command;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

// Third Party Imports
use serde::Serialize;
use tauri_bundler::Settings;
use tauri_bundler::{BundleBinary, BundleSettings, NsisSettings, PackageSettings, PackageType, SettingsBuilder};
use tauri_utils::config::NSISInstallerMode;

// Local Imports
use super::{Error, PackageOptions};
use super::{
    artwork,
    distribution::{self, Distribution},
    license,
    manifest::{Format, InstallerSigning, Manifest, UpdaterSigning},
    pack::ResolvedPack,
    plugins::Staged,
};
use crate::installation;

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

// Installed Payload Collected While Staging: Windows Resources, Linux Files And Their Digests.
pub(crate) struct Staging {
    pub(crate) resources: HashMap<String, String>,
    pub(crate) linux_files: HashMap<PathBuf, PathBuf>,
    pub(crate) files: BTreeMap<String, String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Staging {
    fn add(&mut self, staged: &Path, prefix: &str, installed: &str) -> Result<(), Error> {
        self.resources.insert(staged.to_string_lossy().into_owned(), installed.into());
        self.linux_files.insert(PathBuf::from(format!("/{prefix}{installed}")), staged.to_owned());
        let bytes = fs::read(staged)?;
        self.files.insert(format!("{prefix}{installed}"), distribution::digest(&bytes));
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn prepare(
    manifest: &Manifest, distribution: &Distribution, plugins: &BTreeMap<String, Staged>, scratch: &Path, options: &PackageOptions,
    authenticode: bool, mut pack: ResolvedPack,
) -> Result<Settings, Error> {
    let sign = options.sign;
    let source = &distribution.source;
    let identity = manifest.identity(&distribution.identity)?;
    // Verified Plugin Bundles Have One Installed Location Outside The Organization Pack.
    pack.files.retain(|name, _| !name.starts_with("plugins/"));
    let Some((_, application_bytes)) = pack.asset("application") else {
        return Err(Error::Invalid("organization application artwork is required".into()));
    };
    let application_image = image::load_from_memory(application_bytes).map_err(|error| Error::Invalid(error.to_string()))?;
    let pack_dir = scratch.join("organization");
    pack.write_to(&pack_dir)?;
    let label = pack.label().to_owned();
    let windows = distribution.target.contains("windows");
    let suffix = if windows { ".exe" } else { "" };
    let installed_resource_prefix = if windows { String::new() } else { format!("usr/lib/{}/", identity.slug) };
    let mut linux_files = HashMap::new();
    let bin_dir = scratch.join("build");
    fs::create_dir_all(bin_dir.join("distribution"))?;
    let mut files = BTreeMap::new();
    for name in ["dokime", "dokime-gui"] {
        let file = format!("{name}{suffix}");
        let bytes = fs::read(source.join("bin").join(&file))?;
        let destination = if windows { bin_dir.join(&file) } else { bin_dir.join("distribution").join(&file) };
        let replace_icon = name == "dokime-gui" && manifest.installer.organization.is_some();
        super::stage_executable(&destination, &bytes, replace_icon.then_some(&application_image))?;
        if windows {
            if sign {
                sign_installer(manifest, &destination, options.command)?;
            }
            if authenticode {
                super::signing::authenticode(&destination, options.command)?;
            }
        }
        let installed = format!("{installed_resource_prefix}{file}");
        if !windows {
            linux_files.insert(PathBuf::from(format!("/{installed}")), destination.clone());
            let launcher = name;
            let script = format!("#!/bin/sh\nexec /usr/lib/{}/{file} \"$@\"\n", identity.slug);
            let path = bin_dir.join(launcher);
            super::stage_executable(&path, script.as_bytes(), None)?;
            let installed = format!("usr/bin/{}", path.file_name().unwrap().to_string_lossy());
            files.insert(installed, distribution::digest(script.as_bytes()));
        }
        let final_bytes = fs::read(&destination)?;
        let digest = distribution::digest(&final_bytes);
        files.insert(installed, digest);
    }
    let mut resources = HashMap::new();
    for (name, plugin) in plugins {
        let staged = &plugin.path;
        let relative = format!("{}/{name}", installation::PLUGINS);
        resources.insert(staged.to_string_lossy().into_owned(), relative.clone());
        let linux_path = PathBuf::from(format!("/{installed_resource_prefix}{relative}"));
        linux_files.insert(linux_path, staged.clone());
        let bytes = fs::read(staged)?;
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
        let bytes = fs::read(staged)?;
        let digest = distribution::digest(&bytes);
        let installed = format!("{installed_resource_prefix}{relative}");
        files.insert(installed, digest);
    }
    let mut staging = Staging { resources, linux_files, files };
    let (license, license_file) = stage_licenses(manifest, source, scratch, &installed_resource_prefix, &mut staging)?;
    if !windows {
        stage_copyright(manifest, &pack, source, scratch, &identity.slug, &mut staging)?;
    }
    let Staging {
        mut resources,
        mut linux_files,
        mut files,
    } = staging;

    // Stage Startup Identity in the Verified Inventory
    let installed_identity = distribution::Identity::new(identity.identifier.clone()).map_err(Error::Invalid)?;
    let identity_bytes = serde_json::to_vec_pretty(&serde_json::json!({"schema":1,"identifier":installed_identity.identifier}))?;
    let identity_path = scratch.join("distribution.json");
    fs::write(&identity_path, &identity_bytes)?;
    resources.insert(identity_path.to_string_lossy().into_owned(), "distribution.json".into());
    linux_files.insert(PathBuf::from(format!("/{installed_resource_prefix}distribution.json")), identity_path);
    files.insert(format!("{installed_resource_prefix}distribution.json"), distribution::digest(&identity_bytes));

    // Describe Distribution
    let (mut installer_signed, updater_signed) = match &manifest.signing {
        Some(settings) => (sign && settings.installer.is_some(), sign && settings.updater.is_some()),
        None => (false, false),
    };
    installer_signed |= windows && authenticode;
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
    fs::write(&metadata_path, metadata_bytes)?;
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
    if windows {
        artwork::sidebar(&pack, &sidebar)?;
    }
    fs::write(&hooks, include_str!("hooks.nsh"))?;
    let template = scratch.join("installer.nsi");
    fs::write(&template, include_str!("installer.nsi"))?;
    let png_icon = image_path.to_string_lossy().into_owned();
    let ico_icon = ico_path.to_string_lossy().into_owned();
    let icons = vec![png_icon, ico_icon];
    let root = env::current_dir()?;
    let icons =
        if windows { icons.into_iter().map(|path| resource_path(Path::new(&path), &root)).collect::<Result<Vec<_>, _>>()? } else { icons };
    let mut bundle = BundleSettings {
        identifier: Some(identity.identifier.clone()),
        icon: Some(icons),
        license: Some(license),
        license_file: Some(license_file),
        ..BundleSettings::default()
    };
    bundle.windows.nsis = Some(NsisSettings {
        template: Some(template),
        installer_hooks: Some(hooks),
        installer_icon: Some(ico_path),
        sidebar_image: windows.then_some(sidebar),
        install_mode: NSISInstallerMode::CurrentUser,
        ..NsisSettings::default()
    });
    if !windows {
        let template = scratch.join("application.desktop");
        let contents = desktop_entry(&identity, &label);
        fs::write(&template, contents)?;
        bundle.deb.desktop_template = Some(template.clone());
        bundle.rpm.desktop_template = Some(template);
    }
    if windows {
        let root = env::current_dir()?;
        let resources = resources
            .into_iter()
            .map(|(path, target)| resource_path(Path::new(&path), &root).map(|path| (path, target)))
            .collect::<Result<HashMap<_, _>, _>>()?;
        bundle.resources_map = Some(resources);
    } else {
        bundle.deb.files = linux_files.clone();
        bundle.rpm.files = linux_files;
        // Classic Names Resolve On Ubuntu 22.04 And Debian 12; Later Releases' t64 Packages Provide Them.
        bundle.deb.depends = Some(vec![
            "libwebkit2gtk-4.1-0".into(),
            "libgtk-3-0".into(),
            "libayatana-appindicator3-1".into(),
            "libasound2".into(),
            "libsoup-3.0-0".into(),
            "libjavascriptcoregtk-4.1-0".into(),
            "librsvg2-2".into(),
            "libssl3".into(),
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
    let package =
        package_settings(manifest, &identity, &label, pack.organization_name().unwrap_or(&label), windows, &distribution.identity);
    let application_name = "dokime-gui".into();
    let command_name = "dokime".into();
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

    Ok(settings)
}

pub fn assemble(manifest: &Manifest, distribution: &Distribution, settings: &Settings, output: &Path) -> Result<Vec<PathBuf>, Error> {
    let bundles = tauri_bundler::bundle_project(settings).map_err(|error| {
        let message = error.to_string();
        Error::Invalid(message)
    })?;
    let paths = bundles.into_iter().flat_map(|bundle| {
        let kind = bundle.package_type;
        bundle.bundle_paths.into_iter().map(move |source| (kind, source))
    });
    let mut outputs = Vec::new();
    for (kind, source) in paths {
        let destination = finish_package(manifest, distribution, kind, &source, output)?;
        outputs.push(destination);
    }
    if outputs.is_empty() {
        return Err(Error::Invalid("Install the Native Packaging Tools and Run Packaging Again: No Installer Was Produced".into()));
    }
    Ok(outputs)
}

pub fn validate_signing(manifest: &Manifest, requested: bool) -> Result<(), Error> {
    let provider = env::var("DOKIME_AUTHENTICODE_PROVIDER").ok();
    validate_signing_with_provider(manifest, requested, provider.as_deref())
}

fn validate_signing_with_provider(manifest: &Manifest, requested: bool, provider: Option<&str>) -> Result<(), Error> {
    let hook = super::signing::authenticode_enabled(provider)?;
    if hook && requested && manifest.signing.as_ref().is_some_and(|settings| settings.installer.is_some()) {
        return Err(Error::Invalid("select either manifest installer signing or the Authenticode hook".into()));
    }
    let rpm_variables = ["TAURI_SIGNING_RPM_KEY", "TAURI_SIGNING_RPM_KEY_PASSPHRASE"];
    let inherited_rpm_key = manifest.installer.formats.contains(&Format::Rpm)
        && rpm_variables.iter().any(|name| {
            let value = env::var_os(name);
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

/* === Private Functions === */

fn resource_path(path: &Path, root: &Path) -> Result<String, Error> {
    let relative = path.strip_prefix(root).map_err(|_| Error::Invalid("installer resource is outside its staging root".into()))?;
    if relative.as_os_str().is_empty() || relative.components().any(|component| !matches!(component, Component::Normal(_))) {
        return Err(Error::Invalid("installer resource has an invalid staging path".into()));
    }
    let relative = relative.to_str().ok_or_else(|| Error::Invalid("installer resource path is not UTF-8".into()))?;
    Ok(relative.to_owned())
}

// Stages Dokime's `LICENSE` And Any Pack Terms; Returns The Package License And License Page File.
pub(crate) fn stage_licenses(
    manifest: &Manifest, source: &Path, scratch: &Path, prefix: &str, staging: &mut Staging,
) -> Result<(String, PathBuf), Error> {
    // Dokime's Apache-2.0 License Is Always Installed Unchanged.
    let license = source.join("LICENSE");
    let staged = scratch.join("LICENSE");
    fs::copy(&license, &staged)?;
    let license = staged;
    for name in ["NOTICE", "THIRD-PARTY-NOTICES"] {
        let path = scratch.join(name);
        fs::copy(source.join(name), &path)?;
        staging.add(&path, prefix, name)?;
    }
    staging.add(&license, prefix, "LICENSE")?;
    let Some(terms) = &manifest.license else {
        return Ok((license::DOKIME.into(), license));
    };

    // Pack Terms Are Added Beside Dokime's License And Shown On The NSIS License Page.
    let (spdx, file) = terms.terms()?;
    let name = license::installed_name(file);
    // A Separate Folder Keeps The Terms Clear Of Case-Insensitive Scratch Names Such As LICENSE.
    let folder = scratch.join("terms");
    fs::create_dir_all(&folder)?;
    let staged = folder.join(&name);
    fs::copy(file, &staged)?;
    staging.add(&staged, prefix, &name)?;
    let expression = license::combined(Some(spdx));
    Ok((expression, staged))
}

// Stages The DEP-5 Debian Copyright File At `/usr/share/doc/<package>/copyright`.
//
// Tauri Names The Linux Package After The Product Name, Which Is The Distribution Slug.
pub(crate) fn stage_copyright(
    manifest: &Manifest, pack: &ResolvedPack, source: &Path, scratch: &Path, package: &str, staging: &mut Staging,
) -> Result<(), Error> {
    let apache = fs::read_to_string(source.join("LICENSE"))?;
    let terms = match &manifest.license {
        Some(terms) => Some(terms_text(terms)?),
        None => None,
    };
    let notice = fs::read_to_string(source.join("NOTICE"))?;
    let holder = notice.lines().find_map(|line| line.strip_prefix("Copyright ")).unwrap_or("Dokime Contributors");
    let mut holders = vec![holder];
    let organization = pack.organization_name().map(str::trim).filter(|name| !name.is_empty());
    if let Some(name) = organization.filter(|name| *name != holder) {
        holders.push(name);
    }

    // Shared With The RPM Payload And Hashed Like Every Other Installed File.
    let pack_terms = terms.as_ref().map(|(spdx, text)| (*spdx, text.as_str()));
    let text = license::debian_copyright(&holders, &apache, pack_terms);
    let staged = scratch.join("copyright");
    fs::write(&staged, &text)?;
    let installed = format!("usr/share/doc/{package}/copyright");
    staging.linux_files.insert(PathBuf::from(format!("/{installed}")), staged);
    staging.files.insert(installed, distribution::digest(text.as_bytes()));
    Ok(())
}

// Reads The Pack's Terms As UTF-8 Text For The Debian Copyright File.
fn terms_text(terms: &super::manifest::License) -> Result<(&str, String), Error> {
    let (spdx, file) = terms.terms()?;
    let bytes = fs::read(file)?;
    let text = String::from_utf8(bytes).map_err(|_| Error::License {
        field: "file",
        reason: "must be UTF-8 text on Linux, where it is copied into the Debian copyright file".into(),
    })?;
    Ok((spdx, text))
}

fn package_settings(
    manifest: &Manifest, identity: &distribution::Identity, label: &str, publisher: &str, windows: bool, compiled: &distribution::Identity,
) -> PackageSettings {
    // Tauri Uses The Linux Product Name As The Native Package Name.
    // The Description Supplies DEB's Display Description And RPM's Summary.
    PackageSettings {
        product_name: if windows {
            label.to_owned()
        } else if identity.identifier == compiled.identifier {
            "dokime".into()
        } else {
            identity.slug.clone()
        },
        version: manifest.installer.version.clone(),
        description: label.to_owned(),
        homepage: None,
        authors: Some(vec![publisher.into()]),
        default_run: Some("dokime-gui".into()),
    }
}

fn desktop_entry(identity: &distribution::Identity, label: &str) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={label}\nExec=/usr/lib/{}/dokime-gui\nIcon=dokime-gui\nCategories=Utility;\n",
        identity.slug
    )
}

pub(super) fn resolve_organization(manifest: &Manifest, source: &Path) -> Result<ResolvedPack, Error> {
    let base = source.join("organization");
    let overrides = manifest.installer.organization.as_deref();
    let mut pack = super::pack::read(overrides.unwrap_or(&base))?;
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
    let text = fs::read_to_string(&path)?;
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
    manifest: &Manifest, distribution: &Distribution, kind: PackageType, source: &Path, output: &Path,
) -> Result<PathBuf, Error> {
    let identity = manifest.identity(&distribution.identity)?;
    let slug = if identity.identifier == distribution.identity.identifier { "dokime" } else { &identity.slug };
    let name = match kind {
        PackageType::Nsis => format!("{slug}-{}-{}-setup.exe", manifest.installer.version, manifest.dokime.target),
        PackageType::Deb => format!("{slug}-{}-{}.deb", manifest.installer.version, manifest.dokime.target),
        PackageType::Rpm => format!("{slug}-{}-{}.rpm", manifest.installer.version, manifest.dokime.target),
        _ => {
            let error = Error::Invalid("unexpected native package type".into());
            return Err(error);
        }
    };
    let destination = output.join(name);
    fs::copy(source, &destination)?;
    Ok(destination)
}

pub(super) fn sign(
    manifest: &Manifest, distribution: &Distribution, destination: &Path, output: &Path, options: &PackageOptions,
) -> Result<(), Error> {
    if options.sign {
        sign_installer(manifest, destination, options.command)?;
    }
    if destination.extension().is_some_and(|extension| extension == "exe") {
        super::signing::authenticode(destination, options.command)?;
    }
    if options.sign {
        super::signing::updater(manifest, distribution, destination, output, options.command)?;
    }
    Ok(())
}

fn validate_installer_signing(settings: &InstallerSigning) -> Result<(), Error> {
    if cfg!(target_os = "windows") {
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
    if manifest.dokime.version() != super::release::NIGHTLY && manifest.installer.version != manifest.dokime.version() {
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

    let unavailable = !cfg!(target_os = "windows") || env::var(&settings.password_env).is_err();
    if unavailable {
        let error = Error::Invalid(
            "updater signing requires native Windows, a key file, external password environment and HTTPS artifact URL".into(),
        );
        return Err(error);
    }
    settings.base_url()?;
    Ok(())
}

fn environment_key_available(name: Option<&str>) -> bool {
    let Some(name) = name else {
        return false;
    };
    let Ok(value) = env::var(name) else {
        return false;
    };
    let trimmed = value.trim();
    !trimmed.is_empty()
}

fn environment_file_available(name: Option<&str>) -> bool {
    let Some(name) = name else {
        return false;
    };
    let Some(value) = env::var_os(name) else {
        return false;
    };
    let path = Path::new(&value);
    path.is_file()
}

fn sign_installer(manifest: &Manifest, path: &Path, tools: fn(&OsStr) -> Command) -> Result<(), Error> {
    let Some(signing) = &manifest.signing else {
        return Ok(());
    };
    let Some(settings) = &signing.installer else {
        return Ok(());
    };
    if cfg!(target_os = "windows") {
        return sign_windows_installer(settings, path, tools);
    }
    let Some(key) = settings.key_id.as_deref() else {
        let error = Error::Invalid("missing Linux signing key".into());
        return Err(error);
    };
    let extension = path.extension().and_then(OsStr::to_str);
    match extension {
        Some("deb") => {
            let mut command = tools(OsStr::new("debsigs"));
            command.args(["--sign=origin", "--default-key", key]).arg(path);
            super::signing::run(&mut command, "DEB signing requires debsigs and an available GPG agent")?;
            let mut command = tools(OsStr::new("debsig-verify"));
            command.arg(path);
            super::signing::run(&mut command, "DEB signature verification requires debsig-verify with the signing policy installed")
        }
        Some("rpm") => {
            let key_definition = format!("_gpg_name {key}");
            let mut command = tools(OsStr::new("rpmsign"));
            command.arg("--addsign").arg("--define").arg(key_definition).arg(path);
            super::signing::run(&mut command, "RPM signing requires rpmsign and an available GPG agent")?;
            super::signing::verify_rpm_signature(path, tools)
        }
        _ => {
            let error = Error::Invalid("unsupported signing package format".into());
            Err(error)
        }
    }
}

fn sign_windows_installer(settings: &InstallerSigning, path: &Path, tools: fn(&OsStr) -> Command) -> Result<(), Error> {
    let Some(thumbprint) = settings.certificate_thumbprint.as_deref() else {
        let error = Error::Invalid("missing signing certificate".into());
        return Err(error);
    };
    let Some(timestamp) = settings.timestamp_url.as_deref() else {
        let error = Error::Invalid("missing signing timestamp URL".into());
        return Err(error);
    };
    let mut command = tools(OsStr::new("signtool"));
    command.args(["sign", "/sha1", thumbprint, "/fd", "SHA256", "/tr", timestamp, "/td", "SHA256"]).arg(path);
    super::signing::run(&mut command, "installer signing requires signtool and the configured certificate in the current user store")?;
    let mut command = tools(OsStr::new("signtool"));
    command.args(["verify", "/pa"]).arg(path);
    super::signing::run(&mut command, "installer Authenticode verification failed")
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installer::{fixtures, scratch};
    use std::env::consts::EXE_SUFFIX;
    use std::fs;

    #[test]
    fn prepare_verified_kit_with_complete_replacement_and_default_appearance() -> Result<(), Error> {
        // Fabricate a Complete Kit Without Consulting a Published Release.
        let source = tempfile::tempdir()?;
        fixtures::kit(source.path())?;
        fs::write(source.path().join("organization/default-only.css"), b"default")?;
        let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(source.path().join("distribution.json"))?)?;
        metadata["files"]["organization/default-only.css"] = distribution::digest(b"default").into();
        fs::write(source.path().join("distribution.json"), serde_json::to_vec(&metadata)?)?;
        let mut manifest = fixtures::manifest();
        let kit = distribution::validate(source.path(), &manifest.dokime)?;
        let original = fs::read(source.path().join(format!("bin/dokime-gui{}", EXE_SUFFIX)))?;
        // Prepare the Built-In Appearance Through the Native Tool Boundary.
        let staging = scratch()?;
        let settings = prepare(
            &manifest,
            &kit,
            &BTreeMap::new(),
            staging.path(),
            &fixtures::options(),
            false,
            resolve_organization(&manifest, source.path())?,
        )?;
        assert_eq!(settings.bundle_identifier(), "com.example.testhub");
        assert_eq!(settings.windows().nsis.as_ref().unwrap().install_mode, NSISInstallerMode::CurrentUser);
        assert!(staging.path().join("organization/default-only.css").is_file());
        assert_eq!(fs::read(staging.path().join("NOTICE"))?, b"NOTICE");
        let installed: serde_json::Value = serde_json::from_slice(&fs::read(staging.path().join("distribution.json"))?)?;
        assert_eq!(installed, serde_json::json!({"schema":1,"identifier":"com.example.testhub"}));
        let payload: serde_json::Value = serde_json::from_slice(&fs::read(staging.path().join("installer.json"))?)?;
        assert!(
            payload["files"].get("distribution.json").is_some()
                || payload["files"].get("usr/lib/com-example-testhub/distribution.json").is_some()
        );
        // Supply a Complete Pack and Additional Terms; No Default Asset Survives.
        let organization = tempfile::tempdir()?;
        fixtures::organization(organization.path(), "Example Test Hub")?;
        manifest.installer.organization = Some(organization.path().to_owned());
        let terms = organization.path().join("terms.txt");
        fs::write(&terms, b"example terms")?;
        manifest.license = Some(crate::installer::manifest::License {
            spdx: Some("LicenseRef-Example".into()),
            file: Some(terms),
        });
        let staging = scratch()?;
        prepare(
            &manifest,
            &kit,
            &BTreeMap::new(),
            staging.path(),
            &fixtures::options(),
            false,
            resolve_organization(&manifest, source.path())?,
        )?;
        assert!(!staging.path().join("organization/default-only.css").exists());
        assert_eq!(fs::read(staging.path().join("terms/EULA.txt"))?, b"example terms");
        let appearance: toml::Value = toml::from_str(&fs::read_to_string(staging.path().join("organization/organization.toml"))?).unwrap();
        assert_eq!(appearance["updates"]["mode"].as_str(), Some("disabled"));
        assert_eq!(fs::read(source.path().join(format!("bin/dokime-gui{}", EXE_SUFFIX)))?, original);
        Ok(())
    }

    #[test]
    fn current_user_template_and_hooks_have_no_elevated_execution_path() {
        let source = include_str!("installer.nsi");
        let hooks = include_str!("hooks.nsh");
        let script = handlebars::Handlebars::new()
            .render_template(source, &serde_json::json!({"bundle_id":"com.example.testhub","product_name":"Example Test Hub"}))
            .unwrap();
        assert_eq!(
            script.lines().map(str::trim).filter(|line| line.starts_with("RequestExecutionLevel")).collect::<Vec<_>>(),
            ["RequestExecutionLevel user"]
        );
        for prohibited in [
            "RequestExecutionLevel admin",
            "SetShellVarContext all",
            "WriteRegStr HKLM",
            "WriteRegDWORD HKLM",
            "DeleteRegKey HKLM",
            "PROGRAMFILES",
            "MULTIUSER",
            "RunAs",
            "-Verb",
        ] {
            assert!(!script.contains(prohibited), "{prohibited}");
            assert!(!hooks.contains(prohibited), "{prohibited}");
        }
        assert!(hooks.contains("StrCpy $0 \"User\""));
        assert!(script.contains("InstallDir \"$LOCALAPPDATA\\${BUNDLEID}\""));
        assert!(script.contains("WriteRegStr HKCU"));
    }

    #[test]
    fn native_distribution_names_preserve_toolchain_executables() -> Result<(), Error> {
        let mut manifest = fixtures::manifest();
        for identifier in ["com.example.testhub", "com.example.otherhub"] {
            let identity = distribution::Identity::new(identifier.into()).unwrap();
            let compiled = distribution::Identity::compiled().unwrap();
            let package = package_settings(&manifest, &identity, "Example Test Hub", "Example Test Hub", false, &compiled);
            assert_eq!(package.product_name, identifier.replace('.', "-"));
            assert_eq!(package.default_run.as_deref(), Some("dokime-gui"));
            let desktop = desktop_entry(&identity, "Example Test Hub");
            assert!(desktop.contains(&format!("Exec=/usr/lib/{}/dokime-gui\n", identifier.replace('.', "-"))));
        }
        let source = tempfile::tempdir()?;
        fixtures::kit(source.path())?;
        let kit = distribution::validate(source.path(), &manifest.dokime)?;
        let input = source.path().join("payload");
        fs::write(&input, b"installer fixture")?;
        let output = tempfile::tempdir()?;
        for (identifier, prefix) in [
            (None, "dokime"),
            (Some(kit.identity.identifier.clone()), "dokime"),
            (Some("com.example.testhub".into()), "com-example-testhub"),
        ] {
            manifest.installer.identifier = identifier;
            let identity = manifest.identity(&kit.identity)?;
            let package = package_settings(&manifest, &identity, "Example Test Hub", "Example Test Hub", false, &kit.identity);
            assert_eq!(package.product_name, prefix);
            for (kind, ending) in [(PackageType::Nsis, "-setup.exe"), (PackageType::Deb, ".deb"), (PackageType::Rpm, ".rpm")] {
                let path = finish_package(&manifest, &kit, kind, &input, output.path())?;
                let expected = format!("{prefix}-1.2.3-{}{ending}", manifest.dokime.target);
                assert_eq!(path.file_name().unwrap().to_str(), Some(expected.as_str()));
                assert_eq!(fs::read(path)?, b"installer fixture");
            }
        }
        Ok(())
    }

    #[test]
    fn signing_refusals_preserve_explicit_credential_selection() {
        let mut manifest = fixtures::manifest();
        assert!(validate_signing_with_provider(&manifest, false, None).is_ok());
        assert!(matches!(validate_signing_with_provider(&manifest, true, None), Err(Error::SigningConfiguration)));
        manifest.signing = Some(crate::installer::manifest::Signing { installer: None, updater: None });
        assert!(validate_signing_with_provider(&manifest, true, None).is_err());
        let mut settings = InstallerSigning {
            certificate_thumbprint: Some("a".repeat(40)),
            timestamp_url: Some("https://timestamp.example.test".into()),
            key_id: None,
        };
        assert!(validate_windows_signing(&settings).is_ok());
        settings.key_id = Some("abc".into());
        assert!(validate_windows_signing(&settings).is_err());
        settings.key_id = None;
        settings.timestamp_url = Some("http://timestamp.example.test".into());
        assert!(validate_windows_signing(&settings).is_err());
        settings.timestamp_url = Some("https://timestamp.example.test".into());
        settings.certificate_thumbprint = Some("invalid".into());
        assert!(validate_windows_signing(&settings).is_err());
    }
}
