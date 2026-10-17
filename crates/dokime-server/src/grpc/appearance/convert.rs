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

// Third Party Imports

// Local Imports
use crate::grpc::value::canonical;
use dokime_domain::operations::appearance as wire;
use dokime_runtime::domains::installed_plugins::{RuntimeAppearanceFrame, RuntimeThemeFamily};
use dokime_runtime::domains::{ThemeCornerRadii, ThemeModePolicy, ThemeVariant};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn appearance(
    frame: RuntimeAppearanceFrame, logo: Option<dokime_domain::model::appearance::wire::AppearanceLogo>,
) -> harmos::Result<dokime_domain::model::appearance::wire::AppearanceSnapshot> {
    let mut themes = canonical::slots(frame.themes.len()).map_err(|_| exhausted())?;
    for source in frame.themes {
        themes.push(theme(source)?);
    }
    Ok(dokime_domain::model::appearance::wire::AppearanceSnapshot {
        app_name: frame.app_name,
        organization: frame.organization.map(|value| dokime_domain::model::appearance::wire::OrganizationIdentity {
            name: value.name,
            unit: value.unit,
            website: value.website,
        }),
        title_template: frame.title_template,
        logo,
        themes,
        active_theme: frame.active_theme.map(|id| id.as_str().to_owned()),
        theme_family_locked: frame.theme_family_locked,
        mode_policy: match frame.mode_policy {
            ThemeModePolicy::UserOrSystem => dokime_domain::model::appearance::wire::ThemeModePolicy::UserOrSystem,
            ThemeModePolicy::ForceSystem => dokime_domain::model::appearance::wire::ThemeModePolicy::ForceSystem,
            ThemeModePolicy::ForceLight => dokime_domain::model::appearance::wire::ThemeModePolicy::ForceLight,
            ThemeModePolicy::ForceDark => dokime_domain::model::appearance::wire::ThemeModePolicy::ForceDark,
        },
        mode_locked: frame.mode_locked,
        source_plugin_id: frame.source_plugin_id.map(|id| id.as_str().to_owned()),
    })
}

pub(in crate::grpc) fn historical_memory(source: &RuntimeAppearanceFrame) -> harmos::Result<usize> {
    let mut bytes = std::mem::size_of::<dokime_domain::model::appearance::wire::AppearanceSnapshot>()
        .saturating_add(source.app_name.capacity())
        .saturating_add(heap(&source.organization)?)
        .saturating_add(heap(&source.title_template)?)
        .saturating_add(source.active_theme.as_ref().map_or(0, |id| id.as_str().len()))
        .saturating_add(source.source_plugin_id.as_ref().map_or(0, |id| id.as_str().len()))
        .saturating_add(source.themes.len().saturating_mul(std::mem::size_of::<dokime_domain::model::appearance::wire::ThemeFamily>()));
    for theme in &source.themes {
        bytes = bytes
            .saturating_add(theme.plugin_id.as_str().len())
            .saturating_add(theme.theme_id.as_str().len())
            .saturating_add(theme.label.capacity())
            .saturating_add(heap(&theme.description)?)
            .saturating_add(heap(&theme.stylesheet)?)
            .saturating_add(variables_heap(
                &theme.variables,
                std::mem::size_of::<dokime_domain::model::appearance::wire::ThemeFamilyVariablesEntry>(),
            )?)
            .saturating_add(heap(&theme.corner_radii)?)
            .saturating_add(variant_heap(&theme.light)?)
            .saturating_add(variant_heap(&theme.dark)?);
    }
    if bytes == usize::MAX {
        return Err(exhausted());
    }
    Ok(bytes)
}

/* === Private Functions === */
fn theme(value: RuntimeThemeFamily) -> harmos::Result<dokime_domain::model::appearance::wire::ThemeFamily> {
    let mut variables = Vec::new();
    variables.try_reserve_exact(value.variables.len()).map_err(|_| exhausted())?;
    for (key, value) in value.variables {
        variables.push(dokime_domain::model::appearance::wire::ThemeFamilyVariablesEntry { key, value });
    }
    Ok(dokime_domain::model::appearance::wire::ThemeFamily {
        plugin_id: value.plugin_id.as_str().to_owned(),
        theme_id: value.theme_id.as_str().to_owned(),
        label: value.label,
        description: value.description,
        stylesheet: value.stylesheet,
        variables,
        corner_radii: Some(radii(value.corner_radii)),
        light: Some(variant(value.light)?),
        dark: Some(variant(value.dark)?),
    })
}

fn radii(value: ThemeCornerRadii) -> dokime_domain::model::appearance::wire::ThemeCornerRadii {
    dokime_domain::model::appearance::wire::ThemeCornerRadii {
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

fn variant(value: ThemeVariant) -> harmos::Result<dokime_domain::model::appearance::wire::ThemeVariant> {
    let mut variables = Vec::new();
    variables.try_reserve_exact(value.variables.len()).map_err(|_| exhausted())?;
    for (key, value) in value.variables {
        variables.push(dokime_domain::model::appearance::wire::ThemeVariantVariablesEntry { key, value });
    }
    Ok(dokime_domain::model::appearance::wire::ThemeVariant { variables, plot_palette: value.plot_palette })
}

fn variables_heap(source: &std::collections::BTreeMap<String, String>, slot: usize) -> harmos::Result<usize> {
    let mut bytes = source.len().checked_mul(slot).ok_or_else(exhausted)?;
    for (key, value) in source {
        bytes = bytes.checked_add(key.capacity()).and_then(|bytes| bytes.checked_add(value.capacity())).ok_or_else(exhausted)?;
    }
    Ok(bytes)
}

fn variant_heap(source: &ThemeVariant) -> harmos::Result<usize> {
    variables_heap(&source.variables, std::mem::size_of::<dokime_domain::model::appearance::wire::ThemeVariantVariablesEntry>())?
        .checked_add(heap(&source.plot_palette)?)
        .ok_or_else(exhausted)
}

fn heap<T: harmos::MemorySize>(source: &T) -> harmos::Result<usize> {
    let bytes = source.memory_size();
    if bytes == usize::MAX {
        return Err(exhausted());
    }
    Ok(bytes.saturating_sub(std::mem::size_of::<T>()))
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted { resource: "historical appearance construction" }
}

/* === Tests === */
