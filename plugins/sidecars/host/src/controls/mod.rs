/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Mutating Radio and Adapter Controls, Gated on the Host Effect Journal

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

/// Controls State Retained Across Operation Calls.
#[derive(Default)]
pub struct State {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Declare the Controls Operations.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    Ok(plugin)
}

/* === Private Functions === */

/* === Tests === */
