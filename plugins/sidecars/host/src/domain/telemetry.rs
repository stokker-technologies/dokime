/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Host Telemetry Rows With Their Labels, Units and Measurement Presence

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::domain::readings::WifiScan;

/* === Constants === */

/// Every Collection Outcome Stream and Its Label; Each Collection Publishes One Outcome per Tick.
pub const PROBES: [(&str, &str); 6] = [
    (SYSTEM_PROBE, "Host System Probe"),
    (NETWORK_PROBE, "Host Network Probe"),
    (WIFI_LINK_PROBE, "Host Wi-Fi Link Probe"),
    (WIFI_SCAN_PROBE, "Host Wi-Fi Scan Probe"),
    (WIFI_LATENCY_PROBE, "Host Wi-Fi Latency Probe"),
    (BLUETOOTH_PROBE, "Host Bluetooth Probe"),
];

pub const SYSTEM_PROBE: &str = "host.system_probe";
pub const NETWORK_PROBE: &str = "host.network_probe";
pub const WIFI_LINK_PROBE: &str = "host.wifi_link_probe";
pub const WIFI_SCAN_PROBE: &str = "host.wifi_scan_probe";
pub const WIFI_LATENCY_PROBE: &str = "host.wifi_latency_probe";
pub const BLUETOOTH_PROBE: &str = "host.bluetooth_probe";

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

/// One Collection Outcome: `ok`, or `error` With the Error Text; the Text Is Empty on Success.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct Probe {
    pub outcome: String,
    pub error: String,
}

/// The Host, Operating System and Processor Identity.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct SystemIdentity {
    pub host_name: String,
    pub user: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_architecture: String,
    pub cpu_name: String,
    pub cpu_brand: String,
    pub cpu_vendor_id: String,
    pub cpu_frequency_mhz: i64,
    pub physical_cpu_count: i64,
    pub logical_cpu_count: i64,
}

/// The Configured Git User; Silent Until Both Name and Email Are Configured.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct SystemGit {
    pub git_user_name: String,
    pub git_user_email: String,
}

/// Processor Utilization, Memory and Uptime.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct SystemResources {
    pub cpu_load_percent: f64,
    pub memory_total_bytes: i64,
    pub memory_used_bytes: i64,
    pub memory_pressure_percent: f64,
    pub memory_used_gib: f64,
    pub memory_total_gib: f64,
    pub system_uptime_s: f64,
    pub temperature_sensor_count: i64,
}

/// The Hottest Sensor; Silent Without a Readable Sensor.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct SystemTemperature {
    pub max_temperature_c: f64,
}

/// Calibrated Capacity and the Load It Implies; Silent Until Calibrated.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct CpuCapacity {
    pub cpu_capacity_gflops: f64,
    pub cpu_load_gflops: f64,
}

/// Battery Charge and Energy; Silent Without a Battery.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct Battery {
    pub battery_percent: f64,
    pub battery_state: String,
    pub battery_power_plugged: bool,
    pub battery_technology: String,
    pub battery_state_of_charge_ratio: f64,
    pub battery_energy_wh: f64,
    pub battery_energy_full_wh: f64,
    pub battery_energy_full_design_wh: f64,
    pub battery_energy_rate_w: f64,
    pub battery_voltage_v: f64,
    pub battery_state_of_health_ratio: f64,
}

/// Battery Vendor, Model and Serial Number When the Battery Reports All Three.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct BatteryIdentity {
    pub battery_vendor: String,
    pub battery_model: String,
    pub battery_serial_number: String,
}

/// Battery Temperature When the Battery Reports It.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct BatteryTemperature {
    pub battery_temperature_c: f64,
}

/// Time to Full While Charging.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct BatteryTimeToFull {
    pub battery_time_to_full_s: f64,
}

/// Time to Empty While Discharging.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct BatteryTimeToEmpty {
    pub battery_time_to_empty_s: f64,
}

/// Charge Cycles When the Battery Reports Them.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct BatteryCycles {
    pub battery_cycle_count: i64,
}

/// The Active Named Power Plan and the Mode It Selects; Silent Where the Platform Has None.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct PowerPlan {
    pub power_plan_name: String,
    pub power_plan_guid: String,
    pub power_mode: String,
    pub power_mode_performance_bias: i64,
}

