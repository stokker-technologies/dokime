/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod get_installation;
mod install;
mod list_admitted_plugins;
mod list_installations;
mod remove;
mod set_enabled;

pub use get_installation::GetInstallation;
pub use install::Install;
pub use list_admitted_plugins::ListAdmittedPlugins;
pub use list_installations::ListInstallations;
pub use remove::Remove;
pub use set_enabled::SetEnabled;

use super::Endpoint;
use super::INSTALLATION;
use super::{MUTATION, ORDINARY};

#[allow(unused_imports)]
use dokime_domain::model::identity::wire::*;
use dokime_domain::model::plugins::wire::*;

#[harmos::catalog]
pub enum Operations {
    ListInstallations(ListInstallations),
    GetInstallation(GetInstallation),
    Install(Install),
    SetEnabled(SetEnabled),
    Remove(Remove),
    ListAdmittedPlugins(ListAdmittedPlugins),
}

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
