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
use std::{ffi::OsStr, os::windows::process::CommandExt, process::Command};

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

pub fn command(program: impl AsRef<OsStr>) -> Command {
    #[allow(clippy::disallowed_methods, reason = "the replacement host process constructor")]
    let mut command = Command::new(program);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/* === Private Functions === */

/* === Tests === */