/// Interface Count and Traffic Across Every Interface.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct Network {
    pub network_interface_count: i64,
    pub network_recv_mb_per_second: f64,
    pub network_sent_mb_per_second: f64,
    pub network_total_recv_bytes: i64,
    pub network_total_sent_bytes: i64,
    pub network_rx_errors: i64,
    pub network_tx_errors: i64,
}

/// The Adapter Carrying the Default Route; Silent Without One.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct NetworkPrimary {
    pub primary_interface: String,
}

/// Whether the Host Reaches the Internet; Silent When the Platform Cannot Tell.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct NetworkInternet {
    pub internet: bool,
}

/// One Row per Link or Route Change Since the Previous Collection.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct NetworkChange {
    pub change: String,
    pub adapter_id: String,
}

/// One Row per Address Added or Removed Since the Previous Collection.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct NetworkAddressChange {
    pub change: String,
    pub adapter_id: String,
    pub address: String,
}

/// The Selected Wi-Fi Adapter and Its Connection State; Silent Without an Adapter.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiAdapter {
    pub wifi_adapter: String,
    pub wifi_state: String,
    pub associated: bool,
}

/// The Associated Network; Silent While Disconnected.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiLink {
    pub wifi_ssid: String,
    pub wifi_bssid: String,
    pub wifi_channel: i64,
    pub wifi_band: String,
}

/// The Authentication of the Association; Silent While Disconnected.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiSecurity {
    pub wifi_auth: String,
}

/// The Radio Type and Cipher Where the Platform Reports Both; Silent While Disconnected.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiCipher {
    pub wifi_radio: String,
    pub wifi_cipher: String,
}

/// The Profile of the Connection; Silent Without One.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiProfile {
    pub wifi_profile: String,
}

/// Signal Quality; Silent While Disconnected.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiSignal {
    pub wifi_signal_percent: f64,
}

/// Signal Strength Where the Driver Reports It; Silent While Disconnected.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiRssi {
    pub wifi_rssi_dbm: f64,
}

/// Negotiated Receive and Transmit Rates; Silent While Disconnected.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiPhyRate {
    pub wifi_rx_phy_mbps: f64,
    pub wifi_tx_phy_mbps: f64,
}

/// Adapter Traffic Since the Previous Collection; Silent Without Its Counters.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiThroughput {
    pub wifi_rx_mbps: f64,
    pub wifi_tx_mbps: f64,
}

/// One Row per Scanned Network Reporting Its BSSID, Signal and Channel; All Rows of a Scan Share Its Timestamp.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiNetwork {
    pub wifi_bssid: String,
    pub wifi_signal_percent: f64,
    pub wifi_channel: i64,
    pub wifi_band: String,
    pub associated: bool,
}

/// The SSID and Security of Each Scanned Network Broadcasting Both.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiNetworkName {
    pub wifi_bssid: String,
    pub wifi_ssid: String,
    pub wifi_security: String,
}

/// One Channel of the Adapter's Full Band Plan, Including Zero Occupancy.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiChannels {
    pub sample_index: i64,
    pub wifi_band: String,
    pub wifi_channel: i64,
    pub network_count: i64,
    pub occupancy: f64,
}

/// The Signal Strength of Each Scanned Network Where the Driver Reports It.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiNetworkRssi {
    pub wifi_bssid: String,
    pub wifi_rssi_dbm: f64,
}

/// One Scan's Congestion; Association and Model Inputs May Be Unavailable.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiCongestion {
    pub scan_network_count: Option<i64>,
    pub active_wifi_channel: Option<i64>,
    pub active_wifi_band: Option<String>,
    pub active_wifi_ssid: Option<String>,
    pub wifi_network_congestion_score: Option<f64>,
    pub congestion_model: Option<String>,
}

/// Echo Round Trip and Loss; Silent When the Probe Cannot Run or the Reply Is Lost.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiLatency {
    pub wifi_latency_host: String,
    pub wifi_latency_ms: f64,
}

/// Loss of Each Echo That Ran: 0 for a Reply, 100 for a Lost One.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct WifiLoss {
    pub wifi_latency_host: String,
    pub wifi_packet_loss_percent: f64,
}

/// Present Devices and the Radio State.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct Bluetooth {
    pub bluetooth_present_device_count: i64,
    pub bluetooth_radio_state: String,
}

/// The Number of Bluetooth Radios; Silent When They Cannot Be Read.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct BluetoothRadios {
    pub bluetooth_adapter_count: i64,
}

