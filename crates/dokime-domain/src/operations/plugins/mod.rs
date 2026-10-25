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
use super::Endpoint;
use super::INSTALLATION;
use super::{MUTATION, ORDINARY};

#[allow(unused_imports)]
use crate::model::identity::wire::*;
use crate::model::plugins::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos_domain::catalog]
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

impl Endpoint for ListInstallations {
    const PATH: &'static str = "/dokime.v1.Plugins/ListInstallations";
}

impl Endpoint for GetInstallation {
    const PATH: &'static str = "/dokime.v1.Plugins/GetInstallation";
}

impl Endpoint for Install {
    const PATH: &'static str = "/dokime.v1.Plugins/Install";
}

impl Endpoint for SetEnabled {
    const PATH: &'static str = "/dokime.v1.Plugins/SetEnabled";
}

impl Endpoint for Remove {
    const PATH: &'static str = "/dokime.v1.Plugins/Remove";
}

impl Endpoint for ListAdmittedPlugins {
    const PATH: &'static str = "/dokime.v1.Plugins/ListAdmittedPlugins";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
