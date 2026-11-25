/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Installed Organization Pack

/* === Modules === */

pub mod policy;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
    str,
};

// Third Party Imports
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};

// Local Imports
use crate::installation;

/* === Constants === */

const DEFAULT_INSTALLER_SIDEBAR_COLOR: [u8; 3] = [0x1b, 0x19, 0x15];
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

// One Complete Validated Organization Pack.
#[derive(Clone, Debug)]
pub struct ResolvedPack {
    pub manifest: toml::Value,
    pub files: BTreeMap<String, Vec<u8>>,
}

// The Appearance a Pack Declares, Self-Contained for Any Page That Applies It.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Appearance {
    pub label: String,
    pub title_template: String,
    pub organization: Option<Organization>,

    // Logo Variant to `data:` URI.
    pub logos: BTreeMap<String, String>,

    // Every Theme Stylesheet With Fonts and Images Inlined.
    pub css: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organization {
    pub name: String,

    #[serde(default)]
    pub unit: Option<String>,

    #[serde(default)]
    pub website: Option<String>,
}

#[derive(Deserialize)]
struct Manifest {
    app: AppSection,

    #[serde(default)]
    theme: Option<ThemeSection>,
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

impl ResolvedPack {
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

    pub fn appearance(&self) -> Result<Appearance, String> {
        let manifest: Manifest =
            self.manifest.clone().try_into().map_err(|_| "organization.toml could not be decoded; correct the pack manifest")?;
        let app = manifest.app;
        if app.label.trim().is_empty() {
            return Err("organization.toml needs an app label".to_owned());
        }
        let mut logos = BTreeMap::new();
        for (treatment, surface, variant) in LOGO_VARIANTS {
            let Some(file) = app.logos.get(treatment).and_then(|surfaces| surfaces.get(surface)) else { continue };
            let bytes = self.read_inside(file)?;
            let mime = match Path::new(file).extension().and_then(|extension| extension.to_str()) {
                Some("svg") => "image/svg+xml",
                Some("png") => "image/png",
                _ => return Err(format!("logos must be SVG or PNG: {file}")),
            };
            logos.insert(variant.to_owned(), format!("data:{mime};base64,{}", BASE64.encode(bytes)));
        }
        let mut css = String::new();
        for file in THEME_FILES {
            if !self.files.contains_key(file) {
                continue;
            }
            let source = str::from_utf8(self.read_inside(file)?).map_err(|_| format!("{file} is not UTF-8"))?;
            css.push_str(&format!("/* {file} */\n{}\n", self.inline_urls(file, source)?));
            if file == "theme/base.css" {
                css.push_str(&corner_radii(manifest.theme.as_ref()));
            }
        }
        Ok(Appearance {
            title_template: app.title_template.filter(|template| !template.trim().is_empty()).unwrap_or_else(|| "{app}".to_owned()),
            label: app.label,
            organization: app.organization,
            logos,
            css,
        })
    }

    // Inlines local font and image references; the theme never reaches the network or leaves the pack.
    fn inline_urls(&self, file: &str, css: &str) -> Result<String, String> {
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
                let bytes = self.read_inside(raw)?;
                output.push_str(&format!("url(\"data:{mime};base64,{}\")", BASE64.encode(bytes)));
            }
            rest = &after[end + 1..];
        }
        output.push_str(rest);
        Ok(output)
    }

    // Reads a pack-relative path that cannot escape the pack folder.
    fn read_inside(&self, relative: &str) -> Result<&[u8], String> {
        let path = Path::new(relative);
        if relative.contains(':') || path.components().any(|component| !matches!(component, Component::Normal(_))) {
            return Err(format!("unsafe path in the organization pack: {relative}"));
        }
        let relative = relative.replace('\\', "/");
        self.files
            .get(&relative)
            .map(Vec::as_slice)
            .ok_or_else(|| format!("{relative} is missing from the organization pack; add the referenced file"))
    }
}

/* === Public Functions === */

/// The Organization Pack Folder of an Installation, When It Holds a Manifest.
pub fn installed(installation: &Path) -> Option<PathBuf> {
    let directory = installation.join(installation::ORGANIZATION);
    directory.join("organization.toml").is_file().then_some(directory)
}

/// Read and Validate the Pack in a Folder.
pub fn read(directory: &Path) -> Result<ResolvedPack, String> {
    let canonical = directory.canonicalize().map_err(|error| error.to_string())?;
    let mut files = BTreeMap::new();
    collect_files(&canonical, &canonical, &mut files)?;
    resolve(files, directory)
}

/// Validate a Pack Supplied as Pack-Relative Files, Such as One Embedded in a Product.
pub fn from_files(files: BTreeMap<String, Vec<u8>>) -> Result<ResolvedPack, String> {
    resolve(files, Path::new(installation::ORGANIZATION))
}

/* === Private Functions === */

fn resolve(mut files: BTreeMap<String, Vec<u8>>, directory: &Path) -> Result<ResolvedPack, String> {
    let bytes = files.get("organization.toml").ok_or("the organization has no manifest")?;
    let text = str::from_utf8(bytes).map_err(|_| "organization manifest is not UTF-8")?;
    let manifest = parse_manifest(&directory.join("organization.toml"), text)?;
    let normalized = toml::to_string(&manifest).map_err(|error| error.to_string())?;
    files.insert("organization.toml".into(), normalized.into_bytes());
    let pack = ResolvedPack { manifest, files };
    validate(&pack)?;
    Ok(pack)
}

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
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
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
            files.insert(relative, fs::read(path).map_err(|error| error.to_string())?);
        }
    }
    Ok(())
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
    fn malformed_policy_diagnostics_do_not_echo_manifest_values() {
        let root = tempfile::tempdir().unwrap();
        let manifest = "[app]\nlabel = \"Example Lab\"\n[updates]\nmode = 'private-value-42'\n";
        fs::write(root.path().join("organization.toml"), manifest).unwrap();
        let error = read(root.path()).unwrap_err();
        assert!(error.contains("organization update policy is invalid"));
        assert!(!error.contains("private-value-42"));
    }

    #[test]
    fn stylesheets_inline_pack_files_and_refuse_outside_references() {
        let files: [(&str, &[u8], &str); 3] = [
            ("fonts/test.woff2", &[0, 1, 2, 255], "font/woff2"),
            ("logos/test.svg", b"<svg xmlns='http://www.w3.org/2000/svg'/>", "image/svg+xml"),
            ("images/test.png", &[137, 80, 78, 71], "image/png"),
        ];
        let pack = ResolvedPack {
            manifest: toml::Value::Table(Default::default()),
            files: files.iter().map(|(name, bytes, _)| ((*name).to_owned(), bytes.to_vec())).collect(),
        };
        let css = "/* fixture */@font-face{src:url('fonts/test.woff2')}a{background:url(logos/test.svg)}b{background:url(images/test.png)}";
        let mut expected = css.replace("/* fixture */", "");
        for (name, bytes, mime) in files {
            let url = format!("url(\"data:{mime};base64,{}\")", BASE64.encode(bytes));
            expected = expected.replace(&format!("url('{name}')"), &url).replace(&format!("url({name})"), &url);
        }
        assert_eq!(pack.inline_urls("theme/base.css", css).unwrap(), expected);
        assert!(pack.inline_urls("theme/base.css", "a{background:url(../outside.svg)}").is_err());
        assert!(pack.inline_urls("theme/base.css", "a{background:url(missing.svg)}").is_err());
    }
}
