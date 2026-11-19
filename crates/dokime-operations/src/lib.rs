/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host Operations

/* === Modules === */

mod error;
pub mod host;
pub mod native;
mod resources;
pub mod wrappers;

/* === Re-Exports === */

pub use error::Error;
pub use resources::{Clock, Prompts, Resources};

/* === Imports === */

// Standard Library Imports
use std::result;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
