/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Collectors: Native Readings Assembled Into Telemetry Rows

/* === Modules === */

pub mod bluetooth;
pub mod cpu;
pub mod network;
pub mod system;
pub mod wifi;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

const BITS_PER_MEGABIT: f64 = 1_000_000.0;
const BITS_PER_BYTE: f64 = 8.0;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Convert a Byte Rate to the Megabits per Second Throughput Streams Report.
pub fn megabits_per_second(bytes_per_second: f64) -> f64 {
    bytes_per_second * BITS_PER_BYTE / BITS_PER_MEGABIT
}

/* === Private Functions === */

/* === Tests === */
