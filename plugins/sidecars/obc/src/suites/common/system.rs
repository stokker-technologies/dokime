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
use dokime::document::{Column, PlotDefinition};
use dokime::prelude::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[dokime::catalog]
pub enum SystemInputs {
    System(plugin_host::System),
    Network(plugin_host::Network),
    WifiLink(plugin_host::WifiLink),
    WifiLatency(plugin_host::WifiLatency),
    WifiScan(plugin_host::WifiScan),
    Bluetooth(plugin_host::Bluetooth),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn section(owner: &str) -> Section {
    let mut section = Section::new("System Information");

    section.plot(series(owner, "cpu_load", "CPU Load", "%", 1, [plugin_host::System::cpu_load_percent()]));
    section.plot(series(owner, "memory_pressure", "Memory Pressure", "%", 1, [plugin_host::System::memory_pressure_percent()]));
    section.plot(series(owner, "memory_used", "Memory Used [GiB]", "GiB", 2, [plugin_host::System::memory_used_gib()]));
    section.plot(series(owner, "battery_power", "Battery Power", "W", 2, [plugin_host::System::battery_energy_rate_w()]));
    section.plot(series(owner, "battery_voltage", "Battery Voltage", "V", 2, [plugin_host::System::battery_voltage_v()]));
    section.plot(series(
        owner,
        "battery_energy",
        "Battery Energy",
        "Wh",
        1,
        [
            plugin_host::System::battery_energy_wh(),
            plugin_host::System::battery_energy_full_wh(),
            plugin_host::System::battery_energy_full_design_wh(),
        ],
    ));
    section.plot(series(
        owner,
        "battery_state_of_charge",
        "Battery State of Charge",
        "ratio",
        2,
        [plugin_host::System::battery_state_of_charge_ratio()],
    ));
    section.plot(series(owner, "battery_temperature", "Battery Temperature", "°C", 1, [plugin_host::System::battery_temperature_c()]));
    section.plot(series(owner, "wifi_rssi", "Wi-Fi RSSI", "dBm", 0, [plugin_host::WifiLink::wifi_rssi_dbm()]));
    section.plot(series(
        owner,
        "wifi_phy_rate",
        "Wi-Fi PHY Rate",
        "Mbps",
        1,
        [plugin_host::WifiLink::wifi_rx_phy_mbps(), plugin_host::WifiLink::wifi_tx_phy_mbps()],
    ));
    section.plot(series(
        owner,
        "wifi_throughput",
        "Wi-Fi Throughput",
        "Mbps",
        2,
        [plugin_host::WifiLink::wifi_rx_mbps(), plugin_host::WifiLink::wifi_tx_mbps()],
    ));
    // The Scan Itself: Every Channel With Its Network Count And Strongest Signal
    let mut occupancy = plugin_host::domain::document::wifi_channel_occupancy("Wi-Fi Channel Occupancy");
    occupancy.id(format!("{owner}.system.wifi_channel_occupancy"));
    section.plot(occupancy);
    section.plot(series(owner, "wifi_latency", "Wi-Fi Latency", "ms", 1, [plugin_host::WifiLatency::wifi_latency_ms()]));

    section
}

/* === Private Functions === */

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

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use dokime::document::DocumentCardBody;
    use dokime::domain::model::expressions::DataSource;

    #[test]
    fn memory_plot_bindings() {
        let section = section("test");
        let plots = section
            .cards
            .iter()
            .filter_map(|card| match &card.body {
                DocumentCardBody::Plot { plot } => Some(plot.as_ref()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let pressure_index = plots.iter().position(|plot| plot.metadata.identifier.as_str() == "test.system.memory_pressure").unwrap();
        let pressure = plots[pressure_index];
        let used = plots[pressure_index + 1];
        assert_eq!(pressure.y_expressions.len(), 1);
        assert_eq!(
            pressure.y_expressions[0].source,
            DataSource::Telemetry {
                stream: "host.system".into(),
                field: "memory_pressure_percent".into()
            }
        );
        assert_eq!(pressure.unit.as_deref(), Some("%"));
        assert_eq!(used.metadata.identifier.as_str(), "test.system.memory_used");
        assert_eq!(used.y_expressions.len(), 1);
        assert_eq!(
            used.y_expressions[0].source,
            DataSource::Telemetry {
                stream: "host.system".into(),
                field: "memory_used_gib".into()
            }
        );
        assert_eq!(used.unit.as_deref(), Some("GiB"));
    }
}
