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
use std::{ffi::OsStr, process::Command};

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

pub(crate) fn command(program: impl AsRef<OsStr>) -> Command {
    #[allow(clippy::disallowed_methods, reason = "the replacement host process constructor")]
    let command = Command::new(program);
    command
}

/* === Private Functions === */

/* === Tests === */