/// Connected Devices; Silent Where the Platform Does Not Report Them.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct BluetoothConnections {
    pub bluetooth_connected_device_count: i64,
}

/// Bluetooth Traffic; Silent Where the Platform Does Not Report It.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct BluetoothTraffic {
    pub bluetooth_recv_mb_per_second: f64,
    pub bluetooth_sent_mb_per_second: f64,
}

/// One Row per Present Device; All Rows of an Inventory Share Its Timestamp.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct BluetoothDevice {
    pub bluetooth_device_name: String,
}

/// Measured Closed-Loop Utilization per Controller Period of a Percent Hold.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct LoadPercentControl {
    pub target_percent: f64,
    pub measured_percent: f64,
    pub duty_percent: f64,
    pub generator_flops: f64,
}

/// Measured Closed-Loop Throughput per Controller Period of a FLOP/s Hold.
#[derive(Clone, Debug, PartialEq, dokime::Row)]
#[dokime(package = "host")]
pub struct LoadFlopsControl {
    pub target_flops: f64,
    pub measured_flops: f64,
    pub duty_percent: f64,
    pub generator_flops: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<WifiScan> for WifiCongestion {
    fn from(reading: WifiScan) -> Self {
        Self {
            scan_network_count: reading.scan_network_count,
            active_wifi_channel: reading.active_wifi_channel,
            active_wifi_band: reading.active_wifi_band,
            active_wifi_ssid: reading.active_wifi_ssid,
            wifi_network_congestion_score: reading.wifi_network_congestion_score,
            congestion_model: reading.congestion_model,
        }
    }
}

impl Column {
    pub const fn new(field: &'static str, label: &'static str, unit: &'static str) -> Self {
        Self { field, label, unit }
    }

    /// The Label With Its Unit in Square Brackets, as Plots and Tables Show It.
    pub fn title(&self) -> String {
        if self.unit.is_empty() { self.label.to_owned() } else { format!("{} [{}]", self.label, self.unit) }
    }
}

impl Probe {
    pub const COLUMNS: &'static [Column] = &[Column::new("outcome", "Outcome", ""), Column::new("error", "Error", "")];
}

impl SystemIdentity {
    pub const STREAM: &'static str = "host.system_identity";
    pub const LABEL: &'static str = "Host System Identity";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("host_name", "Host Name", ""),
        Column::new("user", "User", ""),
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
    ];
}

impl SystemGit {
    pub const STREAM: &'static str = "host.system_git";
    pub const LABEL: &'static str = "Host Git Identity";
    pub const COLUMNS: &'static [Column] =
        &[Column::new("git_user_name", "Git User Name", ""), Column::new("git_user_email", "Git User Email", "")];
}

impl SystemResources {
    pub const STREAM: &'static str = "host.system";
    pub const LABEL: &'static str = "Host System";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("cpu_load_percent", "CPU Load", "%"),
        Column::new("memory_total_bytes", "Memory Total", "B"),
        Column::new("memory_used_bytes", "Memory Used", "B"),
        Column::new("memory_pressure_percent", "Memory Pressure", "%"),
        Column::new("memory_used_gib", "Memory Used", "GiB"),
        Column::new("memory_total_gib", "Memory Total", "GiB"),
        Column::new("system_uptime_s", "System Uptime", "s"),
        Column::new("temperature_sensor_count", "Temperature Sensor Count", ""),
    ];
}

impl SystemTemperature {
    pub const STREAM: &'static str = "host.system_temperature";
    pub const LABEL: &'static str = "Host Temperature";
    pub const COLUMNS: &'static [Column] = &[Column::new("max_temperature_c", "Max Temperature", "°C")];
}

impl CpuCapacity {
    pub const STREAM: &'static str = "host.cpu_capacity";
    pub const LABEL: &'static str = "Host CPU Capacity";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("cpu_capacity_gflops", "Calibrated CPU Capacity", "GFLOP/s"),
        Column::new("cpu_load_gflops", "Estimated CPU Load", "GFLOP/s"),
    ];
}

