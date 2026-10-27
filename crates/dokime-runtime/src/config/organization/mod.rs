/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod pack;
pub mod policy;
pub mod theme;

pub use pack::{ResolvedPack, brand, installation_directory, resolve, resolve_with_base};

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

// Third Party Imports
use serde::Deserialize;
use thiserror::Error;

// Local Imports
use crate::domains::{
    AppCustomizationContribution, AppLogoAsset, AppThemeContribution, OrganizationIdentity, ThemeActivationPolicy, ThemeCornerRadii,
    ThemeModePolicy,
};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::plugins::LogoVariant;
use theme::{Bundled, Directory, Error as ThemeCssError, Source, load_from as load_authored_theme_css};

/* === Constants === */

#[cfg(feature = "host")]
pub(crate) const BRAND_ARTIFACT_ID: &str = "brand";
const BRAND_MANIFEST_FILE: &str = "organization.toml";
const BRAND_THEME_ID: &str = "brand.theme";
const BRAND_CUSTOMIZATION_ID: &str = "brand.customization";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct ResolvedBrand {
    pub identity: BrandIdentity,
    pub customization: AppCustomizationContribution,
    pub theme: Option<AppThemeContribution>,
    pub logos: Vec<EmbeddedLogo>,
}

#[derive(Clone, Debug)]
pub struct EmbeddedLogo {
    pub variant: LogoVariant,
    pub file: String,
    pub mime_type: String,
    pub alt: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrandIdentity {
    pub label: String,
    pub title_template: String,
}

// Manifest Serialization

#[derive(Debug, Deserialize)]
struct BrandManifest {
    app: AppSection,

    #[serde(default)]
    theme: Option<ThemeSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppSection {
    label: String,

    #[serde(default)]
    #[serde(rename = "artwork")]
    _artwork: std::collections::BTreeMap<String, String>,

    #[serde(default = "default_title_template")]
    title_template: String,

    #[serde(default)]
    organization: Option<OrganizationSection>,

    #[serde(default)]
    logos: LogoSections,
}

#[derive(Debug, Deserialize)]
struct OrganizationSection {
    name: String,

    #[serde(default)]
    unit: Option<String>,

