/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Telemetry Rows: One Row Type per Host Stream, Their Column Labels and Units

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/// Every Host Stream Identifier, in Catalog Order.
pub const STREAMS: [&str; 6] =
    [System::STREAM, Network::STREAM, WifiLink::STREAM, WifiScan::STREAM, WifiLatency::STREAM, Bluetooth::STREAM];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Row Field With Its Human-Facing Label and Unit Symbol; an Empty Unit Is Dimensionless or Textual.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Column {
    pub field: &'static str,
    pub label: &'static str,
    pub unit: &'static str,
}

/// Host System Telemetry Row.
#[derive(dokime::Row)]
#[dokime(package = "host")]
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

/// Host Network Telemetry Row.
#[derive(dokime::Row)]
#[dokime(package = "host")]
pub struct Network {
    pub sample_index: Option<i64>,
    pub probe_status: Option<String>,
    pub probe_error: Option<String>,
    pub primary_interface: Option<String>,
    pub change: Option<String>,
    pub adapter_id: Option<String>,
    pub address: Option<String>,
    pub network_interface_count: Option<i64>,
    pub network_recv_mb_per_second: Option<f64>,
    pub network_sent_mb_per_second: Option<f64>,
    pub network_total_recv_bytes: Option<i64>,
    pub network_total_sent_bytes: Option<i64>,
    pub network_rx_errors: Option<i64>,
    pub network_tx_errors: Option<i64>,
    pub internet: Option<bool>,
}

/// Host Wi-Fi Link Telemetry Row.
#[derive(dokime::Row)]
#[dokime(package = "host")]
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

/// Host Wi-Fi Scan Telemetry Row.
#[derive(dokime::Row)]
#[dokime(package = "host")]
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

/// Host Wi-Fi Latency Telemetry Row.
#[derive(dokime::Row)]
#[dokime(package = "host")]
pub struct WifiLatency {
    pub sample_index: Option<i64>,
    pub probe_status: Option<String>,
    pub probe_error: Option<String>,
    pub wifi_latency_host: Option<String>,
    pub wifi_latency_ms: Option<f64>,
    pub wifi_packet_loss_percent: Option<f64>,
}

/// Host Bluetooth Telemetry Row.
#[derive(dokime::Row)]
#[dokime(package = "host")]
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

impl Column {
    pub const fn new(field: &'static str, label: &'static str, unit: &'static str) -> Self {
        Self { field, label, unit }
    }

    /// The Label With Its Unit in Square Brackets, as Plots and Tables Show It.
    pub fn title(&self) -> String {
        if self.unit.is_empty() { self.label.to_owned() } else { format!("{} [{}]", self.label, self.unit) }
    }
}

