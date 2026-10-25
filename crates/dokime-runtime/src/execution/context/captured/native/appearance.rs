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
use crate::domains::{OrganizationIdentity, ThemeCornerRadii, ThemeModePolicy, ThemeVariant};
use crate::installed_plugins::{RuntimeAppLogo, RuntimeAppearanceFrame, RuntimeThemeFamily};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::identity::Identifier;

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
    logos: Vec<Logo>,
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
    variant: dokime_domain::model::plugins::LogoVariant,
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

impl Appearance {}

impl TryFrom<&RuntimeAppearanceFrame> for Appearance {
    type Error = Error;
    fn try_from(frame: &RuntimeAppearanceFrame) -> Result<Self> {
        let mut themes = slots(frame.themes.len())?;
        for theme in &frame.themes {
            themes.push(Theme::try_from(theme)?);
        }
        let mut logos = slots(frame.logos.len())?;
        for value in &frame.logos {
            logos.push(Logo {
                variant: value.variant,
                plugin_id: value.plugin_id.as_str().to_owned(),
                asset_id: value.asset_id.as_str().to_owned(),
                url: value.url.clone(),
                alt: value.alt.clone(),
            });
        }
        Ok(Self {
            app_name: frame.app_name.clone(),
            organization: frame.organization.as_ref().map(|value| Organization {
                name: value.name.clone(),
                unit: value.unit.clone(),
                website: value.website.clone(),
            }),
            title_template: frame.title_template.clone(),
            logos,
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
            source_plugin_id: frame.source_plugin_id.as_ref().map(|id| id.as_str().to_owned()),
        })
    }
}

impl TryFrom<Appearance> for RuntimeAppearanceFrame {
    type Error = Error;
    fn try_from(frame: Appearance) -> Result<Self> {
        let mut themes = slots(frame.themes.len())?;
        for theme in frame.themes {
            themes.push(theme.try_into()?);
        }
        let mut logos = slots(frame.logos.len())?;
        for value in frame.logos {
            logos.push(RuntimeAppLogo {
                variant: value.variant,
                plugin_id: Identifier::new(value.plugin_id).map_err(|_| invalid("logo artifact identifier is invalid"))?,
                asset_id: Identifier::new(value.asset_id).map_err(|_| invalid("logo asset identifier is invalid"))?,
                url: value.url,
                alt: value.alt,
            });
        }
        let result = Self {
            app_name: frame.app_name,
            organization: frame.organization.map(|value| OrganizationIdentity {
                name: value.name,
                unit: value.unit,
                website: value.website,
            }),
            title_template: frame.title_template,
            logos,
            themes,
            active_theme: frame.active_theme.map(Identifier::new).transpose().map_err(|_| invalid("active theme identifier is invalid"))?,
            theme_family_locked: frame.theme_family_locked,
            mode_policy: mode(frame.mode_policy)?,
            mode_locked: frame.mode_locked,
            source_plugin_id: frame
                .source_plugin_id
                .map(Identifier::new)
                .transpose()
                .map_err(|_| invalid("appearance source identifier is invalid"))?,
        };
        crate::execution::context::captured::validate_appearance(&result)?;
        Ok(result)
    }
}

impl Theme {}

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
            plugin_id: Identifier::new(theme.plugin_id).map_err(|_| invalid("theme artifact identifier is invalid"))?,
            theme_id: Identifier::new(theme.theme_id).map_err(|_| invalid("theme identifier is invalid"))?,
            label: theme.label,
            description: theme.description,
            stylesheet: theme.stylesheet,
            variables: property_map(theme.variables)?,
            corner_radii: theme.radii.ok_or_else(|| invalid("theme radii are missing"))?.into(),
            light: theme.light.ok_or_else(|| invalid("light theme is missing"))?.try_into()?,
            dark: theme.dark.ok_or_else(|| invalid("dark theme is missing"))?.try_into()?,
        })
    }
}

impl TryFrom<&ThemeVariant> for Variant {
    type Error = Error;
    fn try_from(variant: &ThemeVariant) -> Result<Self> {
        let mut palette = slots(variant.plot_palette.len())?;
        palette.extend(variant.plot_palette.iter().cloned());
        Ok(Self { variables: properties(&variant.variables)?, plot_palette: palette })
    }
}

impl TryFrom<Variant> for ThemeVariant {
    type Error = Error;
    fn try_from(variant: Variant) -> Result<Self> {
        Ok(Self { variables: property_map(variant.variables)?, plot_palette: variant.plot_palette })
    }
}

impl Radii {}

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

/* === Public Functions === */

/* === Private Functions === */

fn slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| invalid("appearance slot allocation failed"))?;
    if values.capacity() > count {
        return Err(invalid("appearance slots exceed admission"));
    }
    Ok(values)
}

fn property_projection(values: &[Property]) -> Result<usize> {
    if values.windows(2).any(|pair| pair[0].key >= pair[1].key) {
        return Err(invalid("theme properties are not canonical"));
    }
    let nodes = harmos::btree_map_memory::<String, String>(values.len());
    if nodes == usize::MAX {
        return Ok(usize::MAX);
    }
    Ok(values.iter().fold(nodes.saturating_sub(std::mem::size_of::<BTreeMap<String, String>>()), |bytes, property| {
        bytes.saturating_add(property.key.capacity()).saturating_add(property.value.capacity())
    }))
}

fn properties(values: &BTreeMap<String, String>) -> Result<Vec<Property>> {
    let mut result = slots(values.len())?;
    result.extend(values.iter().map(|(key, value)| Property { key: key.clone(), value: value.clone() }));
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

#[cfg(test)]
mod tests {
    use super::*;
    use dokime_domain::model::plugins::LogoVariant;

    #[test]
    fn captured_logo_categories() {
        let variants =
            [LogoVariant::TransparentOnLight, LogoVariant::TransparentOnDark, LogoVariant::ContainedOnLight, LogoVariant::ContainedOnDark];
        let frame = RuntimeAppearanceFrame {
            app_name: "Stokker Technologies".into(),
            organization: None,
            title_template: None,
            logos: variants
                .into_iter()
                .enumerate()
                .map(|(index, variant)| RuntimeAppLogo {
                    variant,
                    plugin_id: Identifier::new("brand").unwrap(),
                    asset_id: Identifier::new(format!("logo.{index}")).unwrap(),
                    url: "data:image/svg+xml;base64,PHN2Zy8+".into(),
                    alt: Some("Stokker Technologies".into()),
                })
                .collect(),
            themes: Vec::new(),
            active_theme: None,
            theme_family_locked: false,
            mode_policy: ThemeModePolicy::UserOrSystem,
            mode_locked: false,
            source_plugin_id: None,
        };
        let captured = Appearance::try_from(&frame).unwrap();
        assert_eq!(RuntimeAppearanceFrame::try_from(captured).unwrap(), frame);
        let mut duplicate = frame.clone();
        duplicate.logos[1].variant = duplicate.logos[0].variant;
        assert!(RuntimeAppearanceFrame::try_from(Appearance::try_from(&duplicate).unwrap()).is_err());
        let mut external = frame;
        external.logos[3].url = "https://example.invalid/logo.svg".into();
        assert!(RuntimeAppearanceFrame::try_from(Appearance::try_from(&external).unwrap()).is_err());
    }
}
