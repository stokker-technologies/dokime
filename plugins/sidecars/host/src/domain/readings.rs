/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Readings: What One Collection Read, Before Its Fields Fan Out to the Streams Where They Exist Together

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::collectors::network::Change;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One System Collection, Every Field Present Only When Read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct System {
    pub sample_index: Option<i64>,
    pub probe_status: Option<String>,
    pub probe_error: Option<String>,
    pub host_name: Option<String>,
    pub user: Option<String>,
    pub git_user_name: Option<String>,
    pub git_user_email: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub cpu_architecture: Option<String>,
    pub cpu_name: Option<String>,
    pub cpu_brand: Option<String>,
    pub cpu_vendor_id: Option<String>,
    pub cpu_frequency_mhz: Option<i64>,
    pub physical_cpu_count: Option<i64>,
    pub logical_cpu_count: Option<i64>,
    pub cpu_load_percent: Option<f64>,
    pub cpu_capacity_gflops: Option<f64>,
    pub cpu_load_gflops: Option<f64>,
    pub memory_total_bytes: Option<i64>,
    pub memory_used_bytes: Option<i64>,
    pub memory_pressure_percent: Option<f64>,
    pub memory_used_gib: Option<f64>,
    pub memory_total_gib: Option<f64>,
    pub system_uptime_s: Option<f64>,
    pub temperature_sensor_count: Option<i64>,
    pub max_temperature_c: Option<f64>,
    pub battery_percent: Option<f64>,
    pub battery_state: Option<String>,
    pub battery_power_plugged: Option<bool>,
    pub battery_temperature_c: Option<f64>,
    pub battery_vendor: Option<String>,
    pub battery_model: Option<String>,
    pub battery_serial_number: Option<String>,
    pub battery_technology: Option<String>,
    pub battery_state_of_charge_ratio: Option<f64>,
    pub battery_energy_wh: Option<f64>,
    pub battery_energy_full_wh: Option<f64>,
    pub battery_energy_full_design_wh: Option<f64>,
    pub battery_energy_rate_w: Option<f64>,
    pub battery_voltage_v: Option<f64>,
    pub battery_state_of_health_ratio: Option<f64>,
    pub battery_time_to_full_s: Option<f64>,
    pub battery_time_to_empty_s: Option<f64>,
    pub battery_cycle_count: Option<i64>,
    pub power_plan_name: Option<String>,
    pub power_plan_guid: Option<String>,
    pub power_mode: Option<String>,
    pub power_mode_performance_bias: Option<i64>,
}

/// One Network Collection, Every Field Present Only When Read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Network {
    pub sample_index: Option<i64>,
    pub probe_status: Option<String>,
    pub probe_error: Option<String>,
    pub primary_interface: Option<String>,
    pub changes: Vec<Change>,
    pub network_interface_count: Option<i64>,
    pub network_recv_mb_per_second: Option<f64>,
    pub network_sent_mb_per_second: Option<f64>,
    pub network_total_recv_bytes: Option<i64>,
    pub network_total_sent_bytes: Option<i64>,
    pub network_rx_errors: Option<i64>,
    pub network_tx_errors: Option<i64>,
    pub internet: Option<bool>,
}

/// One Wi-Fi Link Collection, Every Field Present Only When Read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WifiLink {
    pub sample_index: Option<i64>,
    pub probe_status: Option<String>,
    pub probe_error: Option<String>,
    pub wifi_adapter: Option<String>,
    pub wifi_state: Option<String>,
    pub associated: Option<bool>,
    pub wifi_ssid: Option<String>,
    pub wifi_bssid: Option<String>,
    pub wifi_profile: Option<String>,
    pub wifi_radio: Option<String>,
    pub wifi_auth: Option<String>,
    pub wifi_cipher: Option<String>,
    pub wifi_signal_percent: Option<f64>,
    pub wifi_rssi_dbm: Option<f64>,
    pub wifi_rx_phy_mbps: Option<f64>,
    pub wifi_tx_phy_mbps: Option<f64>,
    pub wifi_rx_mbps: Option<f64>,
    pub wifi_tx_mbps: Option<f64>,
    pub wifi_channel: Option<i64>,
    pub wifi_band: Option<String>,
}

/// One Wi-Fi Scan Collection, Every Field Present Only When Read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WifiScan {
    pub sample_index: Option<i64>,
    pub scan_status: Option<String>,
    pub scan_error: Option<String>,
    pub wifi_adapter: Option<String>,
    pub wifi_ssid: Option<String>,
    pub wifi_bssid: Option<String>,
    pub wifi_signal_percent: Option<f64>,
    pub wifi_rssi_dbm: Option<f64>,
    pub wifi_channel: Option<i64>,
    pub wifi_band: Option<String>,
    pub wifi_security: Option<String>,
    pub associated: Option<bool>,
    pub network_count: Option<i64>,
    pub scan_network_count: Option<i64>,
    pub active_wifi_channel: Option<i64>,
    pub active_wifi_band: Option<String>,
    pub active_wifi_ssid: Option<String>,
    pub wifi_network_congestion_score: Option<f64>,
    pub congestion_model: Option<String>,
}

/// One Wi-Fi Latency Collection, Every Field Present Only When Read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WifiLatency {
    pub sample_index: Option<i64>,
    pub probe_status: Option<String>,
    pub probe_error: Option<String>,
    pub wifi_latency_host: Option<String>,
    pub wifi_latency_ms: Option<f64>,
    pub wifi_packet_loss_percent: Option<f64>,
}

/// One Bluetooth Collection, Every Field Present Only When Read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bluetooth {
    pub sample_index: Option<i64>,
    pub probe_status: Option<String>,
    pub probe_error: Option<String>,
    pub bluetooth_adapter: Option<String>,
    pub bluetooth_adapter_count: Option<i64>,
    pub bluetooth_connected_device_count: Option<i64>,
    pub bluetooth_present_device_count: Option<i64>,
    pub bluetooth_recv_mb_per_second: Option<f64>,
    pub bluetooth_sent_mb_per_second: Option<f64>,
    pub bluetooth_radio_state: Option<String>,
    pub bluetooth_device_names: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
