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
pub use crate::domains::{RuntimeAppLogo, RuntimeAppearanceFrame, RuntimeThemeFamily};

/* === Imports === */
// Standard Library Imports
use std::{collections::BTreeMap, path::PathBuf};
// Third Party Imports
// Local Imports
use crate::RuntimeHandle;
use crate::domains::{AppCustomizationContribution, AppLogoAsset, AppThemeContribution, ThemeActivationPolicy, ThemeModePolicy};
use crate::installed_plugins::brand::{BRAND_ARTIFACT_ID, ResolvedBrand};
use dokime_domain::model::identity::Identifier;

/* === Constants === */
const DEFAULT_APP_NAME: &str = "Dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeInstalledPluginAsset {
    pub path: PathBuf,
    pub mime_type: String,
}

pub struct CapturedAppearance {
    pub frame: RuntimeAppearanceFrame,
    pub logo: Option<RuntimeInstalledPluginAsset>,
}

#[derive(Debug, thiserror::Error)]
#[error("appearance exceeds its source budget")]
pub struct AppearanceLimitExceeded;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub fn set_brand(&self, brand: Option<ResolvedBrand>) {
        *self.brand.write().expect("brand lock is healthy") = brand;
    }

    pub async fn appearance_frame(&self) -> RuntimeAppearanceFrame {
        let state = self.state_read();
        let brand = self.brand.read().expect("brand lock is healthy");
        captured_appearance(&state, brand.as_ref())
    }

    pub async fn plugin_appearance_asset(&self, plugin_id: &Identifier, asset_id: &Identifier) -> Option<RuntimeInstalledPluginAsset> {
        let state = self.state_read();
        let brand = self.brand.read().expect("brand lock is healthy");
        appearance_asset(&state, brand.as_ref(), plugin_id, asset_id)
    }

    pub fn capture_appearance(&self, max_bytes: usize) -> Result<CapturedAppearance, AppearanceLimitExceeded> {
        let state = self.state_read();
        let brand = self.brand.read().expect("brand lock is healthy");
        capture_appearance_state(&state, brand.as_ref(), max_bytes)
    }
}

/* === Public Functions === */
pub(crate) fn construction_memory(state: &crate::RuntimeState, brand: Option<&ResolvedBrand>) -> usize {
    let mut bytes = std::mem::size_of::<CapturedAppearance>();
    let mut count = 0usize;
    for (plugin_id, theme) in appearance_themes(state, brand) {
        count = count.saturating_add(1);
        bytes = bytes.saturating_add(theme_heap(plugin_id, theme));
    }
    let slots = count.saturating_mul(std::mem::size_of::<RuntimeThemeFamily>());
    bytes = bytes.saturating_add(slots);
    // Stable sorting may allocate scratch up to the full typed slice length.
    bytes = bytes.saturating_add(slots);
    let Some((plugin_id, customization)) = selected_customization(state, brand) else {
        return bytes.saturating_add(DEFAULT_APP_NAME.len()).saturating_add("{app}".len());
    };
    bytes = bytes
        .saturating_add(plugin_id.len())
        .saturating_add(customization.display_name.as_deref().unwrap_or(DEFAULT_APP_NAME).len())
        .saturating_add(customization.title_template.as_deref().unwrap_or("{app}").len());
    if let Some(organization) = &customization.organization {
        bytes = bytes
            .saturating_add(organization.name.len())
            .saturating_add(optional_text(&organization.unit))
            .saturating_add(optional_text(&organization.website));
    }
    if let ThemeActivationPolicy::Default { theme } | ThemeActivationPolicy::Required { theme, .. } = &customization.theme_policy
        && appearance_themes(state, brand).any(|(_, candidate)| &candidate.metadata.identifier == theme)
    {
        bytes = bytes.saturating_add(theme.as_str().len());
    }
    if let Some(asset) = customization.logo_assets.first() {
        bytes = bytes
            .saturating_add(plugin_id.len())
            .saturating_add(asset.asset_id.as_str().len())
            .saturating_add(logo_url_length(plugin_id, asset))
            .saturating_add(optional_text(&asset.alt))
            .saturating_add(asset.path.len())
            .saturating_add(asset.mime_type.len());
    }
    bytes
}

pub(crate) fn preflight_appearance(
    state: &crate::RuntimeState, brand: Option<&ResolvedBrand>, max_bytes: usize,
) -> Result<usize, AppearanceLimitExceeded> {
    let bytes = construction_memory(state, brand);
    if bytes == usize::MAX || bytes > max_bytes {
        return Err(AppearanceLimitExceeded);
    }
    Ok(bytes)
}

pub(crate) fn captured_appearance(state: &crate::RuntimeState, brand: Option<&ResolvedBrand>) -> RuntimeAppearanceFrame {
    let count = appearance_themes(state, brand).count();
    let mut themes = Vec::with_capacity(count);
    for (plugin_id, theme) in appearance_themes(state, brand) {
        themes.push(runtime_theme(plugin_id, theme));
    }
    sort_themes(&mut themes);
    appearance_frame(themes, selected_customization(state, brand))
}

