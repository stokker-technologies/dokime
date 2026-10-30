/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub mod theme;

use std::path::Path;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use dokime_domain::model::plugins::{LogoVariant, OrganizationIdentity, ThemeCornerRadii};
use serde::{Deserialize, Serialize, ser::SerializeMap};
use theme::Source;

#[derive(Clone, Debug)]
pub struct Materialized {
    pub identity: BrandIdentity,
    pub organization: Option<OrganizationIdentity>,
    pub logos: Vec<EmbeddedLogo>,
    pub theme: Option<theme::Authored>,
    pub corner_radii: ThemeCornerRadii,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrandIdentity {
    pub label: String,
    pub title_template: String,
}

#[derive(Clone, Debug)]
pub struct EmbeddedLogo {
    pub variant: LogoVariant,
    pub file: String,
    pub mime_type: String,
    pub alt: Option<String>,
    pub bytes: Vec<u8>,
}

#[derive(Deserialize)]
struct Manifest {
    app: App,
    theme: Option<Theme>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct App {
    label: String,
    #[serde(default = "default_title_template")]
    title_template: String,
    organization: Option<OrganizationIdentity>,
    #[serde(default)]
    logos: Logos,
    #[serde(default, rename = "artwork")]
    _artwork: std::collections::BTreeMap<String, String>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Logos {
    #[serde(default)]
    transparent: Surfaces,
    #[serde(default)]
    contained: Surfaces,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Surfaces {
    on_light: Option<String>,
    on_dark: Option<String>,
}

#[derive(Deserialize)]
struct Theme {
    #[serde(default)]
    corner_radii: ThemeCornerRadii,
}

#[derive(Debug, thiserror::Error)]
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
    ThemeCss(#[from] theme::Error),
}

impl Materialized {
    pub fn read(text: &str, source: &impl Source) -> Result<Self, BrandLoadError> {
        let manifest: Manifest = toml::from_str(text).map_err(|error| BrandLoadError::Parse(Box::new(error)))?;
        let app = manifest.app;
        let mut logos = Vec::new();
        for (variant, file) in [
            (LogoVariant::TransparentOnLight, app.logos.transparent.on_light),
            (LogoVariant::TransparentOnDark, app.logos.transparent.on_dark),
            (LogoVariant::ContainedOnLight, app.logos.contained.on_light),
            (LogoVariant::ContainedOnDark, app.logos.contained.on_dark),
        ] {
            let Some(file) = file else { continue };
            if file.contains(['\\', ':'])
                || Path::new(&file).components().any(|part| !matches!(part, std::path::Component::Normal(_)))
                || !(file.ends_with(".svg") || file.ends_with(".png"))
            {
                return Err(BrandLoadError::InvalidAssetPath(file.into()));
            }
            let bytes = source.read(&file).map_err(|source| BrandLoadError::Read { path: file.clone().into(), source })?;
            let mime_type = if file.ends_with(".png") {
                image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).map_err(|_| BrandLoadError::Read {
                    path: file.clone().into(),
                    source: std::io::ErrorKind::InvalidData.into(),
                })?;
                "image/png"
            } else {
                "image/svg+xml"
            };
            logos.push(EmbeddedLogo {
                variant,
                file,
                bytes,
                mime_type: mime_type.into(),
                alt: Some(app.label.clone()),
            });
        }
        let theme = manifest.theme.as_ref().map(|_| theme::load_from(source)).transpose()?;
        Ok(Self {
            identity: BrandIdentity {
                label: app.label,
                title_template: app.title_template,
            },
            organization: app.organization,
            logos,
            theme,
            corner_radii: manifest.theme.map(|theme| theme.corner_radii).unwrap_or_default(),
        })
    }

    pub fn stylesheet(&self) -> Option<String> {
        self.theme.as_ref().map(|theme| theme.combined_css(&self.corner_radii))
    }

    /// The frontend's compiled default projection. Field order is retained for reproducible assets.
    pub fn default_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct DefaultOrganization<'a> {
            label: &'a str,
            title: String,
            logos: WebLogos<'a>,
        }
        let default = DefaultOrganization {
            label: &self.identity.label,
            title: self.identity.title_template.replace("{app}", &self.identity.label),
            logos: WebLogos(&self.logos),
        };
        Ok(serde_json::to_string_pretty(&default)? + "\n")
    }
}

impl EmbeddedLogo {
    pub fn data_uri(&self) -> String {
        format!("data:{};base64,{}", self.mime_type, BASE64.encode(&self.bytes))
    }
}

pub fn variant_name(variant: LogoVariant) -> &'static str {
    match variant {
        LogoVariant::TransparentOnLight => "transparent_on_light",
        LogoVariant::TransparentOnDark => "transparent_on_dark",
        LogoVariant::ContainedOnLight => "contained_on_light",
        LogoVariant::ContainedOnDark => "contained_on_dark",
    }
}

struct WebLogos<'a>(&'a [EmbeddedLogo]);

impl Serialize for WebLogos<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Logo<'a> {
            src: String,
            alt: &'a Option<String>,
        }
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for logo in self.0 {
            map.serialize_entry(variant_name(logo.variant), &Logo { src: logo.data_uri(), alt: &logo.alt })?;
        }
        map.end()
    }
}

fn default_title_template() -> String {
    "{app}".into()
}
