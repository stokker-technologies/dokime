/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Platform Calls, One Module per Operating System Family

/* === Modules === */

pub mod command;
#[cfg(unix)]
pub mod unix;
#[cfg(target_os = "windows")]
pub mod windows;

/* === Re-Exports === */

#[cfg(target_os = "windows")]
pub use windows::apartment::Lease;
#[cfg(unix)]
pub use unix::Lease;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Basic Service Set Reported by a Wi-Fi Scan.
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
