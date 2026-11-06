/* ***
 *
 * {{package}}
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub use plugin_host::System;

// The row type declares the stream, so the id is written once and every
// column comes with a handle a plot can name it by:
// `TemperatureRow::temperature_c()` stops resolving the day the field is renamed.
// The observer below says how often, and everything that reads these rows names
// this type.
#[dokime::stream(
    id = "{{crate}}.temperature",
    window = 4096,
    label = "Bench Temperature",
    description = "The chamber's temperature, independent of any execution."
)]
pub struct TemperatureRow {
    #[row(unit = "°C")]
    pub temperature_c: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