pub(crate) fn captured_logo_asset(
    state: &crate::RuntimeState, brand: Option<&ResolvedBrand>, logo: &RuntimeAppLogo,
) -> Option<RuntimeInstalledPluginAsset> {
    let (plugin_id, customization) = selected_customization(state, brand)?;
    let asset = customization.logo_assets.first()?;
    if plugin_id != logo.plugin_id.as_str() || asset.asset_id != logo.asset_id {
        return None;
    }
    Some(RuntimeInstalledPluginAsset { path: PathBuf::from(&asset.path), mime_type: asset.mime_type.clone() })
}

/* === Private Functions === */
pub(crate) fn capture_appearance_state(
    state: &crate::RuntimeState, brand: Option<&ResolvedBrand>, max_bytes: usize,
) -> Result<CapturedAppearance, AppearanceLimitExceeded> {
    preflight_appearance(state, brand, max_bytes)?;
    let frame = captured_appearance(state, brand);
    let logo = frame.logo.as_ref().and_then(|logo| captured_logo_asset(state, brand, logo));
    Ok(CapturedAppearance { frame, logo })
}

fn appearance_asset(
    state: &crate::RuntimeState, brand: Option<&ResolvedBrand>, plugin_id: &Identifier, asset_id: &Identifier,
) -> Option<RuntimeInstalledPluginAsset> {
    if plugin_id.as_str() == BRAND_ARTIFACT_ID {
        return brand.and_then(|brand| brand_appearance_asset(brand, asset_id));
    }
    let source = state.installed_plugins.get(plugin_id.as_str())?;
    if !source.enabled {
        return None;
    }
    source
        .app_customizations
        .iter()
        .flat_map(|customization| customization.logo_assets.iter())
        .find(|asset| &asset.asset_id == asset_id)
        .map(|asset| RuntimeInstalledPluginAsset { path: PathBuf::from(&asset.path), mime_type: asset.mime_type.clone() })
}

fn appearance_frame(themes: Vec<RuntimeThemeFamily>, selected: Option<(&str, &AppCustomizationContribution)>) -> RuntimeAppearanceFrame {
    let Some((plugin_id, customization)) = selected else {
        return neutral_frame(themes);
    };

    let (active_theme, theme_family_locked, mode_policy) = resolve_policy(&customization.theme_policy, &themes);
    let logo = customization.logo_assets.first().map(|asset| runtime_logo(plugin_id, asset));

    RuntimeAppearanceFrame {
        app_name: customization.display_name.clone().unwrap_or_else(|| DEFAULT_APP_NAME.to_owned()),
        organization: customization.organization.clone(),
        title_template: customization.title_template.clone().or_else(|| Some("{app}".to_owned())),
        logo,
        themes,
        active_theme,
        theme_family_locked,
        mode_locked: !matches!(mode_policy, ThemeModePolicy::UserOrSystem),
        mode_policy,
        source_plugin_id: Some(Identifier::new(plugin_id).expect("installed source is valid")),
    }
}

fn selected_customization<'a>(
    state: &'a crate::RuntimeState, brand: Option<&'a ResolvedBrand>,
) -> Option<(&'a str, &'a AppCustomizationContribution)> {
    if let Some(brand) = brand {
        return Some((BRAND_ARTIFACT_ID, &brand.customization));
    }
    let mut selected: Option<(&str, &AppCustomizationContribution)> = None;
    for source in state.installed_plugins.values().filter(|source| source.enabled) {
        for customization in &source.app_customizations {
            if matches!(customization.theme_policy, ThemeActivationPolicy::RegisterOnly) {
                continue;
            }
            let candidate = (source.source.as_str(), customization);
            if selected.is_none_or(|current| customization_order(candidate, current).is_lt()) {
                selected = Some(candidate);
            }
        }
    }
    selected
}

fn customization_order(left: (&str, &AppCustomizationContribution), right: (&str, &AppCustomizationContribution)) -> std::cmp::Ordering {
    policy_rank(&left.1.theme_policy)
        .cmp(&policy_rank(&right.1.theme_policy))
        .then_with(|| left.0.cmp(right.0))
        .then_with(|| left.1.metadata.identifier.cmp(&right.1.metadata.identifier))
}

fn appearance_themes<'a>(
    state: &'a crate::RuntimeState, brand: Option<&'a ResolvedBrand>,
) -> impl Iterator<Item = (&'a str, &'a AppThemeContribution)> {
    state
        .installed_plugins
        .values()
        .filter(|source| source.enabled)
        .flat_map(|source| source.app_themes.iter().map(|theme| (source.source.as_str(), theme)))
        .chain(brand.and_then(|brand| brand.theme.as_ref()).map(|theme| (BRAND_ARTIFACT_ID, theme)))
}

fn neutral_frame(themes: Vec<RuntimeThemeFamily>) -> RuntimeAppearanceFrame {
    RuntimeAppearanceFrame {
        app_name: DEFAULT_APP_NAME.to_owned(),
        organization: None,
        title_template: Some("{app}".to_owned()),
        logo: None,
        themes,
        active_theme: None,
        theme_family_locked: false,
        mode_policy: ThemeModePolicy::UserOrSystem,
        mode_locked: false,
        source_plugin_id: None,
    }
}

