/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Network Adapter Inventory and Typed Snapshots

/* === Modules === */

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

/// Adapters State Retained Across Operation Calls.
#[derive(Default)]
pub struct State {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Adapters Operations.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    Ok(plugin)
}

/* === Private Functions === */

/* === Tests === */
