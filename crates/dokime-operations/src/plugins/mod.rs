/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod get_installation;
mod install;
mod list_admitted_plugins;
mod list_installations;
mod remove;
mod set_enabled;

/* === Re-Exports === */

pub use get_installation::GetInstallation;
pub use install::Install;
pub use list_admitted_plugins::ListAdmittedPlugins;
pub use list_installations::ListInstallations;
pub use remove::Remove;
pub use set_enabled::SetEnabled;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::INSTALLATION;
use super::{MUTATION, ORDINARY};

use dokime_domain::model::plugins::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    ListInstallations(ListInstallations),
    GetInstallation(GetInstallation),
    Install(Install),
    SetEnabled(SetEnabled),
    Remove(Remove),
    ListAdmittedPlugins(ListAdmittedPlugins),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
