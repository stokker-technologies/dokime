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
use serde::{Deserialize, Serialize};

// Local Imports
use super::{OrganizationIdentity, ThemeCornerRadii, ThemeModePolicy, ThemeVariant};
use crate::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeAppearanceFrame {
    pub app_name: String,
    #[serde(default)]
    pub organization: Option<OrganizationIdentity>,
    #[serde(default)]
    pub title_template: Option<String>,
    #[serde(default)]
    pub logo: Option<RuntimeAppLogo>,
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

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for RuntimeAppearanceFrame {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.app_name),
                std::mem::size_of_val(&self.app_name),
            ),
            (
                harmos::MemorySize::memory_size(&self.organization),
                std::mem::size_of_val(&self.organization),
            ),
            (
                harmos::MemorySize::memory_size(&self.title_template),
                std::mem::size_of_val(&self.title_template),
            ),
            (
                harmos::MemorySize::memory_size(&self.logo),
                std::mem::size_of_val(&self.logo),
            ),
            (
                harmos::MemorySize::memory_size(&self.themes),
                std::mem::size_of_val(&self.themes),
            ),
            (
                harmos::MemorySize::memory_size(&self.active_theme),
                std::mem::size_of_val(&self.active_theme),
            ),
            (
                harmos::MemorySize::memory_size(&self.source_plugin_id),
                std::mem::size_of_val(&self.source_plugin_id),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for RuntimeAppLogo {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.plugin_id),
                std::mem::size_of_val(&self.plugin_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.asset_id),
                std::mem::size_of_val(&self.asset_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.url),
                std::mem::size_of_val(&self.url),
            ),
            (
                harmos::MemorySize::memory_size(&self.alt),
                std::mem::size_of_val(&self.alt),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

impl harmos::MemorySize for RuntimeThemeFamily {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.plugin_id),
                std::mem::size_of_val(&self.plugin_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.theme_id),
                std::mem::size_of_val(&self.theme_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.label),
                std::mem::size_of_val(&self.label),
            ),
            (
                harmos::MemorySize::memory_size(&self.description),
                std::mem::size_of_val(&self.description),
            ),
            (
                harmos::MemorySize::memory_size(&self.stylesheet),
                std::mem::size_of_val(&self.stylesheet),
            ),
            (
                harmos::MemorySize::memory_size(&self.variables),
                std::mem::size_of_val(&self.variables),
            ),
            (
                harmos::MemorySize::memory_size(&self.corner_radii),
                std::mem::size_of_val(&self.corner_radii),
            ),
            (
                harmos::MemorySize::memory_size(&self.light),
                std::mem::size_of_val(&self.light),
            ),
            (
                harmos::MemorySize::memory_size(&self.dark),
                std::mem::size_of_val(&self.dark),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
