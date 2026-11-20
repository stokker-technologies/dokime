/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Telemetry: Plugin-Wide System, Network, CPU, Wi-Fi and Bluetooth Streams With Host Controls

/* === Modules === */

pub mod adapters;
pub mod collectors;
pub mod controls;
pub mod domain;
pub mod platform;
pub mod power;
pub mod settings;
pub mod streams;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::operations::Context;
use dokime::plugin::Plugin;
use dokime::{Declarations, Result};

/* === Constants === */

pub const IDENTIFIER: &str = "host";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Plugin-Wide State: One Value per Area, Retained From Connect to Disconnect.
#[derive(Default)]
pub struct Host {
    pub system: collectors::system::Collector,
    pub network: collectors::network::Collector,
    pub cpu: collectors::cpu::Collector,
    pub wifi: collectors::wifi::Collector,
    pub bluetooth: collectors::bluetooth::Collector,
    pub controls: controls::State,
    pub adapters: adapters::State,
    pub power: power::State,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/// Compose the Plugin-Wide Streams, Observers, Operations and Settings of Every Area.
pub fn plugin() -> Result<Plugin> {
    let mut plugin = Plugin::of::<Host>(env!("CARGO_PKG_VERSION")).identifier(IDENTIFIER).label("Host Telemetry").initialize(initialize);
    for setting in settings::catalog() {
        plugin = plugin.setting(setting);
    }
    let plugin = streams::system::catalog(plugin)?;
    let plugin = streams::network::catalog(plugin)?;
    let plugin = streams::cpu::catalog(plugin)?;
    let plugin = streams::wifi::catalog(plugin)?;
    let plugin = streams::bluetooth::catalog(plugin)?;
    let plugin = controls::catalog(plugin)?;
    let plugin = adapters::catalog(plugin)?;
    let plugin = power::catalog(plugin)?;
    Ok(plugin.state(Host::default()))
}

/// The Frozen Catalog the Host Admits, Without Opening a Socket.
pub fn declarations() -> Result<Declarations> {
    plugin()?.declarations()
}

/* === Private Functions === */

/// Refuse a Misconfigured Plugin When It Connects, Then Restore Every Control State a Previous Run Left Outstanding.
async fn initialize(ctx: &Context) -> Result<()> {
    settings::validate(ctx).await?;
    controls::restore(ctx).await
}

/* === Tests === */
