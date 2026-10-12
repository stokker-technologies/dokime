/* ***
 *
 * Dokime
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
#[sdk::message]
#[derive(Clone, sdk::TelemetryRow)]
struct Shape {
    value: f64,
}

#[sdk::stream(id = "fixture.empty", window = 4096, label = "Empty", description = "A stream with no columns: its rows would carry nothing.")]
struct Empty;

#[sdk::stream(id = "other.gateway.rssi", window = 4096, foreign, description = "A foreign stream with no columns: nothing to read, nothing to check.")]
struct GatewayRssi;

#[sdk::stream(id = "fixture.carried", window = 4096, row = Shape, label = "Carried", description = "A stream type that also declares columns: two shapes for one stream.")]
struct Carried {
    value: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn main() {}

/* === Tests === */
