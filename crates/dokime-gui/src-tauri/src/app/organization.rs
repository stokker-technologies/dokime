/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Runtime Organization Packs: Validation, Staging And Boot Promotion

/* === Modules === */

pub(crate) mod policy;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Component, Path, PathBuf},
    sync::Mutex,
};

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

// Local Imports

/* === Constants === */

const DESKTOP_ORGANIZATION_ARCHIVE_BYTES: u64 = 20_971_520;
const DESKTOP_ORGANIZATION_ENTRIES: u64 = 200;

const MANIFEST: &str = "organization.toml";
const ACTIVE: &str = "active";
const PENDING: &str = "pending";
/// Applied by the shell at runtime: identity, logos, theme CSS and fonts.
const RUNTIME_EXTENSIONS: [&str; 5] = ["toml", "css", "woff2", "svg", "png"];
/// Licence and readme text beside fonts and logos; skipped, never applied.
const IGNORED_EXTENSIONS: [&str; 2] = ["txt", "md"];
const THEME_FILES: [&str; 4] = ["theme/typography.css", "theme/base.css", "theme/light.css", "theme/dark.css"];
const LOGO_VARIANTS: [(&str, &str, &str); 4] = [
    ("transparent", "on_light", "transparent_on_light"),
    ("transparent", "on_dark", "transparent_on_dark"),
    ("contained", "on_light", "contained_on_light"),
    ("contained", "on_dark", "contained_on_dark"),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// What the shell resolved at boot and what the operator changed since.
pub(crate) struct OrganizationState {
    root: PathBuf,
    boot: Boot,
    session: Mutex<Session>,
}

#[derive(Default)]
struct Session {
    pending: Option<PackSummary>,
    removed: bool,
}

struct Boot {
    pack: Option<LoadedPack>,
    error: Option<String>,
    active: bool,
}

/// The pack as the frontend applies it before it renders.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoadedPack {
    label: String,
    title_template: String,
    organization: Option<Organization>,
    /// Logo variant to `data:` URI.
    logos: BTreeMap<String, String>,
    css: String,
    pub(crate) updates: Option<policy::UpdatePolicy>,
    #[serde(skip)]
    application_icon: Option<Vec<u8>>,

    #[serde(skip)]
    tray_icon: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Organization {
    name: String,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    website: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PackSummary {
    label: String,
    organization: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OrganizationStatus {
    /// The pack this process applied at boot, unless removed since.
    active: Option<PackSummary>,
    /// A validated pack that applies on the next start.
    pending: Option<PackSummary>,
    /// Packs were removed this session; the compiled brand applies on the next start.
    removed: bool,
    /// Why the active pack could not be applied at boot.
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootScript<'a> {
    pack: Option<&'a LoadedPack>,
    error: Option<&'a str>,
}

#[derive(Deserialize)]
struct Manifest {
    app: AppSection,
    #[serde(default)]
    theme: Option<ThemeSection>,
    #[serde(default)]
    updates: Option<policy::UpdatePolicy>,
}

#[derive(Deserialize)]
struct AppSection {
    label: String,
    #[serde(default)]
    title_template: Option<String>,
    #[serde(default)]
    organization: Option<Organization>,
    #[serde(default)]
    logos: BTreeMap<String, BTreeMap<String, String>>,
}

#[derive(Deserialize)]
struct ThemeSection {
    #[serde(default)]
    corner_radii: BTreeMap<String, String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl OrganizationState {
    pub(crate) fn window_icon(&self) -> Option<tauri::image::Image<'static>> {
        let bytes = self.boot.pack.as_ref()?.application_icon.as_deref()?;
        tauri::image::Image::from_bytes(bytes).ok()
    }

    pub(crate) fn title(&self) -> Option<String> {
        let pack = self.boot.pack.as_ref()?;
        Some(pack.title_template.replace("{app}", &pack.label))
    }

    pub(crate) fn update_policy(&self) -> Result<policy::UpdatePolicy, String> {
        let updates = self.boot.pack.as_ref().and_then(|pack| pack.updates.clone()).unwrap_or_default();
        updates.validate()?;
        Ok(updates)
    }

    fn status(&self) -> OrganizationStatus {
        let session = self.session.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        OrganizationStatus {
            active: (!session.removed && self.boot.active).then(|| self.boot.pack.as_ref().map(LoadedPack::summary)).flatten(),
            pending: session.pending.clone(),
            removed: session.removed,
            error: self.boot.error.clone(),
        }
    }

    /// The script every main window runs before the frontend loads.
    pub(crate) fn initialization_script(&self) -> String {
        let boot = BootScript {
            pack: self.boot.pack.as_ref(),
            error: self.boot.error.as_deref(),
        };
        let json = serde_json::to_string(&boot).unwrap_or_else(|_| "{}".to_owned());
        format!("window.__DOKIME_ORGANIZATION_PACK__ = {json};")
    }
}

impl LoadedPack {
    fn summary(&self) -> PackSummary {
        PackSummary {
            label: self.label.clone(),
            organization: self.organization.as_ref().map(|organization| organization.name.clone()),
        }
    }
}

/* === Public Functions === */

/// Promotes a pending pack and loads the active one before any window exists.
pub(crate) fn setup(app: &tauri::App) -> tauri::Result<()> {
    let application = std::env::var_os("DOKIME_APPLICATION_ROOT").filter(|value| !value.is_empty());
    let root = match application {
        Some(root) => PathBuf::from(root),
        None => app.path().app_data_dir()?,
    }
    .join("organization");
    let boot = boot(&root);
    if let Some(error) = &boot.error {
        tracing::error!(%error, "organization pack could not be activated; retaining the last valid appearance");
    }
    let pending = load(&root.join(PENDING)).ok().map(|pack| pack.summary());
    if let Some(pack) = &boot.pack
        && let Some(bytes) = &pack.tray_icon
    {
        let icon = tauri::image::Image::from_bytes(bytes)?;
        let menu = tauri::menu::Menu::new(app)?;
        let tray = tauri::tray::TrayIconBuilder::with_id("organization").icon(icon).tooltip(&pack.label).menu(&menu).build(app)?;
        app.manage(tray);
    }
    app.manage(OrganizationState {
        root,
        boot,
        session: Mutex::new(Session { pending, removed: false }),
    });
    Ok(())
}

pub fn desktop_organization_status(state: tauri::State<'_, OrganizationState>) -> Result<OrganizationStatus, String> {
    Ok(state.status())
}

pub async fn desktop_organization_choose(app: AppHandle) -> Result<Option<OrganizationStatus>, String> {
    let picked = tauri::async_runtime::spawn_blocking({
        let app = app.clone();
        move || app.dialog().file().add_filter("Organization Pack", &["zip"]).blocking_pick_file()
    })
    .await
    .map_err(|error| error.to_string())?;
    let Some(file) = picked else { return Ok(None) };
    let path = file.into_path().map_err(|error| error.to_string())?;
    let root = app.state::<OrganizationState>().root.clone();
    let summary = tauri::async_runtime::spawn_blocking(move || stage(&root, &path)).await.map_err(|error| error.to_string())??;
    let state = app.state::<OrganizationState>();
    let mut session = state.session.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    session.pending = Some(summary);
    session.removed = false;
    drop(session);
    Ok(Some(state.status()))
}

pub async fn desktop_organization_remove(app: AppHandle) -> Result<OrganizationStatus, String> {
    let root = app.state::<OrganizationState>().root.clone();
    tauri::async_runtime::spawn_blocking(move || {
        for folder in [PENDING, ACTIVE] {
            remove_folder(&root.join(folder))?;
        }
        Ok::<_, String>(())
    })
    .await
    .map_err(|error| error.to_string())??;
    let state = app.state::<OrganizationState>();
    let mut session = state.session.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    session.pending = None;
    session.removed = true;
    drop(session);
    Ok(state.status())
}

/* === Private Functions === */

fn boot(root: &Path) -> Boot {
    let pending = root.join(PENDING);
    let active = root.join(ACTIVE);
    let mut promotion_error = None;
    if pending.is_dir() {
        // Validate Again At Boot Before Replacing The Last Valid Pack.
        let validated = load(&pending);
        let promoted = validated.and_then(|pack| {
            replace_pack(&pending, &active)?;
            Ok(pack)
        });
        match promoted {
            Ok(pack) => {
                return Boot {
                    pack: Some(pack),
                    error: None,
                    active: true,
                };
            }
            Err(error) => {
                promotion_error = Some(format!("the pending organization pack could not be activated: {error}"));
            }
        }
    }
    if !active.is_dir() {
        return base_boot(promotion_error);
    }
    match load(&active) {
        Ok(pack) => Boot {
            pack: Some(pack),
            error: promotion_error,
            active: true,
        },
        Err(error) => {
            let error = match promotion_error {
                Some(pending) => format!("{pending}; the active pack also failed: {error}"),
                None => error,
            };
            base_boot(Some(error))
        }
    }
}

fn base_boot(error: Option<String>) -> Boot {
    match dokime::config::organization::resolve(None).and_then(loaded) {
        Ok(pack) => Boot { pack: Some(pack), error, active: false },
        Err(base) => {
            let base = format!("the bundled organization could not be loaded: {base}");
            let error = error.map_or_else(|| base.clone(), |error| format!("{error}; {base}"));
            Boot {
                pack: None,
                error: Some(error),
                active: false,
            }
        }
    }
}

fn replace_pack(source: &Path, destination: &Path) -> Result<(), String> {
    if !destination.exists() {
        return std::fs::rename(source, destination).map_err(|error| error.to_string());
    }

    // Keep The Previous Pack On Disk Until Publication Succeeds.
    let root = destination.parent().ok_or("the pack destination has no parent directory")?;
    let mut builder = tempfile::Builder::new();
    builder.prefix(".previous-");
    let backup = builder.tempdir_in(root).map_err(|error| error.to_string())?;
    let previous = backup.path().join(ACTIVE);
    std::fs::rename(destination, &previous).map_err(|error| error.to_string())?;
    let backup = backup.keep();

    let error = match std::fs::rename(source, destination) {
        Ok(()) => {
            let cleanup = std::fs::remove_dir_all(&backup);
            if let Err(error) = cleanup {
                tracing::warn!(%error, path = %backup.display(), "the replaced organization pack remains on disk");
            }
            return Ok(());
        }
        Err(error) => error,
    };
    if let Err(restore) = std::fs::rename(&previous, destination) {
        let retained = previous.display();
        return Err(format!("{error}; restoring the previous pack failed: {restore}; it remains at {retained}"));
    }
    let _ = std::fs::remove_dir(&backup);
    Err(error.to_string())
}

/// Validates an archive, extracts it beside the packs and replaces the pending pack only when it loads.
fn stage(root: &Path, archive: &Path) -> Result<PackSummary, String> {
    let metadata = std::fs::metadata(archive).map_err(|error| error.to_string())?;
    if metadata.len() > DESKTOP_ORGANIZATION_ARCHIVE_BYTES {
        return Err(format!("the organization pack exceeds {} MiB", DESKTOP_ORGANIZATION_ARCHIVE_BYTES / 1024 / 1024));
    }
    let bytes = std::fs::read(archive).map_err(|error| error.to_string())?;
    let files = validate_archive(&bytes)?;
    std::fs::create_dir_all(root).map_err(|error| error.to_string())?;
    let staging = tempfile::Builder::new().prefix(".staging-").tempdir_in(root).map_err(|error| error.to_string())?;
    for (relative, contents) in &files {
        let path = staging.path().join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::write(&path, contents).map_err(|error| error.to_string())?;
    }
    let summary = load(staging.path())?.summary();
    let pending = root.join(PENDING);
    replace_pack(staging.path(), &pending)?;
    Ok(summary)
}

/// The runtime files of an organization folder zipped as is, keyed by their path inside the pack.
fn validate_archive(bytes: &[u8]) -> Result<BTreeMap<PathBuf, Vec<u8>>, String> {
    if bytes.len() as u64 > DESKTOP_ORGANIZATION_ARCHIVE_BYTES {
        return Err(format!("the organization pack exceeds {} MiB", DESKTOP_ORGANIZATION_ARCHIVE_BYTES / 1024 / 1024));
    }
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|error| format!("not a zip archive: {error}"))?;
    let maximum_entries = DESKTOP_ORGANIZATION_ENTRIES;
    if archive.len() as u64 > maximum_entries {
        return Err(format!("the organization pack has more than {maximum_entries} entries"));
    }
    let mut names = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let entry = archive.by_index_raw(index).map_err(|error| error.to_string())?;
        let name = entry.name().map_err(|error| format!("unreadable entry name: {error}"))?.into_owned();
        if name.contains('\\') || name.starts_with('/') || name.contains(':') || entry.enclosed_name().is_none() {
            return Err(format!("unsafe path in the organization pack: {name}"));
        }
        if entry.is_symlink() {
            return Err(format!("links are not allowed in an organization pack: {name}"));
        }
        if entry.encrypted() {
            return Err(format!("encrypted entries are not allowed in an organization pack: {name}"));
        }
        names.push((name, entry.is_dir()));
    }
    let (prefix, manifest) = manifest_location(names.iter().map(|(name, _)| name.as_str()))?;
    let mut files = BTreeMap::new();
    let mut total = 0u64;
    for (index, (name, directory)) in names.iter().enumerate() {
        let Some(relative) = name.strip_prefix(&prefix) else {
            return Err(format!("the organization pack mixes folders: {name}"));
        };
        if *directory || relative.is_empty() {
            continue;
        }
        let path = PathBuf::from(if relative == manifest { MANIFEST } else { relative });
        let extension = path.extension().and_then(|extension| extension.to_str()).unwrap_or_default().to_ascii_lowercase();
        if IGNORED_EXTENSIONS.contains(&extension.as_str()) {
            continue;
        }
        if !RUNTIME_EXTENSIONS.contains(&extension.as_str()) {
            return Err(format!("unsupported file in the organization pack: {relative}"));
        }
        let mut entry = archive.by_index(index).map_err(|error| error.to_string())?;
        let mut contents = Vec::new();
        let remaining = DESKTOP_ORGANIZATION_ARCHIVE_BYTES - total;
        (&mut entry).take(remaining + 1).read_to_end(&mut contents).map_err(|error| error.to_string())?;
        total += contents.len() as u64;
        if total > DESKTOP_ORGANIZATION_ARCHIVE_BYTES {
            return Err(format!("the organization pack expands beyond {} MiB", DESKTOP_ORGANIZATION_ARCHIVE_BYTES / 1024 / 1024));
        }
        if files.insert(path, contents).is_some() {
            return Err(format!("the organization pack repeats {relative}"));
        }
    }
    Ok(files)
}

fn manifest_location<'a>(mut names: impl Iterator<Item = &'a str> + Clone) -> Result<(String, &'static str), String> {
    let mut manifests = Vec::new();
    for name in names.clone() {
        if name.matches('/').count() > 1 {
            continue;
        }
        let filename = name.rsplit('/').next().unwrap_or_default();
        let manifest = match filename {
            MANIFEST => MANIFEST,
            _ => continue,
        };
        let prefix = name.trim_end_matches(manifest).to_owned();
        manifests.push((prefix, manifest));
    }

    let [(prefix, manifest)] = manifests.as_slice() else {
        return Err("the organization pack needs exactly one organization.toml at its root or under one top folder".to_owned());
    };
    if names.any(|name| !name.starts_with(prefix)) {
        return Err("the organization pack must hold one top folder or its files at the root".to_owned());
    }
    Ok((prefix.clone(), *manifest))
}

