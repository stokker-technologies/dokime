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
use super::{Bluetooth, Network, System, WifiLatency, WifiLink, WifiScan};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
pub enum Catalog {
    System(System),
    Network(Network),
    WifiLink(WifiLink),
    WifiScan(WifiScan),
    WifiLatency(WifiLatency),
    Bluetooth(Bluetooth),
}

#[dokime::catalog]
pub enum Evidence {
    System(System),
    Network(Network),
    WifiLink(WifiLink),
    WifiScan(WifiScan),
    WifiLatency(WifiLatency),
    Bluetooth(Bluetooth),
    Control(super::cpu::load::LoadControl),
}

/* === Traits === */

/* === Implementations === */

impl dokime::catalog::IdentifierCatalog<dokime::catalog::StreamIdentifiers> for Catalog {
    const IDENTIFIERS: &'static [dokime::catalog::Node<&'static str>] =
        &<Self as dokime::catalog::Catalog<dokime::catalog::StreamIdentifiers>>::ENTRIES;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
