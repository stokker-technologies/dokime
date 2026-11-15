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
use std::{ffi::OsStr, io, process::Command};

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

pub fn own_descendants() -> io::Result<()> {
    // The Library Owns Unix Child Termination When the Host Dies.
    Ok(())
}

pub fn detach_later_children() -> io::Result<()> {
    // Unix Children Outlive Their Parent Unless the Library Owns Them.
    Ok(())
}

pub fn own_later_children() -> io::Result<()> {
    Ok(())
}

#[allow(clippy::disallowed_methods, reason = "the replacement host process constructor")]
pub fn command(program: impl AsRef<OsStr>) -> Command {
    Command::new(program)
}

/* === Private Functions === */

/* === Tests === */
