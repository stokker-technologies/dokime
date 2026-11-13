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

/* === Modules === */

#[cfg(unix)]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

/* === Re-Exports === */

#[cfg(unix)]
pub use unix::command;
#[cfg(target_os = "windows")]
pub use windows::command;

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
