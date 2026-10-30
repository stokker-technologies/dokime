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

/* === Enums === */

#[dokime::catalog]
pub enum Observers {
    System(super::System),
    Network(super::network::NetworkObserver),
    WifiLink(super::WifiLink),
    WifiScan(super::WifiScan),
    WifiLatency(super::WifiLatency),
    Bluetooth(super::Bluetooth),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
