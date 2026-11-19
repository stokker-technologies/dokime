/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Resolve Installation Resources

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io, path::Path};

// Third Party Imports

// Local Imports

/* === Constants === */

pub const PLUGINS: &str = "plugins";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Resolve Resources Beside the Installed Product Executable, Including Linux's Real Binary.
pub fn directory(executable: &Path) -> io::Result<&Path> {
    executable.parent().ok_or_else(|| io::Error::other("Resolve Installation: Use a Product Executable With a Parent Folder"))
}

/* === Private Functions === */

/* === Tests === */
