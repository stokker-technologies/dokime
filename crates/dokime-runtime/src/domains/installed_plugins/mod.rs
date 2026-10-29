/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod appearance;
pub(crate) mod contributions;
mod handle;
pub(crate) mod management;
pub(crate) mod module_preferences;

pub(crate) use crate::config::organization as brand;

pub use appearance::{
    AppearanceLimitExceeded, CapturedAppearance, RuntimeAppLogo, RuntimeAppearanceFrame, RuntimeInstalledPluginAsset, RuntimeThemeFamily,
};
pub use brand::{EmbeddedLogo, ResolvedBrand, compiled_brand, load_brand};
pub use dokime_domain::model::plugins::InstalledPluginErrorRecord;
pub use dokime_domain::model::plugins::InstalledPluginPhase;
pub use management::{
    PluginRuntimeSecretBindings, PluginSecretUpdate, PluginSettingsUpdate, RuntimePluginManagementError, RuntimePluginSettings,
};
