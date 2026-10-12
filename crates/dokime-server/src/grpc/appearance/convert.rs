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
use crate::grpc::value::projection;
use dokime_app::capabilities::installed_plugins::{RuntimeAppearanceFrame, RuntimeThemeFamily};
use dokime_app::domains::{ThemeCornerRadii, ThemeModePolicy, ThemeVariant};
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn appearance(
    frame: RuntimeAppearanceFrame,
    logo: Option<v1::AppearanceLogo>,
) -> harmos::Result<v1::AppearanceSnapshot> {
    let mut themes = projection::slots(frame.themes.len())?;
    for source in frame.themes {
        themes.push(theme(source));
    }
    Ok(v1::AppearanceSnapshot {
        app_name: frame.app_name,
        organization: frame.organization.map(|value| v1::OrganizationIdentity {
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
            ThemeModePolicy::UserOrSystem => v1::ThemeModePolicy::UserOrSystem,
            ThemeModePolicy::ForceSystem => v1::ThemeModePolicy::ForceSystem,
            ThemeModePolicy::ForceLight => v1::ThemeModePolicy::ForceLight,
            ThemeModePolicy::ForceDark => v1::ThemeModePolicy::ForceDark,
        } as i32,
        mode_locked: frame.mode_locked,
        source_plugin_id: frame.source_plugin_id.map(|id| id.as_str().to_owned()),
    })
}

pub(in crate::grpc) fn historical_memory(source: &RuntimeAppearanceFrame) -> harmos::Result<usize> {
    let mut bytes = std::mem::size_of::<v1::AppearanceSnapshot>()
        .saturating_add(source.app_name.capacity())
        .saturating_add(heap(&source.organization)?)
        .saturating_add(heap(&source.title_template)?)
        .saturating_add(
            source
                .active_theme
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
        )
        .saturating_add(
            source
                .source_plugin_id
                .as_ref()
                .map_or(0, |id| id.as_str().len()),
        )
        .saturating_add(
            source
                .themes
                .len()
                .saturating_mul(std::mem::size_of::<v1::ThemeFamily>()),
        );
    for theme in &source.themes {
        bytes = bytes
            .saturating_add(theme.plugin_id.as_str().len())
            .saturating_add(theme.theme_id.as_str().len())
            .saturating_add(theme.label.capacity())
            .saturating_add(heap(&theme.description)?)
            .saturating_add(heap(&theme.stylesheet)?)
            .saturating_add(heap(&theme.variables)?)
            .saturating_add(heap(&theme.corner_radii)?)
            .saturating_add(heap(&theme.light)?)
            .saturating_add(heap(&theme.dark)?);
    }
    if bytes == usize::MAX {
        return Err(exhausted());
    }
    Ok(bytes)
}

/* === Private Functions === */
fn theme(value: RuntimeThemeFamily) -> v1::ThemeFamily {
    v1::ThemeFamily {
        plugin_id: value.plugin_id.as_str().to_owned(),
        theme_id: value.theme_id.as_str().to_owned(),
        label: value.label,
        description: value.description,
        stylesheet: value.stylesheet,
        variables: value.variables,
        corner_radii: Some(radii(value.corner_radii)),
        light: Some(variant(value.light)),
        dark: Some(variant(value.dark)),
    }
}

fn radii(value: ThemeCornerRadii) -> v1::ThemeCornerRadii {
    v1::ThemeCornerRadii {
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

fn variant(value: ThemeVariant) -> v1::ThemeVariant {
    v1::ThemeVariant {
        variables: value.variables,
        plot_palette: value.plot_palette,
    }
}

fn heap<T: harmos::MemorySize>(source: &T) -> harmos::Result<usize> {
    let bytes = source.memory_size();
    if bytes == usize::MAX {
        return Err(exhausted());
    }
    Ok(bytes.saturating_sub(std::mem::size_of::<T>()))
}

fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "historical appearance construction",
    }
}

/* === Tests === */