impl System {
    pub const STREAM: &'static str = "system";
    pub const LABEL: &'static str = "Host System";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("sample_index", "Sample Index", ""),
        Column::new("probe_status", "Probe Status", ""),
        Column::new("probe_error", "Probe Error", ""),
        Column::new("host_name", "Host Name", ""),
        Column::new("user", "User", ""),
        Column::new("git_user_name", "Git User Name", ""),
        Column::new("git_user_email", "Git User Email", ""),
        Column::new("os_name", "OS Name", ""),
        Column::new("os_version", "OS Version", ""),
        Column::new("kernel_version", "Kernel Version", ""),
        Column::new("cpu_architecture", "CPU Architecture", ""),
        Column::new("cpu_name", "CPU Name", ""),
        Column::new("cpu_brand", "CPU Brand", ""),
        Column::new("cpu_vendor_id", "CPU Vendor", ""),
        Column::new("cpu_frequency_mhz", "CPU Frequency", "MHz"),
        Column::new("physical_cpu_count", "Physical CPU Count", ""),
        Column::new("logical_cpu_count", "Logical CPU Count", ""),
        Column::new("cpu_load_percent", "CPU Load", "%"),
        Column::new("cpu_capacity_gflops", "Calibrated CPU Capacity", "GFLOP/s"),
        Column::new("cpu_load_gflops", "Estimated CPU Load", "GFLOP/s"),
        Column::new("memory_total_bytes", "Memory Total", "B"),
        Column::new("memory_used_bytes", "Memory Used", "B"),
        Column::new("memory_pressure_percent", "Memory Pressure", "%"),
        Column::new("memory_used_gib", "Memory Used", "GiB"),
        Column::new("memory_total_gib", "Memory Total", "GiB"),
        Column::new("system_uptime_s", "System Uptime", "s"),
        Column::new("temperature_sensor_count", "Temperature Sensor Count", ""),
        Column::new("max_temperature_c", "Max Temperature", "°C"),
        Column::new("battery_percent", "Battery", "%"),
        Column::new("battery_state", "Battery State", ""),
        Column::new("battery_power_plugged", "Battery Power Plugged", ""),
        Column::new("battery_temperature_c", "Battery Temperature", "°C"),
        Column::new("battery_vendor", "Battery Vendor", ""),
        Column::new("battery_model", "Battery Model", ""),
        Column::new("battery_serial_number", "Battery Serial Number", ""),
        Column::new("battery_technology", "Battery Technology", ""),
        Column::new("battery_state_of_charge_ratio", "Battery State of Charge", ""),
        Column::new("battery_energy_wh", "Battery Energy", "Wh"),
        Column::new("battery_energy_full_wh", "Battery Energy Full", "Wh"),
        Column::new("battery_energy_full_design_wh", "Battery Energy Full Design", "Wh"),
        Column::new("battery_energy_rate_w", "Battery Energy Rate", "W"),
        Column::new("battery_voltage_v", "Battery Voltage", "V"),
        Column::new("battery_state_of_health_ratio", "Battery State of Health", ""),
        Column::new("battery_time_to_full_s", "Battery Time to Full", "s"),
        Column::new("battery_time_to_empty_s", "Battery Time to Empty", "s"),
        Column::new("battery_cycle_count", "Battery Cycle Count", ""),
        Column::new("power_plan_name", "Power Plan", ""),
        Column::new("power_plan_guid", "Power Plan GUID", ""),
        Column::new("power_mode", "Power Mode", ""),
        Column::new("power_mode_performance_bias", "Power Mode Performance Bias", ""),
    ];
}

impl Network {
    pub const STREAM: &'static str = "network";
    pub const LABEL: &'static str = "Host Network";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("sample_index", "Sample Index", ""),
        Column::new("probe_status", "Probe Status", ""),
        Column::new("probe_error", "Probe Error", ""),
        Column::new("primary_interface", "Primary Interface", ""),
        Column::new("change", "Change", ""),
        Column::new("adapter_id", "Adapter", ""),
        Column::new("address", "Address", ""),
        Column::new("network_interface_count", "Network Interface Count", ""),
        Column::new("network_recv_mb_per_second", "Network RX", "MB/s"),
        Column::new("network_sent_mb_per_second", "Network TX", "MB/s"),
        Column::new("network_total_recv_bytes", "Network Total RX", "B"),
        Column::new("network_total_sent_bytes", "Network Total TX", "B"),
        Column::new("network_rx_errors", "Network RX Errors", ""),
        Column::new("network_tx_errors", "Network TX Errors", ""),
        Column::new("internet", "Internet", ""),
    ];
}

