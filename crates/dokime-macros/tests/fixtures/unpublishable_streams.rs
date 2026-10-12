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
#[sdk::stream(id = "other.gateway.rssi", window = 4096, foreign)]
struct GatewayRssi {
    #[row(unit = "dBm")]
    rssi: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn published<T: sdk::Publishable>() {}

fn pushed<T: sdk::telemetry::OwnedStream>() {}

fn main() {
    // A foreign stream's rows are the owner's to publish.
    published::<GatewayRssi>();
    pushed::<GatewayRssi>();
    // A host stream's rows are the runtime's to measure.
    published::<sdk::host::System>();
    pushed::<sdk::host::System>();
}

/* === Tests === */
