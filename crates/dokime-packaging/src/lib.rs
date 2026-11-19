/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod bundle;

#[cfg(feature = "installer")]
pub mod installer;

pub mod project;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error, result};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = result::Result<T, Box<dyn error::Error + Send + Sync>>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
