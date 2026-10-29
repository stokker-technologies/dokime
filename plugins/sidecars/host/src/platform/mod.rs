/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(all(feature = "runtime", target_os = "windows"))]
pub mod windows;

#[cfg(all(feature = "runtime", unix))]
pub mod unix;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// One Basic Service Set Reported By A Wi-Fi Scan.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScannedNetwork {
    pub adapter: Option<String>,
    pub ssid: Option<String>,
    pub bssid: Option<String>,
    pub signal_percent: Option<f64>,
    pub rssi_dbm: Option<f64>,
    pub channel: Option<u32>,
    pub security: Option<String>,
    pub associated: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