impl Battery {
    pub const STREAM: &'static str = "host.battery";
    pub const LABEL: &'static str = "Host Battery";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("battery_percent", "Battery", "%"),
        Column::new("battery_state", "Battery State", ""),
        Column::new("battery_power_plugged", "Battery Power Plugged", ""),
        Column::new("battery_technology", "Battery Technology", ""),
        Column::new("battery_state_of_charge_ratio", "Battery State of Charge", ""),
        Column::new("battery_energy_wh", "Battery Energy", "Wh"),
        Column::new("battery_energy_full_wh", "Battery Energy Full", "Wh"),
        Column::new("battery_energy_full_design_wh", "Battery Energy Full Design", "Wh"),
        Column::new("battery_energy_rate_w", "Battery Energy Rate", "W"),
        Column::new("battery_voltage_v", "Battery Voltage", "V"),
        Column::new("battery_state_of_health_ratio", "Battery State of Health", ""),
    ];
}

impl BatteryIdentity {
    pub const STREAM: &'static str = "host.battery_identity";
    pub const LABEL: &'static str = "Host Battery Identity";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("battery_vendor", "Battery Vendor", ""),
        Column::new("battery_model", "Battery Model", ""),
        Column::new("battery_serial_number", "Battery Serial Number", ""),
    ];
}

impl BatteryTemperature {
    pub const STREAM: &'static str = "host.battery_temperature";
    pub const LABEL: &'static str = "Host Battery Temperature";
    pub const COLUMNS: &'static [Column] = &[Column::new("battery_temperature_c", "Battery Temperature", "°C")];
}

impl BatteryTimeToFull {
    pub const STREAM: &'static str = "host.battery_time_to_full";
    pub const LABEL: &'static str = "Host Battery Time to Full";
    pub const COLUMNS: &'static [Column] = &[Column::new("battery_time_to_full_s", "Battery Time to Full", "s")];
}

impl BatteryTimeToEmpty {
    pub const STREAM: &'static str = "host.battery_time_to_empty";
    pub const LABEL: &'static str = "Host Battery Time to Empty";
    pub const COLUMNS: &'static [Column] = &[Column::new("battery_time_to_empty_s", "Battery Time to Empty", "s")];
}

impl BatteryCycles {
    pub const STREAM: &'static str = "host.battery_cycles";
    pub const LABEL: &'static str = "Host Battery Cycles";
    pub const COLUMNS: &'static [Column] = &[Column::new("battery_cycle_count", "Battery Cycle Count", "")];
}

impl PowerPlan {
    pub const STREAM: &'static str = "host.power_plan";
    pub const LABEL: &'static str = "Host Power Plan";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("power_plan_name", "Power Plan", ""),
        Column::new("power_plan_guid", "Power Plan GUID", ""),
        Column::new("power_mode", "Power Mode", ""),
        Column::new("power_mode_performance_bias", "Power Mode Performance Bias", ""),
    ];
}

impl Network {
    pub const STREAM: &'static str = "host.network";
    pub const LABEL: &'static str = "Host Network";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("network_interface_count", "Network Interface Count", ""),
        Column::new("network_recv_mb_per_second", "Network RX", "MB/s"),
        Column::new("network_sent_mb_per_second", "Network TX", "MB/s"),
        Column::new("network_total_recv_bytes", "Network Total RX", "B"),
        Column::new("network_total_sent_bytes", "Network Total TX", "B"),
        Column::new("network_rx_errors", "Network RX Errors", ""),
        Column::new("network_tx_errors", "Network TX Errors", ""),
    ];
}

impl NetworkPrimary {
    pub const STREAM: &'static str = "host.network_primary";
    pub const LABEL: &'static str = "Host Primary Interface";
    pub const COLUMNS: &'static [Column] = &[Column::new("primary_interface", "Primary Interface", "")];
}

impl NetworkInternet {
    pub const STREAM: &'static str = "host.network_internet";
    pub const LABEL: &'static str = "Host Internet";
    pub const COLUMNS: &'static [Column] = &[Column::new("internet", "Internet", "")];
}

impl NetworkChange {
    pub const STREAM: &'static str = "host.network_change";
    pub const LABEL: &'static str = "Host Network Change";
    pub const COLUMNS: &'static [Column] = &[Column::new("change", "Change", ""), Column::new("adapter_id", "Adapter", "")];
}

impl NetworkAddressChange {
    pub const STREAM: &'static str = "host.network_address_change";
    pub const LABEL: &'static str = "Host Address Change";
    pub const COLUMNS: &'static [Column] =
        &[Column::new("change", "Change", ""), Column::new("adapter_id", "Adapter", ""), Column::new("address", "Address", "")];
}

