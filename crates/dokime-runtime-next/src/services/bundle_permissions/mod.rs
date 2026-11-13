/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

#[cfg(unix)]
mod unix;

#[cfg(target_os = "windows")]
mod windows;

/* === Re-Exports === */

#[cfg(unix)]
pub(super) use unix::executable;

#[cfg(target_os = "windows")]
pub(super) use windows::executable;

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