/// Reads an extracted pack the way the frontend applies it; any missing or unsafe reference fails it whole.
fn load(dir: &Path) -> Result<LoadedPack, String> {
    let pack = dokime::config::organization::resolve(Some(dir))?;
    loaded(pack)
}

fn loaded(pack: dokime::config::organization::ResolvedPack) -> Result<LoadedPack, String> {
    let staging = tempfile::tempdir().map_err(|error| error.to_string())?;
    pack.write_to(staging.path())?;
    let mut loaded = load_resolved(staging.path())?;
    loaded.application_icon = pack.asset("application").map(|(_, bytes)| bytes.to_vec());
    loaded.tray_icon = pack.asset("tray").map(|(_, bytes)| bytes.to_vec());
    for bytes in [&loaded.application_icon, &loaded.tray_icon].into_iter().flatten() {
        tauri::image::Image::from_bytes(bytes).map_err(|_| "organization application and tray artwork must be valid PNG images")?;
    }
    Ok(loaded)
}

fn load_resolved(dir: &Path) -> Result<LoadedPack, String> {
    let text = std::fs::read_to_string(dir.join(MANIFEST)).map_err(|error| format!("organization.toml could not be read: {error}"))?;
    let manifest: Manifest = toml::from_str(&text).map_err(|error: toml::de::Error| {
        let position = error.span().map_or(0, |span| span.start);
        format!("organization.toml does not parse near byte {position}")
    })?;
    if let Some(updates) = &manifest.updates {
        updates.validate()?;
    }
    let app = manifest.app;
    if app.label.trim().is_empty() {
        return Err("organization.toml needs an app label".to_owned());
    }
    let mut logos = BTreeMap::new();
    for (treatment, surface, variant) in LOGO_VARIANTS {
        let Some(file) = app.logos.get(treatment).and_then(|surfaces| surfaces.get(surface)) else { continue };
        let bytes = read_inside(dir, file)?;
        let mime = match Path::new(file).extension().and_then(|extension| extension.to_str()) {
            Some("svg") => "image/svg+xml",
            Some("png") => "image/png",
            _ => return Err(format!("logos must be SVG or PNG: {file}")),
        };
        logos.insert(variant.to_owned(), format!("data:{mime};base64,{}", BASE64.encode(bytes)));
    }
    if logos.is_empty() {
        return Err("organization.toml names no logo the pack contains".to_owned());
    }
    let mut css = String::new();
    for file in THEME_FILES {
        if !dir.join(file).is_file() {
            continue;
        }
        let source = String::from_utf8(read_inside(dir, file)?).map_err(|_| format!("{file} is not UTF-8"))?;
        css.push_str(&format!("/* {file} */\n{}\n", inline_urls(dir, file, &source)?));
        if file == "theme/base.css" {
            css.push_str(&corner_radii(manifest.theme.as_ref()));
        }
    }
    Ok(LoadedPack {
        title_template: app.title_template.filter(|template| !template.trim().is_empty()).unwrap_or_else(|| "{app}".to_owned()),
        label: app.label,
        organization: app.organization,
        logos,
        css,
        updates: manifest.updates,
        application_icon: None,
        tray_icon: None,
    })
}

