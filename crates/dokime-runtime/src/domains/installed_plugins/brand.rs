/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod theme;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::path::Path;
// Third Party Imports
use serde::Deserialize;
use thiserror::Error;
// Local Imports
use crate::core::{Identifier, Metadata};
use crate::domains::{
    AppCustomizationContribution, AppLogoAsset, AppThemeContribution, OrganizationIdentity,
    ThemeActivationPolicy, ThemeCornerRadii, ThemeModePolicy,
};
use theme::{
    Bundled, Directory, Error as ThemeCssError, Source, load_from as load_authored_theme_css,
};

/* === Constants === */
pub(crate) const BRAND_ARTIFACT_ID: &str = "brand";

const BRAND_MANIFEST_FILE: &str = "brand.toml";

const BRAND_LOGOS_DIR: &str = "logos";

const BRAND_THEME_ID: &str = "brand.theme";
const BRAND_CUSTOMIZATION_ID: &str = "brand.customization";
const BRAND_LOGO_ASSET_ID: &str = "brand.logo";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug)]
pub struct ResolvedBrand {
    pub identity: BrandIdentity,
    pub customization: AppCustomizationContribution,
    pub theme: Option<AppThemeContribution>,
    pub logo: Option<EmbeddedLogo>,
}

#[derive(Clone, Debug)]
pub struct EmbeddedLogo {
    pub file: String,
    pub mime_type: String,
    pub alt: Option<String>,
    pub bytes: &'static [u8],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrandIdentity {
    pub label: String,
    pub slug: String,
    pub identifier: String,
    pub deep_link_scheme: Option<String>,
    pub title_template: String,
}

/* === Manifest (serde) === */

#[derive(Debug, Deserialize)]
struct BrandManifest {
    app: AppSection,
    #[serde(default)]
    theme: Option<ThemeSection>,
}

#[derive(Debug, Deserialize)]
struct AppSection {
    label: String,
    slug: String,
    identifier: String,
    #[serde(default)]
    deep_link_scheme: Option<String>,
    #[serde(default = "default_title_template")]
    title_template: String,
    #[serde(default)]
    organization: Option<OrganizationSection>,
    #[serde(default)]
    logo: Option<LogoSection>,
}

