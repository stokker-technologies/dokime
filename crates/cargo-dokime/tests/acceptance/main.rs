/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod bundle;
mod color;
mod dev;
mod development_environment;
mod install;
mod installation;
mod new;
mod starter_host;

#[path = "../support/mod.rs"]
mod support;

/* === Re-Exports === */

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

#[test]
fn cargo_workflow_contract() {
    support::nested_scratch_independent();
    color::compiler_color_policy();
    new::scaffold_contract();
    development_environment::environment_contract();
    bundle::bundle_contract();
    installation::installation_contract();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(dev::development_contract());
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(install::consent_contract());
}
