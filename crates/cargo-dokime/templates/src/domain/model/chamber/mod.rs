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

// What `ReadChamber` answers.
#[dokime::message]
#[derive(Clone, PartialEq)]
pub struct ChamberReading {
    #[row(description = "What the chamber is at, in °C.")]
    pub celsius: f64,
    #[row(
        description = "What read it, for the traceability a template states.\nA template never names this plugin, so anything it wants to say about the bench it reads off a reply: what an operation answers is the whole of what a run knows about the process serving it."
    )]
    pub sensor: String,
}

// The row type declares the stream, so the id is written once and every
// column comes with a handle a plot can name it by:
// `TemperatureRow::celsius()` stops resolving the day the field is renamed.
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
    pub celsius: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
