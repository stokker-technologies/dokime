/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod temperature;

/* === Re-Exports === */
pub use temperature::Temperature;

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
pub enum Observers {
    Temperature(Temperature),
}

#[dokime_runtime::catalog]
pub enum Streams {
    TemperatureRow(crate::domain::model::chamber::TemperatureRow),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