impl WifiAdapter {
    pub const STREAM: &'static str = "host.wifi_adapter";
    pub const LABEL: &'static str = "Host Wi-Fi Adapter";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("wifi_adapter", "Wi-Fi Adapter", ""),
        Column::new("wifi_state", "Wi-Fi State", ""),
        Column::new("associated", "Associated", ""),
    ];
}

impl WifiLink {
    pub const STREAM: &'static str = "host.wifi_link";
    pub const LABEL: &'static str = "Host Wi-Fi Link";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("wifi_ssid", "Wi-Fi SSID", ""),
        Column::new("wifi_bssid", "Wi-Fi BSSID", ""),
        Column::new("wifi_channel", "Wi-Fi Channel", ""),
        Column::new("wifi_band", "Wi-Fi Band", ""),
    ];
}

impl WifiSecurity {
    pub const STREAM: &'static str = "host.wifi_security";
    pub const LABEL: &'static str = "Host Wi-Fi Security";
    pub const COLUMNS: &'static [Column] = &[Column::new("wifi_auth", "Wi-Fi Auth", "")];
}

impl WifiCipher {
    pub const STREAM: &'static str = "host.wifi_cipher";
    pub const LABEL: &'static str = "Host Wi-Fi Radio and Cipher";
    pub const COLUMNS: &'static [Column] = &[Column::new("wifi_radio", "Wi-Fi Radio", ""), Column::new("wifi_cipher", "Wi-Fi Cipher", "")];
}

impl WifiProfile {
    pub const STREAM: &'static str = "host.wifi_profile";
    pub const LABEL: &'static str = "Host Wi-Fi Profile";
    pub const COLUMNS: &'static [Column] = &[Column::new("wifi_profile", "Wi-Fi Profile", "")];
}

impl WifiSignal {
    pub const STREAM: &'static str = "host.wifi_signal";
    pub const LABEL: &'static str = "Host Wi-Fi Signal";
    pub const COLUMNS: &'static [Column] = &[Column::new("wifi_signal_percent", "Wi-Fi Signal", "%")];
}

impl WifiRssi {
    pub const STREAM: &'static str = "host.wifi_rssi";
    pub const LABEL: &'static str = "Host Wi-Fi RSSI";
    pub const COLUMNS: &'static [Column] = &[Column::new("wifi_rssi_dbm", "Wi-Fi RSSI", "dBm")];
}

impl WifiPhyRate {
    pub const STREAM: &'static str = "host.wifi_phy_rate";
    pub const LABEL: &'static str = "Host Wi-Fi PHY Rate";
    pub const COLUMNS: &'static [Column] =
        &[Column::new("wifi_rx_phy_mbps", "Wi-Fi RX PHY Rate", "Mbps"), Column::new("wifi_tx_phy_mbps", "Wi-Fi TX PHY Rate", "Mbps")];
}

impl WifiThroughput {
    pub const STREAM: &'static str = "host.wifi_throughput";
    pub const LABEL: &'static str = "Host Wi-Fi Throughput";
    pub const COLUMNS: &'static [Column] =
        &[Column::new("wifi_rx_mbps", "Wi-Fi RX Throughput", "Mbps"), Column::new("wifi_tx_mbps", "Wi-Fi TX Throughput", "Mbps")];
}

impl WifiNetwork {
    pub const STREAM: &'static str = "host.wifi_network";
    pub const LABEL: &'static str = "Host Wi-Fi Network";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("wifi_bssid", "Wi-Fi BSSID", ""),
        Column::new("wifi_signal_percent", "Wi-Fi Signal", "%"),
        Column::new("wifi_channel", "Wi-Fi Channel", ""),
        Column::new("wifi_band", "Wi-Fi Band", ""),
        Column::new("associated", "Associated", ""),
    ];
}

impl WifiNetworkName {
    pub const STREAM: &'static str = "host.wifi_network_name";
    pub const LABEL: &'static str = "Host Wi-Fi Network Name";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("wifi_bssid", "Wi-Fi BSSID", ""),
        Column::new("wifi_ssid", "Wi-Fi SSID", ""),
        Column::new("wifi_security", "Wi-Fi Security", ""),
    ];
}

impl WifiChannels {
    pub const STREAM: &'static str = "host.wifi_channels";
    pub const LABEL: &'static str = "Host Wi-Fi Channels";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("sample_index", "Sample Index", ""),
        Column::new("wifi_band", "Wi-Fi Band", ""),
        Column::new("wifi_channel", "Wi-Fi Channel", ""),
        Column::new("network_count", "Primary Channel Networks", ""),
        Column::new("occupancy", "Overlap-Weighted Occupancy", ""),
    ];
}

