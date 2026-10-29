/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod source;

/* === Re-Exports === */

pub use source::CapturedRecord;
pub(crate) use source::{Process, Projection, Source};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::runtime::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn error(value: impl std::fmt::Display) -> Error {
    Error::Operation(format!("record capture: {value}"))
}

/* === Tests === */
