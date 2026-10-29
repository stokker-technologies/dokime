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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppCustomizationContribution {
    pub metadata: Metadata,

    #[serde(default)]
    pub organization: Option<OrganizationIdentity>,

    #[serde(default)]
    pub display_name: Option<String>,

    #[serde(default)]
    pub title_template: Option<String>,

    #[serde(default)]
    pub logo_assets: Vec<AppLogoAsset>,

    #[serde(default)]
    pub theme_policy: ThemeActivationPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationIdentity {
    pub name: String,

    #[serde(default)]
    pub unit: Option<String>,

    #[serde(default)]
    pub website: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppLogoAsset {
    pub variant: LogoVariant,
    pub asset_id: Identifier,
    pub path: String,
    pub mime_type: String,

    #[serde(default)]
    pub alt: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppThemeContribution {
    pub metadata: Metadata,

    #[serde(default)]
    pub stylesheet: Option<String>,

    #[serde(default)]
    pub variables: BTreeMap<String, String>,

    #[serde(default)]
    pub corner_radii: ThemeCornerRadii,

    pub light: ThemeVariant,
    pub dark: ThemeVariant,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeCornerRadii {
    #[serde(default)]
    pub base: Option<String>,

    #[serde(default)]
    pub sm: Option<String>,

    #[serde(default)]
    pub md: Option<String>,

    #[serde(default)]
    pub lg: Option<String>,

    #[serde(default)]
    pub xl: Option<String>,

    #[serde(default, rename = "2xl")]
    pub two_xl: Option<String>,

    #[serde(default, rename = "3xl")]
    pub three_xl: Option<String>,

    #[serde(default, rename = "4xl")]
    pub four_xl: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeVariant {
    #[serde(default)]
    pub variables: BTreeMap<String, String>,

    #[serde(default)]
    pub plot_palette: Vec<String>,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogoVariant {
    TransparentOnLight,
    TransparentOnDark,
    ContainedOnLight,
    ContainedOnDark,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ThemeActivationPolicy {
    #[default]
    RegisterOnly,

    Default {
        theme: Identifier,
    },
    Required {
        theme: Identifier,
        mode: ThemeModePolicy,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeModePolicy {
    #[default]
    UserOrSystem,

    ForceSystem,
    ForceLight,
    ForceDark,
}

/* === Traits === */

/* === Implementations === */

impl harmos::MemorySize for OrganizationIdentity {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.name), std::mem::size_of_val(&self.name)),
            (harmos::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
            (harmos::MemorySize::memory_size(&self.website), std::mem::size_of_val(&self.website)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for ThemeCornerRadii {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.base), std::mem::size_of_val(&self.base)),
            (harmos::MemorySize::memory_size(&self.sm), std::mem::size_of_val(&self.sm)),
            (harmos::MemorySize::memory_size(&self.md), std::mem::size_of_val(&self.md)),
            (harmos::MemorySize::memory_size(&self.lg), std::mem::size_of_val(&self.lg)),
            (harmos::MemorySize::memory_size(&self.xl), std::mem::size_of_val(&self.xl)),
            (harmos::MemorySize::memory_size(&self.two_xl), std::mem::size_of_val(&self.two_xl)),
            (harmos::MemorySize::memory_size(&self.three_xl), std::mem::size_of_val(&self.three_xl)),
            (harmos::MemorySize::memory_size(&self.four_xl), std::mem::size_of_val(&self.four_xl)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for ThemeVariant {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.variables), std::mem::size_of_val(&self.variables)),
            (harmos::MemorySize::memory_size(&self.plot_palette), std::mem::size_of_val(&self.plot_palette)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