    #[serde(default)]
    website: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LogoSections {
    #[serde(default)]
    transparent: LogoSurfaces,

    #[serde(default)]
    contained: LogoSurfaces,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct LogoSurfaces {
    on_light: Option<String>,
    on_dark: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ThemeSection {
    #[serde(default)]
    policy: BrandThemePolicy,

    #[serde(default)]
    default_mode: ThemeModePolicy,

    label: String,

    #[serde(default)]
    description: Option<String>,

    #[serde(default)]
    corner_radii: ThemeCornerRadii,
}

/* === Enums === */

#[derive(Debug, Error)]
pub enum BrandLoadError {
    #[error("could not read {path}: {source}")]
    Read { path: std::path::PathBuf, source: std::io::Error },

    #[error("unsafe or unsupported organization asset path: {0}")]
    InvalidAssetPath(std::path::PathBuf),

    #[error("could not parse organization.toml: {0}")]
    Parse(Box<toml::de::Error>),

    #[error("brand identifier is invalid: {0}")]
    Identifier(#[from] dokime_domain::Error),

    #[error("invalid authored brand CSS: {0}")]
    ThemeCss(#[from] ThemeCssError),
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BrandThemePolicy {
    RegisterOnly,

    #[default]
    Default,

    Required,
}

/* === Traits === */

/* === Implementations === */

impl ResolvedBrand {
    pub fn materialize_logos(&mut self, directory: &Path) -> std::io::Result<()> {
        let mut assets = Vec::with_capacity(self.logos.len());
        for logo in &self.logos {
            let path = directory.join(&logo.file);
            if std::fs::read(&path).ok().as_deref() != Some(logo.bytes.as_slice()) {
                harmos::filesystem::publication::write(&path, &logo.bytes)?;
            }
            assets.push(AppLogoAsset {
                variant: logo.variant,
                asset_id: logo_asset_id(logo.variant),
                path: path.to_string_lossy().into_owned(),
                mime_type: logo.mime_type.clone(),
                alt: logo.alt.clone(),
            });
        }
        if !self.logos.is_empty() {
            self.customization.logo_assets = assets;
            self.logos.clear();
        }
        Ok(())
    }
}

impl BrandManifest {
    fn into_resolved(self, source: &impl Source) -> Result<ResolvedBrand, BrandLoadError> {
        let theme_id = Identifier::new(BRAND_THEME_ID).map_err(dokime_domain::Error::from)?;
        let theme = self
            .theme
            .as_ref()
            .map(|theme| -> Result<AppThemeContribution, BrandLoadError> {
                let authored = load_authored_theme_css(source)?;
                Ok(AppThemeContribution {
                    metadata: Metadata::new(theme_id.clone(), &theme.label, theme.description.clone()),
                    stylesheet: Some(authored.combined_css(&theme.corner_radii)),
                    variables: authored.variables,
                    corner_radii: theme.corner_radii.clone(),
                    light: authored.light,
                    dark: authored.dark,
                })
            })
            .transpose()?;
        let theme_policy = theme_activation_policy(self.theme.as_ref(), &theme_id);
        let organization = self.app.organization.as_ref().map(|org| OrganizationIdentity {
            name: org.name.clone(),
            unit: org.unit.clone(),
            website: org.website.clone(),
        });

        let customization = AppCustomizationContribution {
            metadata: Metadata::new(
                Identifier::new(BRAND_CUSTOMIZATION_ID).map_err(dokime_domain::Error::from)?,
                &self.app.label,
                Some(format!("Repo brand identity for {}.", self.app.label)),
            ),
            organization,
            display_name: Some(self.app.label.clone()),
            title_template: Some(self.app.title_template.clone()),
            logo_assets: Vec::new(),
            theme_policy,
        };

        let identity = BrandIdentity { label: self.app.label, title_template: self.app.title_template };

        Ok(ResolvedBrand { identity, customization, theme, logos: Vec::new() })
    }
}

/* === Public Functions === */

pub fn compiled_brand() -> Option<ResolvedBrand> {
    let resolved = parse(Bundled::manifest(), &Bundled).and_then(|(mut brand, logos)| {
        brand.logos = logos
            .into_iter()
            .map(|(variant, name)| -> Result<EmbeddedLogo, BrandLoadError> {
                let file = name.clone();
                let mime_type = logo_mime(&name).to_owned();
                Ok(EmbeddedLogo {
                    variant,
                    bytes: Bundled::file(&file)
                        .ok_or_else(|| BrandLoadError::Read { path: file.into(), source: std::io::ErrorKind::NotFound.into() })?
                        .to_vec(),
                    file: name,
                    mime_type,
                    alt: Some(brand.identity.label.clone()),
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(brand)
    });
    match resolved {
        Ok(brand) => Some(brand),
        Err(error) => {
            tracing::error!(%error, "the compiled-in brand is invalid; using neutral defaults");
            None
        }
    }
}

pub fn load_brand(dir: &Path) -> Result<ResolvedBrand, BrandLoadError> {
    let manifest_path = dir.join(BRAND_MANIFEST_FILE);
    let read_error = |source| BrandLoadError::Read { path: dir.to_path_buf(), source };
    let text = std::fs::read_to_string(&manifest_path).map_err(read_error)?;
    let source = Directory::new(dir).map_err(read_error)?;
    let (mut brand, logos) = parse(&text, &source)?;
    for (variant, file) in logos {
        let path = dir.join(&file);
        if path.is_file() {
            brand.customization.logo_assets.push(AppLogoAsset {
                variant,
                asset_id: logo_asset_id(variant),
                path: path.to_string_lossy().into_owned(),
                mime_type: logo_mime(&file).to_owned(),
                alt: Some(brand.identity.label.clone()),
            });
        } else {
            return Err(BrandLoadError::Read { path, source: std::io::ErrorKind::NotFound.into() });
        }
    }
    Ok(brand)
}

/* === Private Functions === */

pub(crate) fn resolved_pack_brand(pack: &crate::config::organization::ResolvedPack) -> Result<ResolvedBrand, BrandLoadError> {
    let text = toml::to_string(&pack.manifest).expect("TOML values serialize");
    let (mut brand, logos) = parse(&text, pack)?;
    for (variant, file) in logos {
        let bytes = pack.read(&file).map_err(|source| BrandLoadError::Read { path: file.clone().into(), source })?;
        let mime_type = logo_mime(&file).to_owned();
        brand.logos.push(EmbeddedLogo { variant, file, bytes, mime_type, alt: Some(brand.identity.label.clone()) });
    }
    Ok(brand)
}

fn parse(text: &str, source: &impl Source) -> Result<(ResolvedBrand, Vec<(LogoVariant, String)>), BrandLoadError> {
    let mut manifest: BrandManifest = toml::from_str(text).map_err(|error| BrandLoadError::Parse(Box::new(error)))?;
    let logos = std::mem::take(&mut manifest.app.logos);
    let logos = [
        (LogoVariant::TransparentOnLight, logos.transparent.on_light),
        (LogoVariant::TransparentOnDark, logos.transparent.on_dark),
        (LogoVariant::ContainedOnLight, logos.contained.on_light),
        (LogoVariant::ContainedOnDark, logos.contained.on_dark),
    ]
    .into_iter()
    .filter_map(|(variant, file)| file.map(|file| (variant, file)))
    .collect::<Vec<_>>();
    for (_, file) in &logos {
        if file.contains(['\\', ':'])
            || Path::new(file).components().any(|part| !matches!(part, std::path::Component::Normal(_)))
            || !(file.ends_with(".svg") || file.ends_with(".png"))
        {
            return Err(BrandLoadError::InvalidAssetPath(file.into()));
        }
        if file.ends_with(".png") {
            let bytes = source.read(file).map_err(|source| BrandLoadError::Read { path: file.into(), source })?;
            image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
                .map_err(|_| BrandLoadError::Read { path: file.into(), source: std::io::ErrorKind::InvalidData.into() })?;
        }
    }
    Ok((manifest.into_resolved(source)?, logos))
}

fn logo_asset_id(variant: LogoVariant) -> Identifier {
    Identifier::new(match variant {
        LogoVariant::TransparentOnLight => "brand.logo.transparent-on-light",
        LogoVariant::TransparentOnDark => "brand.logo.transparent-on-dark",
        LogoVariant::ContainedOnLight => "brand.logo.contained-on-light",
        LogoVariant::ContainedOnDark => "brand.logo.contained-on-dark",
    })
    .expect("valid asset identifier")
}

fn logo_mime(file: &str) -> &'static str {
    if file.ends_with(".png") { "image/png" } else { "image/svg+xml" }
}

fn default_title_template() -> String {
    "{app}".to_owned()
}

fn theme_activation_policy(theme: Option<&ThemeSection>, theme_id: &Identifier) -> ThemeActivationPolicy {
    match theme {
        Some(theme) => match theme.policy {
            BrandThemePolicy::RegisterOnly => ThemeActivationPolicy::RegisterOnly,
            BrandThemePolicy::Default => ThemeActivationPolicy::Default { theme: theme_id.clone() },
            BrandThemePolicy::Required => ThemeActivationPolicy::Required { theme: theme_id.clone(), mode: theme.default_mode },
        },
        None => ThemeActivationPolicy::RegisterOnly,
    }
}

/* === Tests === */
