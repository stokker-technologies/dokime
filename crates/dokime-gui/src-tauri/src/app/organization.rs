/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Installed Organization Appearance

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use tauri::Manager;

// Local Imports
use super::updates::policy;
use dokime::config::organization::ResolvedPack;

/* === Constants === */

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

pub(crate) struct OrganizationState {
    pack: Option<LoadedPack>,
    error: Option<String>,
}

// The pack as the frontend applies it before it renders.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoadedPack {
    label: String,
    title_template: String,
    organization: Option<Organization>,

    // Logo variant to `data:` URI.
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
    pub(crate) fn label(&self) -> &str {
        self.pack.as_ref().map(|pack| pack.label.as_str()).unwrap_or(crate::PRODUCT)
    }

    pub(crate) fn window_icon(&self) -> Option<tauri::image::Image<'static>> {
        let bytes = self.pack.as_ref()?.application_icon.as_deref()?;
        tauri::image::Image::from_bytes(bytes).ok()
    }

    pub(crate) fn title(&self) -> Option<String> {
        let pack = self.pack.as_ref()?;
        Some(pack.title_template.replace("{app}", &pack.label))
    }

    pub(crate) fn update_policy(&self) -> Result<policy::UpdatePolicy, String> {
        let updates = self.pack.as_ref().and_then(|pack| pack.updates.clone()).unwrap_or_default();
        updates.validate()?;
        Ok(updates)
    }

    // The script every main window runs before the frontend loads.
    pub(crate) fn initialization_script(&self) -> String {
        let boot = BootScript {
            pack: self.pack.as_ref(),
            error: self.error.as_deref(),
        };
        let json = serde_json::to_string(&boot).unwrap_or_else(|_| "{}".to_owned());
        format!("window.__DOKIME_ORGANIZATION_PACK__ = {json};")
    }
}

/* === Public Functions === */

pub(crate) fn resolve() -> OrganizationState {
    match dokime::config::organization::resolve().and_then(loaded) {
        Ok(pack) => OrganizationState { pack: Some(pack), error: None },
        Err(error) => OrganizationState { pack: None, error: Some(error) },
    }
}

pub(crate) fn setup(app: &tauri::App) -> tauri::Result<()> {
    let state = app.state::<OrganizationState>();
    if let Some(error) = &state.error {
        tracing::error!(%error, "the installed organization could not be loaded");
    }
    if let Some(pack) = &state.pack
        && let Some(bytes) = &pack.tray_icon
    {
        let icon = tauri::image::Image::from_bytes(bytes)?;
        let menu = tauri::menu::Menu::new(app)?;
        let tray = tauri::tray::TrayIconBuilder::with_id("organization").icon(icon).tooltip(&pack.label).menu(&menu).build(app)?;
        app.manage(tray);
    }
    Ok(())
}

/* === Private Functions === */

fn loaded(pack: ResolvedPack) -> Result<LoadedPack, String> {
    let mut loaded = load_resolved(&pack)?;
    loaded.application_icon = pack.asset("application").map(|(_, bytes)| bytes.to_vec());
    loaded.tray_icon = pack.asset("tray").map(|(_, bytes)| bytes.to_vec());
    for bytes in [&loaded.application_icon, &loaded.tray_icon].into_iter().flatten() {
        tauri::image::Image::from_bytes(bytes).map_err(|_| "organization application and tray artwork must be valid PNG images")?;
    }
    Ok(loaded)
}

fn load_resolved(pack: &ResolvedPack) -> Result<LoadedPack, String> {
    let manifest: Manifest =
        pack.manifest.clone().try_into().map_err(|_| "organization.toml could not be decoded; correct the pack manifest")?;
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
        let bytes = read_inside(pack, file)?;
        let mime = match Path::new(file).extension().and_then(|extension| extension.to_str()) {
            Some("svg") => "image/svg+xml",
            Some("png") => "image/png",
            _ => return Err(format!("logos must be SVG or PNG: {file}")),
        };
        logos.insert(variant.to_owned(), format!("data:{mime};base64,{}", BASE64.encode(bytes)));
    }
    let mut css = String::new();
    for file in THEME_FILES {
        if !pack.files.contains_key(file) {
            continue;
        }
        let source = std::str::from_utf8(read_inside(pack, file)?).map_err(|_| format!("{file} is not UTF-8"))?;
        css.push_str(&format!("/* {file} */\n{}\n", inline_urls(pack, file, source)?));
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

// Inlines local font and image references; the theme never reaches the network or leaves the pack.
fn inline_urls(pack: &ResolvedPack, file: &str, css: &str) -> Result<String, String> {
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
            let bytes = read_inside(pack, raw)?;
            output.push_str(&format!("url(\"data:{mime};base64,{}\")", BASE64.encode(bytes)));
        }
        rest = &after[end + 1..];
    }
    output.push_str(rest);
    Ok(output)
}

// Comments may mention `url()` and `@import`; only declarations count.
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

// Reads a pack-relative path that cannot escape the pack folder.
fn read_inside<'a>(pack: &'a ResolvedPack, relative: &str) -> Result<&'a [u8], String> {
    let path = Path::new(relative);
    if relative.contains(':') || path.components().any(|component| !matches!(component, Component::Normal(_))) {
        return Err(format!("unsafe path in the organization pack: {relative}"));
    }
    let relative = relative.replace('\\', "/");
    pack.files
        .get(&relative)
        .map(Vec::as_slice)
        .ok_or_else(|| format!("{relative} is missing from the organization pack; add the referenced file"))
}

#[cfg(feature = "testing")]
#[allow(dead_code)]
pub(crate) mod testing {
    /* === Modules === */

    /* === Re-Exports === */

    /* === Imports === */

    // Standard Library Imports

    // Third Party Imports

    // Local Imports

    use super::*;

    /* === Constants === */

    /* === Macros === */

    /* === Type Aliases === */

    /* === Structs === */

    /* === Enums === */

    /* === Traits === */

    /* === Implementations === */

    /* === Public Functions === */

    pub fn inspect_resolved(pack: ResolvedPack) -> Result<serde_json::Value, String> {
        serde_json::to_value(loaded(pack)?).map_err(|error| error.to_string())
    }

    pub fn inspect_css(pack: &ResolvedPack, css: &str) -> Result<String, String> {
        inline_urls(pack, "theme/base.css", css)
    }

    /* === Private Functions === */

    /* === Tests === */
}

/* === Tests === */
