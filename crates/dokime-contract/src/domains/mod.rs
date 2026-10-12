/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod appearance;
pub mod contribution;
mod dispatch;
pub mod error;
mod installed;
mod permission;
pub mod product;

/* === Re-Exports === */
pub use appearance::{RuntimeAppLogo, RuntimeAppearanceFrame, RuntimeThemeFamily};
pub use contribution::{
    AppCustomizationContribution, AppLogoAsset, AppThemeContribution, OptionProviderContribution,
    OptionProviderOption, OptionProviderSource, OrganizationIdentity, PluginEntrypoint,
    ThemeActivationPolicy, ThemeCornerRadii, ThemeModePolicy, ThemeVariant,
};
pub use dispatch::ActionDispatcherError;
pub use error::{InstalledPluginErrorRecord, InstalledPluginPhase};
pub use installed::{InstalledPlugin, PluginListener, SettingDiscovery};
pub use permission::{Permission, PermissionError};
pub use product::{
    ExecutionProviderTarget, SidecarExecutionBinding, SuiteContractError, TestSuiteDefinition,
    resolve_suite_plot_axes,
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
