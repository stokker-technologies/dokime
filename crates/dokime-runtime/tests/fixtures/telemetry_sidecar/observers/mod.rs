/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod continuous;
pub(super) mod controls;
pub(super) mod record_demand;

/* === Re-Exports === */
pub(super) use continuous::Continuous;
pub(super) use record_demand::RecordDemand;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[dokime_runtime::catalog]
pub(crate) enum Observers {
    Continuous(Continuous),
    RecordDemand(RecordDemand),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
