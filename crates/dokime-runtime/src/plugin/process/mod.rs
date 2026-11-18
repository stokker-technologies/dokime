/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Own Plugin Child Lifetimes

/* === Modules === */

#[cfg(unix)]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process::Child;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct OwnedChild(pub(super) Child);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for OwnedChild {
    fn drop(&mut self) {
        // Reap Before Rollback Removes the Executable, Including on Cancellation
        let _ = self.kill();
        let _ = self.0.wait();
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
