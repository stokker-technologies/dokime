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

// Local Imports
use crate::domains::{
    AppCustomizationContribution, AppLogoAsset, AppThemeContribution, ThemeActivationPolicy, ThemeCornerRadii, ThemeModePolicy,
};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::plugins::LogoVariant;
pub use dokime_organization::{BrandIdentity, BrandLoadError, EmbeddedLogo, Materialized, variant_name};
use theme::{Bundled, Directory, Source};

/* === Constants === */

#[cfg(feature = "native")]
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

// Manifest Serialization

#[derive(Debug, Deserialize)]
struct BrandManifest {
    #[serde(default)]
    theme: Option<ThemeSection>,
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
    fn into_resolved(self, materialized: Materialized) -> Result<ResolvedBrand, BrandLoadError> {
        let theme_id = Identifier::new(BRAND_THEME_ID).map_err(dokime_domain::Error::from)?;
        let theme = self
            .theme
            .as_ref()
            .map(|theme| -> Result<AppThemeContribution, BrandLoadError> {
                let authored = materialized.theme.as_ref().expect("the materializer read the declared theme");
                Ok(AppThemeContribution {
                    metadata: Metadata::new(theme_id.clone(), &theme.label, theme.description.clone()),
                    stylesheet: materialized.stylesheet(),
                    variables: authored.variables.clone(),
                    corner_radii: theme.corner_radii.clone(),
                    light: authored.light.clone(),
                    dark: authored.dark.clone(),
                })
            })
            .transpose()?;
        let theme_policy = theme_activation_policy(self.theme.as_ref(), &theme_id);

        let customization = AppCustomizationContribution {
            metadata: Metadata::new(
                Identifier::new(BRAND_CUSTOMIZATION_ID).map_err(dokime_domain::Error::from)?,
                &materialized.identity.label,
                Some(format!("Repo brand identity for {}.", materialized.identity.label)),
            ),
            organization: materialized.organization.clone(),
            display_name: Some(materialized.identity.label.clone()),
            title_template: Some(materialized.identity.title_template.clone()),
            logo_assets: Vec::new(),
            theme_policy,
        };

        Ok(ResolvedBrand {
            identity: materialized.identity,
            customization,
            theme,
            logos: materialized.logos,
        })
    }
}

/* === Public Functions === */

pub fn compiled_brand() -> Option<ResolvedBrand> {
    let resolved = parse(Bundled::manifest(), &Bundled);
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
    let mut brand = parse(&text, &source)?;
    for logo in std::mem::take(&mut brand.logos) {
        brand.customization.logo_assets.push(AppLogoAsset {
            variant: logo.variant,
            asset_id: logo_asset_id(logo.variant),
            path: dir.join(&logo.file).to_string_lossy().into_owned(),
            mime_type: logo.mime_type,
            alt: logo.alt,
        });
    }
    Ok(brand)
}

/* === Private Functions === */

pub(crate) fn resolved_pack_brand(pack: &crate::config::organization::ResolvedPack) -> Result<ResolvedBrand, BrandLoadError> {
    let text = toml::to_string(&pack.manifest).expect("TOML values serialize");
    parse(&text, pack)
}

fn parse(text: &str, source: &impl Source) -> Result<ResolvedBrand, BrandLoadError> {
    let manifest: BrandManifest = toml::from_str(text).map_err(|error| BrandLoadError::Parse(Box::new(error)))?;
    manifest.into_resolved(Materialized::read(text, source)?)
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

fn theme_activation_policy(theme: Option<&ThemeSection>, theme_id: &Identifier) -> ThemeActivationPolicy {
    match theme {
        Some(theme) => match theme.policy {
            BrandThemePolicy::RegisterOnly => ThemeActivationPolicy::RegisterOnly,
            BrandThemePolicy::Default => ThemeActivationPolicy::Default { theme: theme_id.clone() },
            BrandThemePolicy::Required => ThemeActivationPolicy::Required {
                theme: theme_id.clone(),
                mode: theme.default_mode,
            },
        },
        None => ThemeActivationPolicy::RegisterOnly,
    }
}

/* === Tests === */
