/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Own Windows Plugin Processes

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io, process::Command};

// Third Party Imports

// Local Imports
use super::OwnedChild;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl OwnedChild {
    pub(in crate::plugin) fn spawn(mut command: Command) -> io::Result<Self> {
        command.spawn().map(Self)
    }

    pub(in crate::plugin) fn kill(&mut self) -> io::Result<()> {
        self.0.kill()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