fn corner_radii(theme: Option<&ThemeSection>) -> String {
    let Some(radii) = theme.map(|theme| &theme.corner_radii).filter(|radii| !radii.is_empty()) else { return String::new() };
    let mut css = String::from(":root {\n");
    for (key, suffix) in
        [("base", ""), ("sm", "-sm"), ("md", "-md"), ("lg", "-lg"), ("xl", "-xl"), ("2xl", "-2xl"), ("3xl", "-3xl"), ("4xl", "-4xl")]
    {
        if let Some(value) = radii.get(key).filter(|value| value.chars().all(|c| c.is_ascii_alphanumeric() || c == '.')) {
            css.push_str(&format!("  --brand-radius{suffix}: {value};\n"));
        }
    }
    css.push_str("}\n");
    css
}

/// Inlines local font and image references; the theme never reaches the network or leaves the pack.
fn inline_urls(dir: &Path, file: &str, css: &str) -> Result<String, String> {
    let stripped = strip_comments(css)?;
    let css = stripped.as_str();
    if css.to_ascii_lowercase().contains("@import") {
        return Err(format!("{file} may not @import other stylesheets"));
    }
    let mut output = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.to_ascii_lowercase().find("url(") {
        output.push_str(&rest[..start]);
        let after = &rest[start + 4..];
        let end = after.find(')').ok_or_else(|| format!("{file} has an unterminated url()"))?;
        let raw = after[..end].trim().trim_matches(['"', '\'']).trim();
        if raw.starts_with("data:") {
            output.push_str(&rest[start..start + 4 + end + 1]);
        } else {
            let mime = match Path::new(raw).extension().and_then(|extension| extension.to_str()) {
                Some("woff2") => "font/woff2",
                Some("svg") => "image/svg+xml",
                Some("png") => "image/png",
                _ => return Err(format!("{file} references an unsupported file: {raw}")),
            };
            let bytes = read_inside(dir, raw)?;
            output.push_str(&format!("url(\"data:{mime};base64,{}\")", BASE64.encode(bytes)));
        }
        rest = &after[end + 1..];
    }
    output.push_str(rest);
    Ok(output)
}