fn brand_appearance_asset(brand: &ResolvedBrand, asset_id: &Identifier) -> Option<RuntimeInstalledPluginAsset> {
    brand
        .customization
        .logo_assets
        .iter()
        .find(|asset| &asset.asset_id == asset_id)
        .map(|asset| RuntimeInstalledPluginAsset { path: PathBuf::from(&asset.path), mime_type: asset.mime_type.clone() })
}

fn sort_themes(themes: &mut [RuntimeThemeFamily]) {
    themes.sort_by(|left, right| left.label.cmp(&right.label).then_with(|| left.theme_id.cmp(&right.theme_id)));
}

fn policy_rank(policy: &ThemeActivationPolicy) -> u8 {
    match policy {
        ThemeActivationPolicy::Required { .. } => 0,
        ThemeActivationPolicy::Default { .. } => 1,
        ThemeActivationPolicy::RegisterOnly => 2,
    }
}

fn resolve_policy(policy: &ThemeActivationPolicy, themes: &[RuntimeThemeFamily]) -> (Option<Identifier>, bool, ThemeModePolicy) {
    match policy {
        ThemeActivationPolicy::RegisterOnly => (None, false, ThemeModePolicy::UserOrSystem),
        ThemeActivationPolicy::Default { theme } => {
            (theme_exists(theme, themes).then(|| theme.clone()), false, ThemeModePolicy::UserOrSystem)
        }
        ThemeActivationPolicy::Required { theme, mode } => (theme_exists(theme, themes).then(|| theme.clone()), true, *mode),
    }
}

fn theme_exists(theme: &Identifier, themes: &[RuntimeThemeFamily]) -> bool {
    themes.iter().any(|candidate| &candidate.theme_id == theme)
}

fn runtime_theme(plugin_id: &str, theme: &AppThemeContribution) -> RuntimeThemeFamily {
    RuntimeThemeFamily {
        plugin_id: Identifier::new(plugin_id).expect("installed source is valid"),
        theme_id: theme.metadata.identifier.clone(),
        label: theme.metadata.label.clone(),
        description: theme.metadata.description.clone(),
        stylesheet: theme.stylesheet.clone(),
        variables: theme.variables.clone(),
        corner_radii: theme.corner_radii.clone(),
        light: theme.light.clone(),
        dark: theme.dark.clone(),
    }
}

fn runtime_logo(plugin_id: &str, asset: &AppLogoAsset) -> RuntimeAppLogo {
    let mut url = String::with_capacity(logo_url_length(plugin_id, asset));
    url.push_str("/api/installed-plugins/");
    url.push_str(plugin_id);
    url.push_str("/assets/");
    url.push_str(asset.asset_id.as_str());
    RuntimeAppLogo {
        plugin_id: Identifier::new(plugin_id).expect("installed source is valid"),
        asset_id: asset.asset_id.clone(),
        url,
        alt: asset.alt.clone(),
    }
}

fn logo_url_length(plugin_id: &str, asset: &AppLogoAsset) -> usize {
    "/api/installed-plugins/"
        .len()
        .saturating_add(plugin_id.len())
        .saturating_add("/assets/".len())
        .saturating_add(asset.asset_id.as_str().len())
}

fn optional_text(value: &Option<String>) -> usize {
    value.as_ref().map_or(0, String::len)
}

fn variable_heap(values: &BTreeMap<String, String>) -> usize {
    let nodes = harmos::btree_map_memory::<String, String>(values.len());
    if nodes == usize::MAX {
        return usize::MAX;
    }
    values.iter().fold(nodes.saturating_sub(std::mem::size_of_val(values)), |bytes, (key, value)| {
        bytes.saturating_add(key.len()).saturating_add(value.len())
    })
}

fn theme_heap(plugin_id: &str, theme: &AppThemeContribution) -> usize {
    let mut bytes = plugin_id
        .len()
        .saturating_add(theme.metadata.identifier.as_str().len())
        .saturating_add(theme.metadata.label.len())
        .saturating_add(optional_text(&theme.metadata.description))
        .saturating_add(optional_text(&theme.stylesheet))
        .saturating_add(variable_heap(&theme.variables));
    let radii = &theme.corner_radii;
    for radius in [&radii.base, &radii.sm, &radii.md, &radii.lg, &radii.xl, &radii.two_xl, &radii.three_xl, &radii.four_xl] {
        bytes = bytes.saturating_add(optional_text(radius));
    }
    for variant in [&theme.light, &theme.dark] {
        bytes = bytes
            .saturating_add(variable_heap(&variant.variables))
            .saturating_add(variant.plot_palette.len().saturating_mul(std::mem::size_of::<String>()));
        for color in &variant.plot_palette {
            bytes = bytes.saturating_add(color.len());
        }
    }
    bytes
}

/* === Tests === */
