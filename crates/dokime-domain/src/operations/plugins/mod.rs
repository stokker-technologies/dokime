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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
#[allow(unused_imports)]
use crate::model::identity::wire::*;
use super::Access;
use super::Endpoint;
use super::INSTALLATION;
use super::ORDINARY;

use crate::model::plugins::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */











































/* === Enums === */







#[harmos::catalog]
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
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for GetInstallation {
    const PATH: &'static str = "/dokime.v1.Plugins/GetInstallation";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for Stage {
    const PATH: &'static str = "/dokime.v1.Plugins/Stage";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for Consent {
    const PATH: &'static str = "/dokime.v1.Plugins/Consent";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for SetEnabled {
    const PATH: &'static str = "/dokime.v1.Plugins/SetEnabled";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for Remove {
    const PATH: &'static str = "/dokime.v1.Plugins/Remove";
    const ACCESS: Access = Access::Admin;
}

impl Endpoint for ListAdmittedPlugins {
    const PATH: &'static str = "/dokime.v1.Plugins/ListAdmittedPlugins";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
