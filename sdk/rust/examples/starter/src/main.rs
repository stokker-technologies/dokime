/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Serve the Starter Plugin

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Result;
use starter_project::plugin::plugin;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn main() -> Result<()> {
    plugin()?.run()
}

/* === Tests === */
