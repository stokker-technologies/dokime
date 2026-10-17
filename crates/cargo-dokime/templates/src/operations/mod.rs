/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod chamber;
pub mod status;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
/// The handlers this plugin binds, one per request in `domain::operations`.
#[dokime_runtime::catalog]
pub enum Implemented {
    StatusProbe(status::StatusProbe),
    ChamberReader(chamber::ChamberReader),
    PortDetector(chamber::PortDetector),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
