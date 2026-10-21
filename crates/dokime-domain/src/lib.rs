/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[macro_use]
mod macros;
mod error;
pub mod model;
pub mod operations;
pub mod state;

/* === Re-Exports === */

pub use error::Error;
pub use error::Result;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

pub const DEV_PROTOCOL: u32 = 4;
pub const READINESS_PROTOCOL: u64 = 3;
/// How long a stopped run has to answer before the host settles it as
/// stopped; the desktop's close deadline is built on it.
pub const STOP_SETTLEMENT_SECONDS: u64 = 5;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