impl WifiLink {
    pub const STREAM: &'static str = "wifi_link";
    pub const LABEL: &'static str = "Host Wi-Fi Link";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("sample_index", "Sample Index", ""),
        Column::new("probe_status", "Probe Status", ""),
        Column::new("probe_error", "Probe Error", ""),
        Column::new("wifi_adapter", "Wi-Fi Adapter", ""),
        Column::new("wifi_state", "Wi-Fi State", ""),
        Column::new("associated", "Associated", ""),
        Column::new("wifi_ssid", "Wi-Fi SSID", ""),
        Column::new("wifi_bssid", "Wi-Fi BSSID", ""),
        Column::new("wifi_profile", "Wi-Fi Profile", ""),
        Column::new("wifi_radio", "Wi-Fi Radio", ""),
        Column::new("wifi_auth", "Wi-Fi Auth", ""),
        Column::new("wifi_cipher", "Wi-Fi Cipher", ""),
        Column::new("wifi_signal_percent", "Wi-Fi Signal", "%"),
        Column::new("wifi_rssi_dbm", "Wi-Fi RSSI", "dBm"),
        Column::new("wifi_rx_phy_mbps", "Wi-Fi RX PHY Rate", "Mbps"),
        Column::new("wifi_tx_phy_mbps", "Wi-Fi TX PHY Rate", "Mbps"),
        Column::new("wifi_rx_mbps", "Wi-Fi RX Throughput", "Mbps"),
        Column::new("wifi_tx_mbps", "Wi-Fi TX Throughput", "Mbps"),
        Column::new("wifi_channel", "Wi-Fi Channel", ""),
        Column::new("wifi_band", "Wi-Fi Band", ""),
    ];
}

impl WifiScan {
    pub const STREAM: &'static str = "wifi_scan";
    pub const LABEL: &'static str = "Host Wi-Fi Scan";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("sample_index", "Sample Index", ""),
        Column::new("scan_status", "Scan Status", ""),
        Column::new("scan_error", "Scan Error", ""),
        Column::new("wifi_adapter", "Wi-Fi Adapter", ""),
        Column::new("wifi_ssid", "Wi-Fi SSID", ""),
        Column::new("wifi_bssid", "Wi-Fi BSSID", ""),
        Column::new("wifi_signal_percent", "Wi-Fi Signal", "%"),
        Column::new("wifi_rssi_dbm", "Wi-Fi RSSI", "dBm"),
        Column::new("wifi_channel", "Wi-Fi Channel", ""),
        Column::new("wifi_band", "Wi-Fi Band", ""),
        Column::new("wifi_security", "Wi-Fi Security", ""),
        Column::new("associated", "Associated", ""),
        Column::new("network_count", "Channel Networks", ""),
        Column::new("scan_network_count", "Scan Networks", ""),
        Column::new("active_wifi_channel", "Active Wi-Fi Channel", ""),
        Column::new("active_wifi_band", "Active Wi-Fi Band", ""),
        Column::new("active_wifi_ssid", "Active Wi-Fi SSID", ""),
        Column::new("wifi_network_congestion_score", "Wi-Fi Congestion", ""),
        Column::new("congestion_model", "Congestion Model", ""),
    ];
}

impl WifiLatency {
    pub const STREAM: &'static str = "wifi_latency";
    pub const LABEL: &'static str = "Host Wi-Fi Latency";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("sample_index", "Sample Index", ""),
        Column::new("probe_status", "Probe Status", ""),
        Column::new("probe_error", "Probe Error", ""),
        Column::new("wifi_latency_host", "Wi-Fi Latency Host", ""),
        Column::new("wifi_latency_ms", "Wi-Fi Latency", "ms"),
        Column::new("wifi_packet_loss_percent", "Wi-Fi Packet Loss", "%"),
    ];
}

impl Bluetooth {
    pub const STREAM: &'static str = "bluetooth";
    pub const LABEL: &'static str = "Host Bluetooth";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("sample_index", "Sample Index", ""),
        Column::new("probe_status", "Probe Status", ""),
        Column::new("probe_error", "Probe Error", ""),
        Column::new("bluetooth_adapter", "Bluetooth Adapter", ""),
        Column::new("bluetooth_adapter_count", "Bluetooth Adapters", ""),
        Column::new("bluetooth_connected_device_count", "Connected Bluetooth Devices", ""),
        Column::new("bluetooth_present_device_count", "Present Bluetooth Devices", ""),
        Column::new("bluetooth_recv_mb_per_second", "Bluetooth RX", "MB/s"),
        Column::new("bluetooth_sent_mb_per_second", "Bluetooth TX", "MB/s"),
        Column::new("bluetooth_radio_state", "Bluetooth Radio State", ""),
        Column::new("bluetooth_device_names", "Bluetooth Devices", ""),
    ];
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
