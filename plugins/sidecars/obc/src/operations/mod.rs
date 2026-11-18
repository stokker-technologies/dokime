/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin-Level Manual Operation Catalog

/* === Modules === */

pub mod power;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Result;
use dokime::plugin::Plugin;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn catalog(mut plugin: Plugin) -> Result<Plugin> {
    let manual = power::Manual::new()?;
    for operation in manual.operations() {
        plugin = plugin.operation(operation);
    }
    Ok(plugin.on_stop(move || manual.power_down()))
}

/* === Private Functions === */

/* === Tests === */
