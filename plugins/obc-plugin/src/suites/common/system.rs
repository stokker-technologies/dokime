/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime::document::{Column, PlotDefinition};
use dokime::prelude::*;

#[dokime::catalog]
pub enum SystemInputs {
    System(host::System),
    Network(host::Network),
    Wifi(host::Wifi),
    WifiLatency(host::WifiLatency),
    WifiChannelOccupancy(host::WifiChannelOccupancy),
    Bluetooth(host::Bluetooth),
}

pub fn section(owner: &str) -> Section {
    let mut section = Section::new("System Information");

    section.plot(series(owner, "cpu_load", "CPU Load", "%", 1, [host::System::cpu_load_percent()]));
    section.plot(series(owner, "memory_pressure", "Memory Pressure", "%", 1, [host::System::memory_pressure_percent()]));
    section.plot(series(owner, "battery_power", "Battery Power", "W", 2, [host::System::battery_energy_rate_w()]));
    section.plot(series(owner, "battery_voltage", "Battery Voltage", "V", 2, [host::System::battery_voltage_v()]));
    section.plot(series(
        owner,
        "battery_energy",
        "Battery Energy",
        "Wh",
        1,
        [host::System::battery_energy_wh(), host::System::battery_energy_full_wh(), host::System::battery_energy_full_design_wh()],
    ));
    section.plot(series(
        owner,
        "battery_state_of_charge",
        "Battery State of Charge",
        "ratio",
        2,
        [host::System::battery_state_of_charge_ratio()],
    ));
    section.plot(series(owner, "battery_temperature", "Battery Temperature", "°C", 1, [host::System::battery_temperature_c()]));
    section.plot(series(owner, "wifi_rssi", "Wi-Fi RSSI", "dBm", 0, [host::Wifi::wifi_rssi_dbm()]));
    section.plot(series(
        owner,
        "wifi_phy_rate",
        "Wi-Fi PHY Rate",
        "Mbps",
        1,
        [host::Wifi::wifi_rx_phy_mbps(), host::Wifi::wifi_tx_phy_mbps()],
    ));
    section.plot(series(owner, "wifi_throughput", "Wi-Fi Throughput", "Mbps", 2, [host::Wifi::wifi_rx_mbps(), host::Wifi::wifi_tx_mbps()]));
    // The Scan Itself: Every Channel With Its Network Count And Strongest Signal
    let mut occupancy = Plot::wifi_channel_occupancy("Wi-Fi Channel Occupancy");
    occupancy.id(format!("{owner}.system.wifi_channel_occupancy"));
    section.plot(occupancy);
    section.plot(series(owner, "wifi_latency", "Wi-Fi Latency", "ms", 1, [host::WifiLatency::wifi_latency_ms()]));

    section
}

fn series<R: TelemetryStream>(
    owner: &str, metric: &str, label: &str, unit: &str, decimals: u8, columns: impl IntoIterator<Item = Column<R>>,
) -> PlotDefinition {
    let mut plot = Plot::time_series(label);
    plot.id(format!("{owner}.system.{metric}"));
    plot.unit(unit);
    plot.y_format(format::fixed(decimals));
    for column in columns {
        plot.y(column);
    }

    plot
}
