/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use crate::grpc::value::canonical;
use dokime::domains::installed_plugins::{RuntimeAppearanceFrame, RuntimeThemeFamily};
use dokime::domains::{ThemeCornerRadii, ThemeModePolicy, ThemeVariant};

pub(super) fn appearance(
    frame: RuntimeAppearanceFrame, logos: Vec<dokime::domain::model::appearance::wire::AppearanceLogo>,
) -> dokime::harmos::Result<dokime::domain::model::appearance::wire::AppearanceSnapshot> {
    let mut themes = canonical::slots(frame.themes.len()).map_err(|_| exhausted())?;
    for source in frame.themes {
        themes.push(theme(source)?);
    }
    Ok(dokime::domain::model::appearance::wire::AppearanceSnapshot {
        app_name: frame.app_name,
        organization: frame.organization.map(|value| dokime::domain::model::appearance::wire::OrganizationIdentity {
            name: value.name,
            unit: value.unit,
            website: value.website,
        }),
        title_template: frame.title_template,
        logos,
        themes,
        active_theme: frame.active_theme.map(|id| id.as_str().to_owned()),
        theme_family_locked: frame.theme_family_locked,
        mode_policy: match frame.mode_policy {
            ThemeModePolicy::UserOrSystem => dokime::domain::model::appearance::wire::ThemeModePolicy::UserOrSystem,
            ThemeModePolicy::ForceSystem => dokime::domain::model::appearance::wire::ThemeModePolicy::ForceSystem,
            ThemeModePolicy::ForceLight => dokime::domain::model::appearance::wire::ThemeModePolicy::ForceLight,
            ThemeModePolicy::ForceDark => dokime::domain::model::appearance::wire::ThemeModePolicy::ForceDark,
        },
        mode_locked: frame.mode_locked,
        source_plugin_id: frame.source_plugin_id.map(|id| id.as_str().to_owned()),
    })
}

fn theme(value: RuntimeThemeFamily) -> dokime::harmos::Result<dokime::domain::model::appearance::wire::ThemeFamily> {
    let mut variables = Vec::new();
    variables.try_reserve_exact(value.variables.len()).map_err(|_| exhausted())?;
    for (key, value) in value.variables {
        variables.push(dokime::domain::model::appearance::wire::ThemeFamilyVariablesEntry { key, value });
    }
    Ok(dokime::domain::model::appearance::wire::ThemeFamily {
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

fn radii(value: ThemeCornerRadii) -> dokime::domain::model::appearance::wire::ThemeCornerRadii {
    dokime::domain::model::appearance::wire::ThemeCornerRadii {
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

fn variant(value: ThemeVariant) -> dokime::harmos::Result<dokime::domain::model::appearance::wire::ThemeVariant> {
    let mut variables = Vec::new();
    variables.try_reserve_exact(value.variables.len()).map_err(|_| exhausted())?;
    for (key, value) in value.variables {
        variables.push(dokime::domain::model::appearance::wire::ThemeVariantVariablesEntry { key, value });
    }
    Ok(dokime::domain::model::appearance::wire::ThemeVariant {
        variables,
        plot_palette: value.plot_palette,
    })
}

fn exhausted() -> dokime::harmos::Error {
    dokime::harmos::Error::ResourceExhausted {
        resource: "historical appearance construction",
    }
}
