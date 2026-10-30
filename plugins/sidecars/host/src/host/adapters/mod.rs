/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[cfg(feature = "runtime")]
mod capture;

#[cfg(all(feature = "runtime", unix))]
mod unix;
#[cfg(all(feature = "runtime", target_os = "windows"))]
pub(crate) mod windows;

/* === Re-Exports === */

#[cfg(all(feature = "runtime", unix))]
pub use unix::snapshot;
#[cfg(all(feature = "runtime", target_os = "windows"))]
pub use windows::snapshot;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::operation(id = "host.network.adapters", response = Snapshot, policy = super::network::POLICY, visibility = Public,
    resources = [crate::Workers])]
#[derive(Clone, Default, PartialEq)]
pub struct Adapters {}

#[dokime::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    pub adapters: Vec<Adapter>,
}

#[dokime::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Adapter {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: AdapterKind,
    pub mac: Option<String>,
    pub status: AdapterStatus,
    pub mtu: u32,
    pub speed_bps: Option<u64>,
    pub ipv4: Vec<Address>,
    pub ipv6: Vec<Address>,
    pub gateways: Vec<String>,
    pub dns_servers: Vec<String>,
    pub dhcp: Option<bool>,
    pub primary: bool,
}

#[dokime::message]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Address {
    pub address: String,
    pub prefix: u32,
    pub scope: Option<u32>,
}

#[dokime::operation(id = "host.network.adapters.capture", response = dokime::operations::effects::StartEvidence, policy = super::network::POLICY, visibility = Host,
    resources = [crate::Workers])]
#[derive(Clone, Default, PartialEq)]
pub struct CaptureAdapters {}

/* === Enums === */

#[dokime::message]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterKind {
    Ethernet,
    Wifi,
    Cellular,
    Loopback,
    Virtual,
    Other,
}

#[dokime::message]
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterStatus {
    Up,
    Down,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
