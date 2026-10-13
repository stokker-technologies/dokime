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
use std::collections::{BTreeMap, HashMap};
// Third Party Imports

// Local Imports
use crate::{HostStreamSample, HostTelemetryStream};
use dokime_contract::core::Value;

/* === Constants === */
pub const HOST_FIXTURE_SOURCE: &str = "host.fixture";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default)]
pub struct FixtureHostTelemetrySource {
    sample_indices: HashMap<HostTelemetryStream, u64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl FixtureHostTelemetrySource {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect(&mut self, stream: HostTelemetryStream) -> Vec<HostStreamSample> {
        let index = self.next_index(stream);
        match stream {
            HostTelemetryStream::System => vec![HostStreamSample::new(
                stream,
                BTreeMap::from([
                    ("sample_index".to_owned(), Value::Integer(index as i64)),
                    ("probe_status".to_owned(), Value::from("ok")),
                    ("host_name".to_owned(), Value::from("fixture-host")),
                    ("user".to_owned(), Value::from("fixture-user")),
                    ("git_user_name".to_owned(), Value::from("Fixture User")),
                    (
                        "git_user_email".to_owned(),
                        Value::from("fixture@example.invalid"),
                    ),
                    ("os_name".to_owned(), Value::from("fixture-os")),
                    ("os_version".to_owned(), Value::from("Fixture OS 1.0")),
                    ("kernel_version".to_owned(), Value::from("fixture-kernel")),
                    ("cpu_architecture".to_owned(), Value::from("x86_64")),
                    ("cpu_name".to_owned(), Value::from("fixture-cpu0")),
                    ("cpu_brand".to_owned(), Value::from("Fixture CPU")),
                    ("cpu_vendor_id".to_owned(), Value::from("FixtureVendor")),
                    ("cpu_frequency_mhz".to_owned(), Value::Integer(3200)),
                    ("physical_cpu_count".to_owned(), Value::Integer(4)),
                    ("logical_cpu_count".to_owned(), Value::Integer(8)),
                    ("cpu_load_percent".to_owned(), Value::from(12.5)),
                    ("memory_total_bytes".to_owned(), Value::Integer(32_000)),
                    ("memory_used_bytes".to_owned(), Value::Integer(16_000)),
                    ("memory_pressure_percent".to_owned(), Value::from(41.0)),
                    ("system_uptime_s".to_owned(), Value::from(index as f64)),
                    (
                        "power_plan_name".to_owned(),
                        Value::from("High performance"),
                    ),
                    (
                        "power_plan_guid".to_owned(),
                        Value::from("8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"),
                    ),
                    ("power_mode".to_owned(), Value::from("best_performance")),
                    ("power_mode_performance_bias".to_owned(), Value::Integer(0)),
                    // One discharging lithium-ion pack whose readings agree:
                    // 42 of 50 Wh is 84 %, 50 of 57 Wh design is 0.877
                    // health, and 42 Wh at 8.5 W lasts 17 788 s.
                    ("battery_percent".to_owned(), Value::from(84.0)),
                    ("battery_state".to_owned(), Value::from("discharging")),
                    ("battery_power_plugged".to_owned(), Value::from(false)),
                    ("battery_temperature_c".to_owned(), Value::from(31.5)),
                    ("battery_vendor".to_owned(), Value::from("Fixture Cells")),
                    ("battery_model".to_owned(), Value::from("FX-5700")),
                    ("battery_serial_number".to_owned(), Value::from("FX0001")),
                    ("battery_technology".to_owned(), Value::from("lithium-ion")),
                    (
                        "battery_state_of_charge_ratio".to_owned(),
                        Value::from(0.84),
                    ),
                    ("battery_energy_wh".to_owned(), Value::from(42.0)),
                    ("battery_energy_full_wh".to_owned(), Value::from(50.0)),
                    (
                        "battery_energy_full_design_wh".to_owned(),
                        Value::from(57.0),
                    ),
                    ("battery_energy_rate_w".to_owned(), Value::from(8.5)),
                    ("battery_voltage_v".to_owned(), Value::from(11.4)),
                    (
                        "battery_state_of_health_ratio".to_owned(),
                        Value::from(50.0 / 57.0),
                    ),
                    ("battery_time_to_empty_s".to_owned(), Value::from(17_788.0)),
                    ("battery_cycle_count".to_owned(), Value::Integer(212)),
                ]),
            )],
            HostTelemetryStream::Network => vec![HostStreamSample::new(
                stream,
                BTreeMap::from([
                    ("sample_index".to_owned(), Value::Integer(index as i64)),
                    ("probe_status".to_owned(), Value::from("ok")),
                    ("radio_link".to_owned(), Value::from("fixture")),
                    (
                        "network_recv_mb_per_second".to_owned(),
                        Value::from((1_024.0 + index as f64) / 1_000_000.0),
                    ),
                    (
                        "network_sent_mb_per_second".to_owned(),
                        Value::from((512.0 + index as f64) / 1_000_000.0),
                    ),
                    ("network_interface_count".to_owned(), Value::Integer(1)),
                ]),
            )],
            HostTelemetryStream::Wifi => vec![HostStreamSample::new(
                stream,
                BTreeMap::from([
                    ("sample_index".to_owned(), Value::Integer(index as i64)),
                    ("probe_status".to_owned(), Value::from("ok")),
                    ("wifi_adapter".to_owned(), Value::from("fixture-wifi")),
                    ("wifi_state".to_owned(), Value::from("connected")),
                    ("wifi_ssid".to_owned(), Value::from("DOKIME-Fixture")),
                    ("wifi_channel".to_owned(), Value::Integer(6)),
                    ("wifi_band".to_owned(), Value::from("2.4GHz")),
                    ("wifi_signal_percent".to_owned(), Value::from(86.0)),
                    ("wifi_rx_phy_mbps".to_owned(), Value::from(144.0)),
                    ("wifi_tx_phy_mbps".to_owned(), Value::from(144.0)),
                    (
                        "wifi_rx_mbps".to_owned(),
                        Value::from((128_000.0 + index as f64) * 8.0 / 1_000_000.0),
                    ),
                    (
                        "wifi_tx_mbps".to_owned(),
                        Value::from((96_000.0 + index as f64) * 8.0 / 1_000_000.0),
                    ),
                    (
                        "wifi_network_congestion_score".to_owned(),
                        Value::from(40.0),
                    ),
                    (
                        "congestion_model".to_owned(),
                        Value::from("2.4ghz_adjacent_channel_v1"),
                    ),
                ]),
            )],
            HostTelemetryStream::WifiLatency => vec![HostStreamSample::new(
                stream,
                BTreeMap::from([
                    ("sample_index".to_owned(), Value::Integer(index as i64)),
                    ("probe_status".to_owned(), Value::from("ok")),
                    ("wifi_latency_host".to_owned(), Value::from("1.1.1.1")),
                    ("wifi_latency_ms".to_owned(), Value::from(8.5)),
                ]),
            )],
            HostTelemetryStream::WifiChannelOccupancy => vec![HostStreamSample::new(
                stream,
                BTreeMap::from([
                    ("sample_index".to_owned(), Value::Integer(index as i64)),
                    ("scan_status".to_owned(), Value::from("ok")),
                    ("wifi_adapter".to_owned(), Value::from("fixture-wifi")),
                    ("wifi_channel".to_owned(), Value::Integer(6)),
                    ("wifi_band".to_owned(), Value::from("2.4GHz")),
                    ("active_wifi_channel".to_owned(), Value::Integer(6)),
                    ("active_wifi_band".to_owned(), Value::from("2.4GHz")),
                    ("active_channel".to_owned(), Value::from(true)),
                    ("network_count".to_owned(), Value::Integer(4)),
                    ("max_signal_percent".to_owned(), Value::from(86.0)),
                    ("scan_network_count".to_owned(), Value::Integer(4)),
                    ("overlap_weight".to_owned(), Value::from(1.0)),
                    ("same_channel_network_count".to_owned(), Value::Integer(4)),
                    ("overlapping_network_count".to_owned(), Value::Integer(4)),
                    (
                        "overlap_weighted_network_count".to_owned(),
                        Value::from(4.0),
                    ),
                    (
                        "wifi_network_congestion_score".to_owned(),
                        Value::from(40.0),
                    ),
                    (
                        "congestion_model".to_owned(),
                        Value::from("2.4ghz_adjacent_channel_v1"),
                    ),
                ]),
            )],
            HostTelemetryStream::Bluetooth => vec![HostStreamSample::new(
                stream,
                BTreeMap::from([
                    ("sample_index".to_owned(), Value::Integer(index as i64)),
                    ("probe_status".to_owned(), Value::from("ok")),
                    (
                        "bluetooth_adapter".to_owned(),
                        Value::from("fixture-bluetooth"),
                    ),
                    ("bluetooth_adapter_count".to_owned(), Value::Integer(1)),
                    (
                        "bluetooth_connected_device_count".to_owned(),
                        Value::Integer(1),
                    ),
                    (
                        "bluetooth_present_device_count".to_owned(),
                        Value::Integer(1),
                    ),
                    (
                        "bluetooth_device_names".to_owned(),
                        Value::Array(vec![Value::from("fixture-headset")]),
                    ),
                ]),
            )],
        }
    }

    fn next_index(&mut self, stream: HostTelemetryStream) -> u64 {
        let index = self.sample_indices.entry(stream).or_default();
        *index += 1;
        *index
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