impl WifiNetworkRssi {
    pub const STREAM: &'static str = "host.wifi_network_rssi";
    pub const LABEL: &'static str = "Host Wi-Fi Network RSSI";
    pub const COLUMNS: &'static [Column] =
        &[Column::new("wifi_bssid", "Wi-Fi BSSID", ""), Column::new("wifi_rssi_dbm", "Wi-Fi RSSI", "dBm")];
}

impl WifiCongestion {
    pub const STREAM: &'static str = "host.wifi_congestion";
    pub const LABEL: &'static str = "Host Wi-Fi Congestion";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("scan_network_count", "Scan Networks", ""),
        Column::new("active_wifi_channel", "Active Wi-Fi Channel", ""),
        Column::new("active_wifi_band", "Active Wi-Fi Band", ""),
        Column::new("active_wifi_ssid", "Active Wi-Fi SSID", ""),
        Column::new("wifi_network_congestion_score", "Wi-Fi Congestion", ""),
        Column::new("congestion_model", "Congestion Model", ""),
    ];
}

impl WifiLatency {
    pub const STREAM: &'static str = "host.wifi_latency";
    pub const LABEL: &'static str = "Host Wi-Fi Latency";
    pub const COLUMNS: &'static [Column] =
        &[Column::new("wifi_latency_host", "Wi-Fi Latency Host", ""), Column::new("wifi_latency_ms", "Wi-Fi Latency", "ms")];
}

impl WifiLoss {
    pub const STREAM: &'static str = "host.wifi_loss";
    pub const LABEL: &'static str = "Host Wi-Fi Packet Loss";
    pub const COLUMNS: &'static [Column] =
        &[Column::new("wifi_latency_host", "Wi-Fi Latency Host", ""), Column::new("wifi_packet_loss_percent", "Wi-Fi Packet Loss", "%")];
}

impl Bluetooth {
    pub const STREAM: &'static str = "host.bluetooth";
    pub const LABEL: &'static str = "Host Bluetooth";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("bluetooth_present_device_count", "Present Bluetooth Devices", ""),
        Column::new("bluetooth_radio_state", "Bluetooth Radio State", ""),
    ];
}

impl BluetoothRadios {
    pub const STREAM: &'static str = "host.bluetooth_radios";
    pub const LABEL: &'static str = "Host Bluetooth Radios";
    pub const COLUMNS: &'static [Column] = &[Column::new("bluetooth_adapter_count", "Bluetooth Adapters", "")];
}

impl BluetoothConnections {
    pub const STREAM: &'static str = "host.bluetooth_connections";
    pub const LABEL: &'static str = "Host Bluetooth Connections";
    pub const COLUMNS: &'static [Column] = &[Column::new("bluetooth_connected_device_count", "Connected Bluetooth Devices", "")];
}

impl BluetoothTraffic {
    pub const STREAM: &'static str = "host.bluetooth_traffic";
    pub const LABEL: &'static str = "Host Bluetooth Traffic";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("bluetooth_recv_mb_per_second", "Bluetooth RX", "MB/s"),
        Column::new("bluetooth_sent_mb_per_second", "Bluetooth TX", "MB/s"),
    ];
}

impl BluetoothDevice {
    pub const STREAM: &'static str = "host.bluetooth_device";
    pub const LABEL: &'static str = "Host Bluetooth Device";
    pub const COLUMNS: &'static [Column] = &[Column::new("bluetooth_device_name", "Bluetooth Device", "")];
}

impl LoadPercentControl {
    pub const STREAM: &'static str = "host.cpu_load_percent_control";
    pub const LABEL: &'static str = "CPU Load Control";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("target_percent", "Target", "%"),
        Column::new("measured_percent", "Measured", "%"),
        Column::new("duty_percent", "Duty", "%"),
        Column::new("generator_flops", "Generator", "FLOP/s"),
    ];
}

impl LoadFlopsControl {
    pub const STREAM: &'static str = "host.cpu_load_flops_control";
    pub const LABEL: &'static str = "CPU Throughput Control";
    pub const COLUMNS: &'static [Column] = &[
        Column::new("target_flops", "Target", "FLOP/s"),
        Column::new("measured_flops", "Measured", "FLOP/s"),
        Column::new("duty_percent", "Duty", "%"),
        Column::new("generator_flops", "Generator", "FLOP/s"),
    ];
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
