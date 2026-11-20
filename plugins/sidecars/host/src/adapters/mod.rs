/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Network Adapter Inventory and Typed Snapshots

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use crate::settings::{self, NETWORK_PROBE_TIMEOUT};
use dokime::operations::{Context, Operation};
use dokime::plugin::Plugin;
use dokime::{Error, Message, Result, Visibility};

/* === Constants === */

pub const ADAPTERS: &str = "host.network.adapters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Adapters State Retained Across Operation Calls.
#[derive(Default)]
pub struct State {}

/// Read the Network Adapter Inventory Without Changing It.
#[derive(Message)]
#[dokime(package = "host")]
pub struct AdaptersRead {}

/// Every Network Adapter, Ordered by Identity; a Template Records It From Its Initialization.
#[derive(Clone, Debug, PartialEq, Message)]
#[dokime(package = "host")]
pub struct Snapshot {
    pub adapters: Vec<Adapter>,
}

/// One Network Adapter, Its Kind and Status as `AdapterKind::as_str` and `AdapterStatus::as_str` Name Them.
#[derive(Clone, Debug, PartialEq, Message)]
#[dokime(package = "host")]
pub struct Adapter {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: String,
    pub mac: Option<String>,
    pub status: String,
    pub mtu: u32,
    pub speed_bps: Option<u64>,
    pub ipv4: Vec<Address>,
    pub ipv6: Vec<Address>,
    pub gateways: Vec<String>,
    pub dns_servers: Vec<String>,
    pub dhcp: Option<bool>,
    pub primary: bool,
}

/// One Unicast Address With Its Prefix Length and, for IPv6, Its Scope.
#[derive(Clone, Debug, PartialEq, Message)]
#[dokime(package = "host")]
pub struct Address {
    pub address: String,
    pub prefix: u32,
    pub scope: Option<u32>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterKind {
    Ethernet,
    Wifi,
    Cellular,
    Loopback,
    Virtual,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterStatus {
    Up,
    Down,
}

/* === Traits === */

/* === Implementations === */

impl AdapterKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ethernet => "ethernet",
            Self::Wifi => "wifi",
            Self::Cellular => "cellular",
            Self::Loopback => "loopback",
            Self::Virtual => "virtual",
            Self::Other => "other",
        }
    }
}

impl AdapterStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}

impl Adapter {
    pub fn is(&self, kind: AdapterKind) -> bool {
        self.kind == kind.as_str()
    }
}

/* === Public Functions === */

/// Declare the Adapters Operations.
pub fn catalog(plugin: Plugin) -> Result<Plugin> {
    let read = Operation::new(ADAPTERS, Visibility::Public, async |ctx: &Context, _: AdaptersRead| {
        let timeout = settings::duration_value(ctx, NETWORK_PROBE_TIMEOUT).await?;
        snapshot(timeout).await
    })
    .description("Report Every Network Adapter With Its Kind, Status, Addresses, Gateways, DNS Servers and DHCP State.");
    Ok(plugin.operation(read))
}

/// Read the Native Adapter Inventory Away From the Async Runtime Thread.
pub async fn snapshot(timeout: Duration) -> Result<Snapshot> {
    let task = tokio::task::spawn_blocking(move || native(timeout));
    task.await.map_err(|error| Error::Invalid(format!("Read Adapters: Retry After the Inventory Worker Stopped: {error}")))?
}

/* === Private Functions === */

#[cfg(windows)]
fn native(timeout: Duration) -> Result<Snapshot> {
    crate::platform::windows::adapters::snapshot(timeout)
}

#[cfg(unix)]
fn native(timeout: Duration) -> Result<Snapshot> {
    crate::platform::unix::adapters::snapshot(timeout)
}

/* === Tests === */
