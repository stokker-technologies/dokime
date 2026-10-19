/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod consent;
mod get_installation;
mod list_admitted_plugins;
mod list_installations;
mod remove;
mod set_enabled;
mod stage;

/* === Re-Exports === */
pub use consent::Consent;
pub use get_installation::GetInstallation;
pub use list_admitted_plugins::ListAdmittedPlugins;
pub use list_installations::ListInstallations;
pub use remove::Remove;
pub use set_enabled::SetEnabled;
pub use stage::Stage;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Endpoint;
use super::INSTALLATION;
use super::ORDINARY;
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
    Stage(Stage),
    Consent(Consent),
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

impl Endpoint for Stage {
    const PATH: &'static str = "/dokime.v1.Plugins/Stage";
}

impl Endpoint for Consent {
    const PATH: &'static str = "/dokime.v1.Plugins/Consent";
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
