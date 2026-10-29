/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::{Bluetooth, Network, System, WifiLatency, WifiLink, WifiScan};

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Catalog {
    System(System),
    Network(Network),
    WifiLink(WifiLink),
    WifiScan(WifiScan),
    WifiLatency(WifiLatency),
    Bluetooth(Bluetooth),
}

impl dokime::catalog::IdentifierCatalog<dokime::catalog::StreamIdentifiers> for Catalog {
    const IDENTIFIERS: &'static [dokime::catalog::Node<&'static str>] =
        &<Self as dokime::catalog::Catalog<dokime::catalog::StreamIdentifiers>>::ENTRIES;
}
