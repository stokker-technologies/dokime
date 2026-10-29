/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// TODO: Describe this module's responsibility.

/* === Modules === */

/* === Re-Exports === */
pub use crate::model::plugins::LogoVariant;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceSnapshot {
    pub app_name: String,
    pub organization: Option<OrganizationIdentity>,
    pub title_template: Option<String>,
    pub logos: Vec<AppearanceLogo>,
    pub themes: Vec<ThemeFamily>,
    pub active_theme: Option<String>,
    pub theme_family_locked: bool,
    pub mode_policy: ThemeModePolicy,
    pub mode_locked: bool,
    pub source_plugin_id: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OrganizationIdentity {
    pub name: String,
    pub unit: Option<String>,
    pub website: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceLogo {
    pub variant: LogoVariant,
    pub asset: Option<AppearanceAssetMetadata>,
    pub alt: Option<String>,
}

// Public registered asset identity, never a path or HTTP URL. A changed digest
// requires fetching appearance again; the download never substitutes new bytes.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceAssetReference {
    pub plugin_id: String,
    pub asset_id: String,
    pub sha256: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceAssetMetadata {
    pub reference: Option<AppearanceAssetReference>,
    pub content_type: String,
    pub size_bytes: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeFamilyVariablesEntry {
    pub key: String,
    pub value: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeFamily {
    pub plugin_id: String,
    pub theme_id: String,
    pub label: String,
    pub description: Option<String>,

    // Self-contained authored CSS, including embedded font data. Bounded by the
    // appearance response budget; no external resource-resolution API is implied.
    #[row(
        description = "Self-contained authored CSS, including embedded font data. Bounded by the appearance response budget; no external resource-resolution API is implied."
    )]
    pub stylesheet: Option<String>,

    pub variables: Vec<ThemeFamilyVariablesEntry>,
    pub corner_radii: Option<ThemeCornerRadii>,
    pub light: Option<ThemeVariant>,
    pub dark: Option<ThemeVariant>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeCornerRadii {
    pub base: Option<String>,
    pub sm: Option<String>,
    pub md: Option<String>,
    pub lg: Option<String>,
    pub xl: Option<String>,
    pub two_xl: Option<String>,
    pub three_xl: Option<String>,
    pub four_xl: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeVariantVariablesEntry {
    pub key: String,
    pub value: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeVariant {
    pub variables: Vec<ThemeVariantVariablesEntry>,
    pub plot_palette: Vec<String>,
}

// Typed details in a standard google.rpc.Status envelope.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceFailure {
    pub kind: AppearanceFailureKind,
}

/* === Enums === */

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeModePolicy {
    Unspecified,
    UserOrSystem,
    ForceSystem,
    ForceLight,
    ForceDark,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppearanceFailureKind {
    Unspecified,
    AssetMissing,
    AssetChanged,
    TooLarge,
    InvalidReference,
    AssetUnavailable,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
