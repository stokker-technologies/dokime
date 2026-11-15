/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The One Constructor For Child Processes.
//
// A Windows GUI-subsystem parent, such as the desktop, gives every console
// child it starts a visible console window of its own. Every Dokime spawn
// starts through `command`, which starts Windows children without one; Clippy's
// `disallowed-methods` refuses any other `Command::new`.
//
// On Windows the host's kill-on-close job owns plugins and hosts. `open` and
// `detach_later_children` start what the user should keep, such as a viewer or
// an installer, outside that job.

/* === Modules === */

#[cfg(unix)]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

/* === Re-Exports === */

#[cfg(unix)]
pub use unix::{command, detach_later_children, own_descendants, own_later_children};
#[cfg(target_os = "windows")]
pub use windows::{command, detach_later_children, open, own_descendants, own_later_children};

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

/* === Tests === */