/// Comments may mention `url()` and `@import`; only declarations count.
fn strip_comments(css: &str) -> Result<String, String> {
    let mut output = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        output.push_str(&rest[..start]);
        let end = rest[start + 2..].find("*/").ok_or("a theme stylesheet has an unterminated comment")?;
        rest = &rest[start + 2 + end + 2..];
    }
    output.push_str(rest);
    Ok(output)
}

/// Reads a pack-relative path that cannot escape the pack folder.
fn read_inside(dir: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = Path::new(relative);
    if relative.contains(':') || path.components().any(|component| !matches!(component, Component::Normal(_))) {
        return Err(format!("unsafe path in the organization pack: {relative}"));
    }
    std::fs::read(dir.join(path)).map_err(|error| format!("{relative} could not be read: {error}"))
}

fn remove_folder(path: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(path) {
        Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(format!("{} could not be removed: {error}", path.display())),
        _ => Ok(()),
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    fn boot(root: &Path) -> Boot {
        super::boot(root)
    }
    fn load(path: &Path) -> Result<LoadedPack, String> {
        super::load(path)
    }
    fn stage(root: &Path, archive: &Path) -> Result<PackSummary, String> {
        super::stage(root, archive)
    }
    fn validate_archive(bytes: &[u8]) -> Result<BTreeMap<PathBuf, Vec<u8>>, String> {
        super::validate_archive(bytes)
    }
    use std::io::Write;
    use zip::write::SimpleFileOptions;

    const LOGO: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg'/>";
    const MANIFEST_TEXT: &str = "[app]\nlabel = \"Stokker Technologies\"\n[app.logos.transparent]\non_light = \"logos/mark.svg\"\n";

    fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        for (name, contents) in entries {
            writer.start_file(*name, SimpleFileOptions::default()).unwrap();
            writer.write_all(contents).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    fn organization_folder() -> PathBuf {
        PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory")).join("../../../organization")
    }

    fn zip_folder(root: &Path, prefix: &str) -> Vec<u8> {
        fn walk(root: &Path, dir: &Path, prefix: &str, writer: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                let name = format!("{prefix}{}", path.strip_prefix(root).unwrap().to_str().unwrap().replace('\\', "/"));
                if path.is_dir() {
                    walk(root, &path, prefix, writer);
                } else {
                    writer.start_file(name, SimpleFileOptions::default()).unwrap();
                    writer.write_all(&std::fs::read(&path).unwrap()).unwrap();
                }
            }
        }
        let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        walk(root, root, prefix, &mut writer);
        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn the_repository_organization_folder_zipped_as_is_loads() {
        for prefix in ["", "organization/"] {
            let files = validate_archive(&zip_folder(&organization_folder(), prefix)).unwrap();
            assert!(files.contains_key(Path::new("organization.toml")));
            assert!(files.keys().all(|path| !path.starts_with("generated")));
            assert!(!files.contains_key(Path::new("fonts/OFL.txt")));
            let root = tempfile::tempdir().unwrap();
            for (relative, contents) in &files {
                let path = root.path().join(relative);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, contents).unwrap();
            }
            let pack = load(root.path()).unwrap();
            assert_eq!(pack.label, "Dokime");
            assert_eq!(pack.logos.len(), 4);
            assert!(pack.css.contains("data:font/woff2;base64,"));
            assert!(pack.css.contains("--brand-radius-sm"));
            assert!(!pack.css.contains("url(\"fonts/"));
        }
    }

    #[test]
    fn unsafe_and_oversized_archives_are_refused() {
        for name in ["../organization.toml", "/etc/organization.toml", "logos\\mark.svg", "C:/organization.toml"] {
            assert!(validate_archive(&archive(&[(MANIFEST, MANIFEST_TEXT.as_bytes()), (name, LOGO)])).is_err(), "{name}");
        }
        let many: Vec<(String, &[u8])> =
            (0..=DESKTOP_ORGANIZATION_ENTRIES as usize).map(|index| (format!("logos/{index}.svg"), LOGO)).collect();
        let many: Vec<(&str, &[u8])> = many.iter().map(|(name, contents)| (name.as_str(), *contents)).collect();
        assert!(validate_archive(&archive(&many)).unwrap_err().contains("entries"));
        let large = vec![b' '; DESKTOP_ORGANIZATION_ARCHIVE_BYTES as usize + 1];
        assert!(
            validate_archive(&archive(&[(MANIFEST, MANIFEST_TEXT.as_bytes()), ("theme/base.css", &large)])).unwrap_err().contains("MiB")
        );
        assert!(
            validate_archive(&archive(&[(MANIFEST, MANIFEST_TEXT.as_bytes()), ("run.sh", b"echo")])).unwrap_err().contains("unsupported")
        );
    }

    #[test]
    fn partial_pack_inherits_assets() {
        assert!(validate_archive(&archive(&[("logos/mark.svg", LOGO)])).unwrap_err().contains("organization.toml"));
        let root = tempfile::tempdir().unwrap();
        let pack = archive(&[(MANIFEST, b"[app]\nlabel = \"Stokker Technologies\"\n")]);
        std::fs::write(root.path().join("pack.zip"), pack).unwrap();
        assert_eq!(stage(&root.path().join("organization"), &root.path().join("pack.zip")).unwrap().label, "Stokker Technologies");
        let escaping =
            archive(&[(MANIFEST, b"[app]\nlabel = \"Stokker Technologies\"\n[app.logos.transparent]\non_light = \"../../secret.svg\"\n")]);
        std::fs::write(root.path().join("escape.zip"), escaping).unwrap();
        assert!(stage(&root.path().join("organization"), &root.path().join("escape.zip")).unwrap_err().contains("unsafe"));
    }

    #[test]
    fn a_staged_pack_activates_on_the_next_boot_and_an_invalid_one_falls_back() {
        let root = tempfile::tempdir().unwrap();
        let organization = root.path().join("organization");
        std::fs::write(root.path().join("pack.zip"), archive(&[(MANIFEST, MANIFEST_TEXT.as_bytes()), ("logos/mark.svg", LOGO)])).unwrap();
        let summary = stage(&organization, &root.path().join("pack.zip")).unwrap();
        assert_eq!(summary.label, "Stokker Technologies");
        let booted = boot(&organization);
        assert_eq!(booted.pack.unwrap().label, "Stokker Technologies");
        assert!(!organization.join(PENDING).exists() && organization.join(ACTIVE).is_dir());
        std::fs::write(organization.join(ACTIVE).join(MANIFEST), "not toml [").unwrap();
        let fallback = boot(&organization);
        assert_eq!(fallback.pack.unwrap().label, "Dokime");
        assert!(fallback.error.unwrap().contains("organization.toml"));
        assert!(organization.join(ACTIVE).is_dir(), "an invalid pack stays on disk for diagnosis");
    }

    #[test]
    fn invalid_pending_pack_preserves_the_last_valid_active_pack() {
        let root = tempfile::tempdir().unwrap();
        let organization = root.path().join("organization");
        let archive_path = root.path().join("pack.zip");
        std::fs::write(&archive_path, archive(&[(MANIFEST, MANIFEST_TEXT.as_bytes()), ("logos/mark.svg", LOGO)])).unwrap();
        stage(&organization, &archive_path).unwrap();
        assert_eq!(boot(&organization).pack.unwrap().label, "Stokker Technologies");
        stage(&organization, &archive_path).unwrap();
        std::fs::write(organization.join(PENDING).join(MANIFEST), "invalid [").unwrap();

        let retained = boot(&organization);

        assert_eq!(retained.pack.unwrap().label, "Stokker Technologies");
        assert!(retained.error.unwrap().contains("pending organization pack"));
        assert_eq!(std::fs::read_to_string(organization.join(ACTIVE).join(MANIFEST)).unwrap(), MANIFEST_TEXT);
        assert_eq!(std::fs::read_to_string(organization.join(PENDING).join(MANIFEST)).unwrap(), "invalid [");
    }

    #[test]
    fn failed_promotion_restores_the_active_pack() {
        let root = tempfile::tempdir().unwrap();
        let active = root.path().join(ACTIVE);
        std::fs::create_dir(&active).unwrap();
        std::fs::write(active.join(MANIFEST), MANIFEST_TEXT).unwrap();

        let pending = root.path().join(PENDING);
        assert!(replace_pack(&pending, &active).is_err(), "a missing pending pack cannot replace the active pack");

        assert_eq!(std::fs::read_to_string(active.join(MANIFEST)).unwrap(), MANIFEST_TEXT);
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 1, "rollback removes only its empty backup directory");
    }

    #[test]
    fn invalid_update_policy_preserves_the_previous_pending_pack() {
        let root = tempfile::tempdir().unwrap();
        let store = root.path().join("organization");
        let archive_path = root.path().join("pack.zip");
        let valid = format!("{MANIFEST_TEXT}\n[updates]\nmode = 'disabled'\n");
        std::fs::write(&archive_path, archive(&[(MANIFEST, valid.as_bytes()), ("logos/mark.svg", LOGO)])).unwrap();
        stage(&store, &archive_path).unwrap();

        let invalid = valid.replace("mode = 'disabled'", "mode = 'manual'\nsource = 'missing'");
        std::fs::write(&archive_path, archive(&[(MANIFEST, invalid.as_bytes()), ("logos/mark.svg", LOGO)])).unwrap();
        assert!(stage(&store, &archive_path).unwrap_err().contains("does not exist"));
        let booted = boot(&store);
        let script = serde_json::to_value(BootScript {
            pack: booted.pack.as_ref(),
            error: booted.error.as_deref(),
        })
        .unwrap();
        assert_eq!(script["pack"]["updates"]["mode"], "disabled");
        let updates = booted.pack.unwrap().updates.unwrap();
        assert_eq!(updates.mode, policy::Mode::Disabled);
        assert_eq!(std::fs::read_to_string(store.join(ACTIVE).join(MANIFEST)).unwrap(), valid);
    }

    #[test]
    fn malformed_policy_diagnostics_do_not_echo_manifest_values() {
        let root = tempfile::tempdir().unwrap();
        let manifest = format!("{MANIFEST_TEXT}\n[updates]\nmode = 'private-value-42'\n");
        std::fs::write(root.path().join(MANIFEST), manifest).unwrap();

        let error = load(root.path()).unwrap_err();

        assert!(error.contains("organization update policy is invalid"));
        assert!(!error.contains("private-value-42"));
    }
}
