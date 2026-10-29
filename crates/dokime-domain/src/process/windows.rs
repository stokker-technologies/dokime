/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{os::windows::process::CommandExt, process::Command};

// Third Party Imports

// Local Imports

/* === Constants === */

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn without_console(command: &mut Command) {
    command.creation_flags(CREATE_NO_WINDOW);
}

/* === Private Functions === */

/* === Tests === */
