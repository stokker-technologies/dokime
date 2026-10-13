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
use dokime::core::Value;
use dokime::document::{Column, DocumentFilter, FilterOperator, PlotDefinition};
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
/// The host streams every Mini template leases, so the system section below
/// draws what each run captured. Each stream declares its own cadence.
#[dokime::catalog]
pub enum SystemInputs {
    System(host::System),
    Network(host::Network),
    Wifi(host::Wifi),
    WifiLatency(host::WifiLatency),
    WifiChannelOccupancy(host::WifiChannelOccupancy),
    Bluetooth(host::Bluetooth),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// The host around the bench, the same for every Mini template: appended last
/// in each template's document under ids the template `owner` prefixes.
pub fn section(owner: &str) -> Section {
    let mut section = Section::new(format!("{owner}.system"), "System Information");

    section.plot(series(
        owner,
        "cpu_load",
        "CPU Load",
        "%",
        1,
        [host::System::cpu_load_percent()],
    ));
    section.plot(series(
        owner,
        "memory_pressure",
        "Memory Pressure",
        "%",
        1,
        [host::System::memory_pressure_percent()],
    ));
    section.plot(series(
        owner,
        "battery_power",
        "Battery Power",
        "W",
        2,
        [host::System::battery_energy_rate_w()],
    ));
    section.plot(series(
        owner,
        "battery_voltage",
        "Battery Voltage",
        "V",
        2,
        [host::System::battery_voltage_v()],
    ));
    section.plot(series(
        owner,
        "battery_energy",
        "Battery Energy",
        "Wh",
        1,
        [
            host::System::battery_energy_wh(),
            host::System::battery_energy_full_wh(),
            host::System::battery_energy_full_design_wh(),
        ],
    ));
    section.plot(series(
        owner,
        "battery_state_of_charge",
        "Battery State of Charge",
        "ratio",
        2,
        [host::System::battery_state_of_charge_ratio()],
    ));
    section.plot(series(
        owner,
        "battery_temperature",
        "Battery Temperature",
        "°C",
        1,
        [host::System::battery_temperature_c()],
    ));
    section.plot(series(
        owner,
        "wifi_rssi",
        "Wi-Fi RSSI",
        "dBm",
        0,
        [host::Wifi::wifi_rssi_dbm()],
    ));
    // A scan emits one row per channel; only the row of the channel the host
    // is associated with is drawn, so each scan contributes one point.
    let mut occupancy = series(
        owner,
        "wifi_channel_occupancy",
        "Wi-Fi Channel Occupancy",
        "networks",
        1,
        [
            host::WifiChannelOccupancy::same_channel_network_count(),
            host::WifiChannelOccupancy::overlapping_network_count(),
            host::WifiChannelOccupancy::overlap_weighted_network_count(),
        ],
    );
    occupancy.filter(active_channel());
    section.plot(occupancy);
    let mut signal = series(
        owner,
        "wifi_channel_signal",
        "Wi-Fi Active Channel Signal",
        "%",
        0,
        [host::WifiChannelOccupancy::max_signal_percent()],
    );
    signal.filter(active_channel());
    section.plot(signal);
    section.plot(series(
        owner,
        "wifi_latency",
        "Wi-Fi Latency",
        "ms",
        1,
        [host::WifiLatency::wifi_latency_ms()],
    ));

    section
}

/* === Private Functions === */
fn active_channel() -> DocumentFilter {
    DocumentFilter {
        expression: host::WifiChannelOccupancy::active_channel().into(),
        operator: FilterOperator::Eq,
        value: Some(Value::from(true)),
    }
}

fn series<R: TelemetryStream>(
    owner: &str,
    metric: &str,
    label: &str,
    unit: &str,
    decimals: u8,
    columns: impl IntoIterator<Item = Column<R>>,
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

/* === Tests === */
