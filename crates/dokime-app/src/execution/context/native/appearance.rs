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
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use crate::installed_plugins::{RuntimeAppLogo, RuntimeAppearanceFrame, RuntimeThemeFamily};
use crate::{Error, Result};
use dokime::core::Identifier;
use dokime::domains::{OrganizationIdentity, ThemeCornerRadii, ThemeModePolicy, ThemeVariant};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Appearance {
    app_name: String,
    organization: Option<Organization>,
    title_template: Option<String>,
    logo: Option<Logo>,
    themes: Vec<Theme>,
    active_theme: Option<String>,
    theme_family_locked: bool,
    mode_policy: u32,
    mode_locked: bool,
    source_plugin_id: Option<String>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Organization {
    name: String,
    unit: Option<String>,
    website: Option<String>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Logo {
    plugin_id: String,
    asset_id: String,
    url: String,
    alt: Option<String>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Theme {
    plugin_id: String,
    theme_id: String,
    label: String,
    description: Option<String>,
    stylesheet: Option<String>,
    variables: Vec<Property>,
    radii: Option<Radii>,
    light: Option<Variant>,
    dark: Option<Variant>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Property {
    key: String,
    value: String,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Variant {
    variables: Vec<Property>,
    plot_palette: Vec<String>,
}
#[harmos::message]
#[derive(Clone, PartialEq)]
struct Radii {
    base: Option<String>,
    sm: Option<String>,
    md: Option<String>,
    lg: Option<String>,
    xl: Option<String>,
    two_xl: Option<String>,
    three_xl: Option<String>,
    four_xl: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Appearance {
    pub(crate) fn projection_memory(&self) -> Result<usize> {
        mode(self.mode_policy)?;
        let mut bytes = std::mem::size_of::<RuntimeAppearanceFrame>()
            .saturating_add(self.app_name.capacity())
            .saturating_add(optional_capacity(&self.title_template))
            .saturating_add(optional_capacity(&self.active_theme))
            .saturating_add(optional_capacity(&self.source_plugin_id));
        if let Some(organization) = &self.organization {
            bytes = bytes
                .saturating_add(organization.name.capacity())
                .saturating_add(optional_capacity(&organization.unit))
                .saturating_add(optional_capacity(&organization.website));
        }
        if let Some(logo) = &self.logo {
            bytes = bytes
                .saturating_add(logo.plugin_id.capacity())
                .saturating_add(logo.asset_id.capacity())
                .saturating_add(logo.url.capacity())
                .saturating_add(optional_capacity(&logo.alt));
        }
        for theme in &self.themes {
            bytes = bytes.saturating_add(theme.projection_memory()?);
        }
        let validation = crate::execution::context::appearance_validation_memory(
            self.themes.iter().flat_map(Theme::css_values),
            self.themes
                .iter()
                .map(|theme| theme.theme_id.len())
                .max()
                .unwrap_or(0),
        );
        Ok(bytes.saturating_add(validation))
    }

    pub(crate) fn encoding_memory(frame: &RuntimeAppearanceFrame) -> usize {
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(frame.app_name.len())
            .saturating_add(optional_length(&frame.title_template))
            .saturating_add(
                frame
                    .active_theme
                    .as_ref()
                    .map_or(0, |id| id.as_str().len()),
            )
            .saturating_add(
                frame
                    .source_plugin_id
                    .as_ref()
                    .map_or(0, |id| id.as_str().len()),
            );
        if let Some(organization) = &frame.organization {
            bytes = bytes
                .saturating_add(organization.name.len())
                .saturating_add(optional_length(&organization.unit))
                .saturating_add(optional_length(&organization.website));
        }
        if let Some(logo) = &frame.logo {
            bytes = bytes
                .saturating_add(logo.plugin_id.as_str().len())
                .saturating_add(logo.asset_id.as_str().len())
                .saturating_add(logo.url.len())
                .saturating_add(optional_length(&logo.alt));
        }
        for theme in &frame.themes {
            bytes = bytes
                .saturating_add(std::mem::size_of::<Theme>())
                .saturating_add(theme.plugin_id.as_str().len())
                .saturating_add(theme.theme_id.as_str().len())
                .saturating_add(theme.label.len())
                .saturating_add(optional_length(&theme.description))
                .saturating_add(optional_length(&theme.stylesheet))
                .saturating_add(property_encoding(&theme.variables));
            for radius in domain_radii(&theme.corner_radii).into_iter().flatten() {
                bytes = bytes.saturating_add(radius.len());
            }
            for variant in [&theme.light, &theme.dark] {
                bytes = bytes
                    .saturating_add(property_encoding(&variant.variables))
                    .saturating_add(
                        variant
                            .plot_palette
                            .len()
                            .saturating_mul(std::mem::size_of::<String>()),
                    );
                for color in &variant.plot_palette {
                    bytes = bytes.saturating_add(color.len());
                }
            }
        }
        bytes
    }
}

impl TryFrom<&RuntimeAppearanceFrame> for Appearance {
    type Error = Error;
    fn try_from(frame: &RuntimeAppearanceFrame) -> Result<Self> {
        if Self::encoding_memory(frame) == usize::MAX {
            return Err(invalid("appearance encoding memory overflows"));
        }
        let mut themes = slots(frame.themes.len())?;
        for theme in &frame.themes {
            themes.push(Theme::try_from(theme)?);
        }
        Ok(Self {
            app_name: frame.app_name.clone(),
            organization: frame.organization.as_ref().map(|value| Organization {
                name: value.name.clone(),
                unit: value.unit.clone(),
                website: value.website.clone(),
            }),
            title_template: frame.title_template.clone(),
            logo: frame.logo.as_ref().map(|value| Logo {
                plugin_id: value.plugin_id.as_str().to_owned(),
                asset_id: value.asset_id.as_str().to_owned(),
                url: value.url.clone(),
                alt: value.alt.clone(),
            }),
            themes,
            active_theme: frame.active_theme.as_ref().map(|id| id.as_str().to_owned()),
            theme_family_locked: frame.theme_family_locked,
            mode_policy: match frame.mode_policy {
                ThemeModePolicy::UserOrSystem => 1,
                ThemeModePolicy::ForceSystem => 2,
                ThemeModePolicy::ForceLight => 3,
                ThemeModePolicy::ForceDark => 4,
            },
            mode_locked: frame.mode_locked,
            source_plugin_id: frame
                .source_plugin_id
                .as_ref()
                .map(|id| id.as_str().to_owned()),
        })
    }
}

impl TryFrom<Appearance> for RuntimeAppearanceFrame {
    type Error = Error;
    fn try_from(frame: Appearance) -> Result<Self> {
        if frame.projection_memory()? == usize::MAX {
            return Err(invalid("appearance projection memory overflows"));
        }
        let mut themes = slots(frame.themes.len())?;
        for theme in frame.themes {
            themes.push(theme.try_into()?);
        }
        let result = Self {
            app_name: frame.app_name,
            organization: frame.organization.map(|value| OrganizationIdentity {
                name: value.name,
                unit: value.unit,
                website: value.website,
            }),
            title_template: frame.title_template,
            logo: frame
                .logo
                .map(|value| -> Result<RuntimeAppLogo> {
                    Ok(RuntimeAppLogo {
                        plugin_id: Identifier::new(value.plugin_id)
                            .map_err(|_| invalid("logo artifact identifier is invalid"))?,
                        asset_id: Identifier::new(value.asset_id)
                            .map_err(|_| invalid("logo asset identifier is invalid"))?,
                        url: value.url,
                        alt: value.alt,
                    })
                })
                .transpose()?,
            themes,
            active_theme: frame
                .active_theme
                .map(Identifier::new)
                .transpose()
                .map_err(|_| invalid("active theme identifier is invalid"))?,
            theme_family_locked: frame.theme_family_locked,
            mode_policy: mode(frame.mode_policy)?,
            mode_locked: frame.mode_locked,
            source_plugin_id: frame
                .source_plugin_id
                .map(Identifier::new)
                .transpose()
                .map_err(|_| invalid("appearance source identifier is invalid"))?,
        };
        crate::execution::context::validate_appearance(&result)?;
        Ok(result)
    }
}

impl Theme {
    fn projection_memory(&self) -> Result<usize> {
        let radii = self
            .radii
            .as_ref()
            .ok_or_else(|| invalid("theme radii are missing"))?;
        let light = self
            .light
            .as_ref()
            .ok_or_else(|| invalid("light theme is missing"))?;
        let dark = self
            .dark
            .as_ref()
            .ok_or_else(|| invalid("dark theme is missing"))?;
        let mut bytes = std::mem::size_of::<RuntimeThemeFamily>()
            .saturating_add(self.plugin_id.capacity())
            .saturating_add(self.theme_id.capacity())
            .saturating_add(self.label.capacity())
            .saturating_add(optional_capacity(&self.description))
            .saturating_add(optional_capacity(&self.stylesheet))
            .saturating_add(property_projection(&self.variables)?);
        for radius in radii.values().into_iter().flatten() {
            bytes = bytes.saturating_add(radius.capacity());
        }
        for variant in [light, dark] {
            bytes = bytes
                .saturating_add(property_projection(&variant.variables)?)
                .saturating_add(
                    variant
                        .plot_palette
                        .capacity()
                        .saturating_mul(std::mem::size_of::<String>()),
                );
            for color in &variant.plot_palette {
                bytes = bytes.saturating_add(color.capacity());
            }
        }
        Ok(bytes)
    }

    fn css_values(&self) -> impl Iterator<Item = &str> {
        self.stylesheet
            .iter()
            .map(String::as_str)
            .chain(
                self.variables
                    .iter()
                    .map(|property| property.value.as_str()),
            )
            .chain(self.light.iter().chain(&self.dark).flat_map(|variant| {
                variant
                    .variables
                    .iter()
                    .map(|property| property.value.as_str())
                    .chain(variant.plot_palette.iter().map(String::as_str))
            }))
            .chain(
                self.radii
                    .iter()
                    .flat_map(|radii| radii.values().into_iter().flatten().map(String::as_str)),
            )
    }
}

impl TryFrom<&RuntimeThemeFamily> for Theme {
    type Error = Error;
    fn try_from(theme: &RuntimeThemeFamily) -> Result<Self> {
        Ok(Self {
            plugin_id: theme.plugin_id.as_str().to_owned(),
            theme_id: theme.theme_id.as_str().to_owned(),
            label: theme.label.clone(),
            description: theme.description.clone(),
            stylesheet: theme.stylesheet.clone(),
            variables: properties(&theme.variables)?,
            radii: Some(Radii::from(&theme.corner_radii)),
            light: Some(Variant::try_from(&theme.light)?),
            dark: Some(Variant::try_from(&theme.dark)?),
        })
    }
}
impl TryFrom<Theme> for RuntimeThemeFamily {
    type Error = Error;
    fn try_from(theme: Theme) -> Result<Self> {
        Ok(Self {
            plugin_id: Identifier::new(theme.plugin_id)
                .map_err(|_| invalid("theme artifact identifier is invalid"))?,
            theme_id: Identifier::new(theme.theme_id)
                .map_err(|_| invalid("theme identifier is invalid"))?,
            label: theme.label,
            description: theme.description,
            stylesheet: theme.stylesheet,
            variables: property_map(theme.variables)?,
            corner_radii: theme
                .radii
                .ok_or_else(|| invalid("theme radii are missing"))?
                .into(),
            light: theme
                .light
                .ok_or_else(|| invalid("light theme is missing"))?
                .try_into()?,
            dark: theme
                .dark
                .ok_or_else(|| invalid("dark theme is missing"))?
                .try_into()?,
        })
    }
}
impl TryFrom<&ThemeVariant> for Variant {
    type Error = Error;
    fn try_from(variant: &ThemeVariant) -> Result<Self> {
        let mut palette = slots(variant.plot_palette.len())?;
        palette.extend(variant.plot_palette.iter().cloned());
        Ok(Self {
            variables: properties(&variant.variables)?,
            plot_palette: palette,
        })
    }
}
impl TryFrom<Variant> for ThemeVariant {
    type Error = Error;
    fn try_from(variant: Variant) -> Result<Self> {
        Ok(Self {
            variables: property_map(variant.variables)?,
            plot_palette: variant.plot_palette,
        })
    }
}

impl Radii {
    fn values(&self) -> [Option<&String>; 8] {
        [
            self.base.as_ref(),
            self.sm.as_ref(),
            self.md.as_ref(),
            self.lg.as_ref(),
            self.xl.as_ref(),
            self.two_xl.as_ref(),
            self.three_xl.as_ref(),
            self.four_xl.as_ref(),
        ]
    }
}
impl From<&ThemeCornerRadii> for Radii {
    fn from(value: &ThemeCornerRadii) -> Self {
        Self {
            base: value.base.clone(),
            sm: value.sm.clone(),
            md: value.md.clone(),
            lg: value.lg.clone(),
            xl: value.xl.clone(),
            two_xl: value.two_xl.clone(),
            three_xl: value.three_xl.clone(),
            four_xl: value.four_xl.clone(),
        }
    }
}
impl From<Radii> for ThemeCornerRadii {
    fn from(value: Radii) -> Self {
        Self {
            base: value.base,
            sm: value.sm,
            md: value.md,
            lg: value.lg,
            xl: value.xl,
            two_xl: value.two_xl,
            three_xl: value.three_xl,
            four_xl: value.four_xl,
        }
    }
}
fn domain_radii(value: &ThemeCornerRadii) -> [Option<&String>; 8] {
    [
        value.base.as_ref(),
        value.sm.as_ref(),
        value.md.as_ref(),
        value.lg.as_ref(),
        value.xl.as_ref(),
        value.two_xl.as_ref(),
        value.three_xl.as_ref(),
        value.four_xl.as_ref(),
    ]
}

/* === Public Functions === */

/* === Private Functions === */
fn slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| invalid("appearance slot allocation failed"))?;
    if values.capacity() > count {
        return Err(invalid("appearance slots exceed admission"));
    }
    Ok(values)
}
fn optional_length(value: &Option<String>) -> usize {
    value.as_ref().map_or(0, String::len)
}
fn optional_capacity(value: &Option<String>) -> usize {
    value.as_ref().map_or(0, String::capacity)
}
fn property_encoding(values: &BTreeMap<String, String>) -> usize {
    values.iter().fold(
        values.len().saturating_mul(std::mem::size_of::<Property>()),
        |bytes, (key, value)| bytes.saturating_add(key.len()).saturating_add(value.len()),
    )
}
fn property_projection(values: &[Property]) -> Result<usize> {
    if values.windows(2).any(|pair| pair[0].key >= pair[1].key) {
        return Err(invalid("theme properties are not canonical"));
    }
    let nodes = harmos::btree_map_memory::<String, String>(values.len());
    if nodes == usize::MAX {
        return Ok(usize::MAX);
    }
    Ok(values.iter().fold(
        nodes.saturating_sub(std::mem::size_of::<BTreeMap<String, String>>()),
        |bytes, property| {
            bytes
                .saturating_add(property.key.capacity())
                .saturating_add(property.value.capacity())
        },
    ))
}
fn properties(values: &BTreeMap<String, String>) -> Result<Vec<Property>> {
    let mut result = slots(values.len())?;
    result.extend(values.iter().map(|(key, value)| Property {
        key: key.clone(),
        value: value.clone(),
    }));
    Ok(result)
}
fn property_map(values: Vec<Property>) -> Result<BTreeMap<String, String>> {
    if property_projection(&values)? == usize::MAX {
        return Err(invalid("theme property projection overflows"));
    }
    let mut result = BTreeMap::new();
    for property in values {
        result.insert(property.key, property.value);
    }
    Ok(result)
}
fn mode(value: u32) -> Result<ThemeModePolicy> {
    match value {
        1 => Ok(ThemeModePolicy::UserOrSystem),
        2 => Ok(ThemeModePolicy::ForceSystem),
        3 => Ok(ThemeModePolicy::ForceLight),
        4 => Ok(ThemeModePolicy::ForceDark),
        _ => Err(invalid("appearance mode policy is unknown")),
    }
}
fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */
