/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod appearance;
pub(crate) mod brand;
mod bundled;
pub(crate) mod contributions;
mod handle;
pub(crate) mod management;
pub(crate) mod module_preferences;

/* === Re-Exports === */
pub use appearance::{
    AppearanceLimitExceeded, CapturedAppearance, RuntimeAppLogo, RuntimeAppearanceFrame,
    RuntimeInstalledPluginAsset, RuntimeThemeFamily,
};
pub use brand::{EmbeddedLogo, ResolvedBrand, compiled_brand, load_brand};
pub(crate) use bundled::builtin_plugins;
pub use dokime_contract::domains::{InstalledPluginErrorRecord, InstalledPluginPhase};
pub use management::{
    PluginCredentialSecretUpdate, PluginRuntimeCredentialBindings, PluginSettingsUpdate,
    RuntimePluginManagementError, RuntimePluginSettings,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
