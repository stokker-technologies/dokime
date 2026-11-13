/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

/* === Modules === */

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

// A desktop build never owns a console, whatever its Cargo profile: the
// dev-profile installer is a debug build too, and it must not flash a
// terminal at launch. `tauri dev` opts back into one through the `console`
// feature so developers keep the process output.

fn main() {
    dokime_gui::run();
}

/* === Tests === */
