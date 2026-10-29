/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


extern crate dokime_host;


#[sdk::stream(id = "other.gateway.rssi", window = 4096, foreign)]
struct GatewayRssi {
    #[row(unit = "dBm")]
    rssi: f64,
}


fn published<T: sdk::Publishable>() {}

fn pushed<T: sdk::telemetry::OwnedStream>() {}

fn main() {
    // A foreign stream's rows are the owner's to publish.
    published::<GatewayRssi>();
    pushed::<GatewayRssi>();
    // Only The Host Plugin Publishes Its Rows.
    published::<dokime_host::System>();
    pushed::<dokime_host::System>();
}