#[derive(Debug, Deserialize)]
struct OrganizationSection {
    name: String,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    website: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LogoSection {
    file: String,
    mime_type: String,
    #[serde(default)]
    alt: Option<String>,
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
    Read {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    #[error("could not parse brand.toml: {0}")]
    Parse(Box<toml::de::Error>),
    #[error("brand identifier is invalid: {0}")]
    Identifier(#[from] dokime_contract::Error),
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
    pub fn materialize_logo(&mut self, directory: &Path) -> std::io::Result<()> {
        let Some(logo) = self.logo.take() else {
            return Ok(());
        };
        let path = directory.join(&logo.file);
        if std::fs::read(&path).ok().as_deref() != Some(logo.bytes) {
            dokime_filesystem::publication::write(&path, logo.bytes)?;
        }
        self.customization.logo_assets = vec![AppLogoAsset {
            asset_id: Identifier::new(BRAND_LOGO_ASSET_ID).expect("valid asset identifier"),
            path: path.to_string_lossy().into_owned(),
            mime_type: logo.mime_type,
            alt: logo.alt,
        }];
        Ok(())
    }
}

impl BrandManifest {
    fn into_resolved(self, source: &impl Source) -> Result<ResolvedBrand, BrandLoadError> {
        let theme_id = Identifier::new(BRAND_THEME_ID).map_err(dokime_contract::Error::from)?;
        let theme = self
            .theme
            .as_ref()
            .map(|theme| -> Result<AppThemeContribution, BrandLoadError> {
                let authored = load_authored_theme_css(source)?;
                Ok(AppThemeContribution {
                    metadata: Metadata::new(
                        theme_id.clone(),
                        &theme.label,
                        theme.description.clone(),
                    ),
                    stylesheet: Some(authored.combined_css(&theme.corner_radii)),
                    variables: authored.variables,
                    corner_radii: theme.corner_radii.clone(),
                    light: authored.light,
                    dark: authored.dark,
                })
            })
            .transpose()?;
        let theme_policy = theme_activation_policy(self.theme.as_ref(), &theme_id);
        let organization = self
            .app
            .organization
            .as_ref()
            .map(|org| OrganizationIdentity {
                name: org.name.clone(),
                unit: org.unit.clone(),
                website: org.website.clone(),
            });

        let customization = AppCustomizationContribution {
            metadata: Metadata::new(
                Identifier::new(BRAND_CUSTOMIZATION_ID).map_err(dokime_contract::Error::from)?,
                &self.app.label,
                Some(format!("Repo brand identity for {}.", self.app.label)),
            ),
            organization,
            display_name: Some(self.app.label.clone()),
            title_template: Some(self.app.title_template.clone()),
            logo_assets: Vec::new(),
            theme_policy,
        };

        let identity = BrandIdentity {
            label: self.app.label,
            slug: self.app.slug,
            identifier: self.app.identifier,
            deep_link_scheme: self.app.deep_link_scheme,
            title_template: self.app.title_template,
        };

        Ok(ResolvedBrand {
            identity,
            customization,
            theme,
            logo: None,
        })
    }
}

/* === Public Functions === */
pub fn compiled_brand() -> Option<ResolvedBrand> {
    let resolved = parse(Bundled::manifest(), &Bundled).and_then(|(mut brand, logo)| {
        brand.logo = logo
            .map(|logo| -> Result<EmbeddedLogo, BrandLoadError> {
                let file = format!("{BRAND_LOGOS_DIR}/{}", logo.file);
                Ok(EmbeddedLogo {
                    bytes: Bundled::file(&file).ok_or_else(|| BrandLoadError::Read {
                        path: file.into(),
                        source: std::io::ErrorKind::NotFound.into(),
                    })?,
                    file: logo.file,
                    mime_type: logo.mime_type,
                    alt: logo.alt,
                })
            })
            .transpose()?;
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
    let read_error = |source| BrandLoadError::Read {
        path: dir.to_path_buf(),
        source,
    };
    let text = std::fs::read_to_string(&manifest_path).map_err(read_error)?;
    let source = Directory::new(dir).map_err(read_error)?;
    let (mut brand, logo) = parse(&text, &source)?;
    if let Some(logo) = logo {
        let path = dir.join(BRAND_LOGOS_DIR).join(&logo.file);
        if path.is_file() {
            brand.customization.logo_assets = vec![AppLogoAsset {
                asset_id: Identifier::new(BRAND_LOGO_ASSET_ID)
                    .map_err(dokime_contract::Error::from)?,
                path: path.to_string_lossy().into_owned(),
                mime_type: logo.mime_type,
                alt: logo.alt,
            }];
        } else {
            tracing::warn!(path = %path.display(), "brand logo not found; rendering without one");
        }
    }
    Ok(brand)
}

/* === Private Functions === */
fn parse(
    text: &str,
    source: &impl Source,
) -> Result<(ResolvedBrand, Option<LogoSection>), BrandLoadError> {
    let mut manifest: BrandManifest =
        toml::from_str(text).map_err(|error| BrandLoadError::Parse(Box::new(error)))?;
    let logo = manifest.app.logo.take();
    Ok((manifest.into_resolved(source)?, logo))
}

fn default_title_template() -> String {
    "{app}".to_owned()
}

fn theme_activation_policy(
    theme: Option<&ThemeSection>,
    theme_id: &Identifier,
) -> ThemeActivationPolicy {
    match theme {
        Some(theme) => match theme.policy {
            BrandThemePolicy::RegisterOnly => ThemeActivationPolicy::RegisterOnly,
            BrandThemePolicy::Default => ThemeActivationPolicy::Default {
                theme: theme_id.clone(),
            },
            BrandThemePolicy::Required => ThemeActivationPolicy::Required {
                theme: theme_id.clone(),
                mode: theme.default_mode,
            },
        },
        None => ThemeActivationPolicy::RegisterOnly,
    }
}

/* === Tests === */
