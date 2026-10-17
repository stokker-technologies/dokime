/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod color;
mod development_environment;
mod new;
mod scaffold;

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
fn wrapper_contract() {
    support::nested_scratch_independent();
    color::compiler_color_policy();
    new::scaffold_contract();
    development_environment::environment_contract();
}

// Red until the Harmos re-pin: the starter declares its observers without an
// `observer_control`, which the built-in `harmos.observe` makes unnecessary.
#[test]
fn generated_plugin_contract() {
    scaffold::generated_plugin_tasks();
}
