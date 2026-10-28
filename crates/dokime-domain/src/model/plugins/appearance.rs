/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use super::OrganizationIdentity;
use super::ThemeCornerRadii;
use super::ThemeModePolicy;
use super::ThemeVariant;
use crate::model::identity::Identifier;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeAppearanceFrame {
    pub app_name: String,

    #[serde(default)]
    pub organization: Option<OrganizationIdentity>,

    #[serde(default)]
    pub title_template: Option<String>,

    #[serde(default)]
    pub logos: Vec<RuntimeAppLogo>,

    #[serde(default)]
    pub themes: Vec<RuntimeThemeFamily>,

    #[serde(default)]
    pub active_theme: Option<Identifier>,

    pub theme_family_locked: bool,
    pub mode_policy: ThemeModePolicy,
    pub mode_locked: bool,

    #[serde(default)]
    pub source_plugin_id: Option<Identifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeAppLogo {
    pub variant: super::LogoVariant,
    pub plugin_id: Identifier,
    pub asset_id: Identifier,
    pub url: String,

    #[serde(default)]
    pub alt: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeThemeFamily {
    pub plugin_id: Identifier,
    pub theme_id: Identifier,
    pub label: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub stylesheet: Option<String>,

    #[serde(default)]
    pub variables: BTreeMap<String, String>,

    #[serde(default)]
    pub corner_radii: ThemeCornerRadii,

    pub light: ThemeVariant,
    pub dark: ThemeVariant,
}

impl harmos_domain::MemorySize for RuntimeAppearanceFrame {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.app_name), std::mem::size_of_val(&self.app_name)),
            (harmos_domain::MemorySize::memory_size(&self.organization), std::mem::size_of_val(&self.organization)),
            (harmos_domain::MemorySize::memory_size(&self.title_template), std::mem::size_of_val(&self.title_template)),
            (harmos_domain::MemorySize::memory_size(&self.logos), std::mem::size_of_val(&self.logos)),
            (harmos_domain::MemorySize::memory_size(&self.themes), std::mem::size_of_val(&self.themes)),
            (harmos_domain::MemorySize::memory_size(&self.active_theme), std::mem::size_of_val(&self.active_theme)),
            (harmos_domain::MemorySize::memory_size(&self.source_plugin_id), std::mem::size_of_val(&self.source_plugin_id)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for RuntimeAppLogo {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.plugin_id), std::mem::size_of_val(&self.plugin_id)),
            (harmos_domain::MemorySize::memory_size(&self.asset_id), std::mem::size_of_val(&self.asset_id)),
            (harmos_domain::MemorySize::memory_size(&self.url), std::mem::size_of_val(&self.url)),
            (harmos_domain::MemorySize::memory_size(&self.alt), std::mem::size_of_val(&self.alt)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for RuntimeThemeFamily {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.plugin_id), std::mem::size_of_val(&self.plugin_id)),
            (harmos_domain::MemorySize::memory_size(&self.theme_id), std::mem::size_of_val(&self.theme_id)),
            (harmos_domain::MemorySize::memory_size(&self.label), std::mem::size_of_val(&self.label)),
            (harmos_domain::MemorySize::memory_size(&self.description), std::mem::size_of_val(&self.description)),
            (harmos_domain::MemorySize::memory_size(&self.stylesheet), std::mem::size_of_val(&self.stylesheet)),
            (harmos_domain::MemorySize::memory_size(&self.variables), std::mem::size_of_val(&self.variables)),
            (harmos_domain::MemorySize::memory_size(&self.corner_radii), std::mem::size_of_val(&self.corner_radii)),
            (harmos_domain::MemorySize::memory_size(&self.light), std::mem::size_of_val(&self.light)),
            (harmos_domain::MemorySize::memory_size(&self.dark), std::mem::size_of_val(&self.dark)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}
