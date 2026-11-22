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

#[cfg(unix)]
pub use unix::Lease;
#[cfg(unix)]
pub use unix::wifi;
#[cfg(target_os = "windows")]
pub use windows::apartment::Lease;
#[cfg(target_os = "windows")]
pub use windows::wifi;

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
    pub channel_width_mhz: Option<u32>,
    pub center_channel: Option<u32>,
    pub security: Option<String>,
    pub associated: bool,
}

/// Adapter Bands and Whether Its Regulatory Domain Allows Channel 14.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WifiCapabilities {
    pub bands: Vec<WifiBand>,
    pub channel_14_allowed: bool,
}

/* === Enums === */

/// Frequency Bands Kept Separate Even Where Their Display Channels Coincide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WifiBand {
    Ghz24,
    Ghz5,
    Ghz6,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
