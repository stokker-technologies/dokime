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
pub(crate) use source::{Process, Projection, Source, group_limits};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn read_capacity(error: &Error) -> bool {
    matches!(
        error,
        Error::Harmos(harmos::Error::ResourceExhausted {
            resource: "storage read memory" | "storage views"
        }) | Error::Storage(dokime::storage::Error::Runtime(
            harmos::Error::ResourceExhausted {
                resource: "storage read memory" | "storage views"
            }
        ))
    )
}

/* === Private Functions === */
fn error(value: impl std::fmt::Display) -> Error {
    Error::Operation(format!("record capture: {value}"))
}

/* === Tests === */
