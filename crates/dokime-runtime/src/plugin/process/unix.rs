/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Own Unix Plugin Process Groups

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io, os::unix::process::CommandExt, process::Command};

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
        command.process_group(0);
        command.spawn().map(Self)
    }

    pub(in crate::plugin) fn kill(&mut self) -> io::Result<()> {
        // The Child Creates This Group Before Starting Any Sidecar Descendant
        let group = i32::try_from(self.0.id()).map_err(io::Error::other)?;
        let killed = unsafe { libc::kill(-group, libc::SIGKILL) };
        if killed != 0 {
            let error = io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error);
            }
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
