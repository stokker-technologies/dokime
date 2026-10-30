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

/* === Constants === */

/* === Macros === */

macro_rules! host_streams {
    ($expand:ident) => {
        $expand! {
            System {
                id: "host.system",
                label: "Host System",
                columns: {
                    sample_index: Integer, Counter, "Sample Index", Option<i64>;
                    probe_status: Text, State, "Probe Status", Option<String>;
                    probe_error: Text, State, "Probe Error", Option<String>;
                    host_name: Text, State, "Host Name", Option<String>;
                    user: Text, State, "User", Option<String>;
                    git_user_name: Text, State, "Git User Name", Option<String>;
                    git_user_email: Text, State, "Git User Email", Option<String>;
                    os_name: Text, State, "OS Name", Option<String>;
                    os_version: Text, State, "OS Version", Option<String>;
                    kernel_version: Text, State, "Kernel Version", Option<String>;
                    cpu_architecture: Text, State, "CPU Architecture", Option<String>;
                    cpu_name: Text, State, "CPU Name", Option<String>;
                    cpu_brand: Text, State, "CPU Brand", Option<String>;
                    cpu_vendor_id: Text, State, "CPU Vendor", Option<String>;
                    cpu_frequency_mhz: Integer, State, "CPU Frequency", Option<i64>;
                    physical_cpu_count: Integer, State, "Physical CPU Count", Option<i64>;
                    logical_cpu_count: Integer, State, "Logical CPU Count", Option<i64>;
                    cpu_load_percent: Number, Value, "CPU Load", Option<f64>;
                    cpu_capacity_gflops: Number, Value, "Calibrated CPU Capacity (GFLOP/s)", Option<f64>;
                    cpu_load_gflops: Number, Value, "Estimated CPU Load (GFLOP/s)", Option<f64>;
                    memory_total_bytes: Integer, Value, "Memory Total", Option<i64>;
                    memory_used_bytes: Integer, Value, "Memory Used", Option<i64>;
                    memory_pressure_percent: Number, Value, "Memory Pressure", Option<f64>;
                    memory_used_gib: Number, Value, "Memory Used [GiB]", Option<f64>, unit: "GiB";
                    memory_total_gib: Number, Value, "Memory Total [GiB]", Option<f64>, unit: "GiB";
                    system_uptime_s: Number, Value, "System Uptime", Option<f64>;
                    temperature_sensor_count: Integer, Value, "Temperature Sensor Count", Option<i64>;
                    max_temperature_c: Number, Value, "Max Temperature", Option<f64>;
                    battery_percent: Number, Value, "Battery", Option<f64>;
                    battery_state: Text, State, "Battery State", Option<String>;
                    battery_power_plugged: Boolean, State, "Battery Power Plugged", Option<bool>;
                    battery_temperature_c: Number, Value, "Battery Temperature", Option<f64>;
                    battery_vendor: Text, State, "Battery Vendor", Option<String>;
                    battery_model: Text, State, "Battery Model", Option<String>;
                    battery_serial_number: Text, State, "Battery Serial Number", Option<String>;
                    battery_technology: Text, State, "Battery Technology", Option<String>;
                    battery_state_of_charge_ratio: Number, Value, "Battery State of Charge", Option<f64>;
                    battery_energy_wh: Number, Value, "Battery Energy", Option<f64>;
                    battery_energy_full_wh: Number, Value, "Battery Energy Full", Option<f64>;
                    battery_energy_full_design_wh: Number, Value, "Battery Energy Full Design", Option<f64>;
                    battery_energy_rate_w: Number, Value, "Battery Energy Rate", Option<f64>;
                    battery_voltage_v: Number, Value, "Battery Voltage", Option<f64>;
                    battery_state_of_health_ratio: Number, Value, "Battery State of Health", Option<f64>;
                    battery_time_to_full_s: Number, Value, "Battery Time to Full", Option<f64>;
                    battery_time_to_empty_s: Number, Value, "Battery Time to Empty", Option<f64>;
                    battery_cycle_count: Integer, Counter, "Battery Cycle Count", Option<i64>;
                    power_plan_name: Text, State, "Power Plan", Option<String>;
                    power_plan_guid: Text, State, "Power Plan GUID", Option<String>;
                    power_mode: Text, State, "Power Mode", Option<String>;
                    power_mode_performance_bias: Integer, Value, "Power Mode Performance Bias", Option<i64>;
                }
            }
            Network {
                id: "host.network",
                label: "Host Network",
                columns: {
                    sample_index: Integer, Counter, "Sample Index", Option<i64>;
                    probe_status: Text, State, "Probe Status", Option<String>;
                    probe_error: Text, State, "Probe Error", Option<String>;
                    primary_interface: Text, State, "Primary Interface", Option<String>;
                    change: Text, State, "Change", Option<String>;
                    adapter_id: Text, State, "Adapter", Option<String>;
                    address: Text, State, "Address", Option<String>;
                    network_interface_count: Integer, Value, "Network Interface Count", Option<i64>;
                    network_recv_mb_per_second: Number, Throughput, "Network RX", Option<f64>;
                    network_sent_mb_per_second: Number, Throughput, "Network TX", Option<f64>;
                    network_total_recv_bytes: Integer, Counter, "Network Total RX", Option<i64>;
                    network_total_sent_bytes: Integer, Counter, "Network Total TX", Option<i64>;
                    network_rx_errors: Integer, Counter, "Network RX Errors", Option<i64>;
                    network_tx_errors: Integer, Counter, "Network TX Errors", Option<i64>;
                    internet: Boolean, State, "Internet", Option<bool>;
                }
            }
            WifiLink {
                id: "host.wifi.link",
                label: "Host WiFi Link",
                columns: {
                    sample_index: Integer, Counter, "Sample Index", Option<i64>;
                    probe_status: Text, State, "Probe Status", Option<String>;
                    probe_error: Text, State, "Probe Error", Option<String>;
                    wifi_adapter: Text, State, "WiFi Adapter", Option<String>;
                    wifi_state: Text, State, "WiFi State", Option<String>;
                    associated: Boolean, State, "Associated", Option<bool>;
                    wifi_ssid: Text, State, "WiFi SSID", Option<String>;
                    wifi_bssid: Text, State, "WiFi BSSID", Option<String>;
                    wifi_profile: Text, State, "WiFi Profile", Option<String>;
                    wifi_radio: Text, State, "WiFi Radio", Option<String>;
                    wifi_auth: Text, State, "WiFi Auth", Option<String>;
                    wifi_cipher: Text, State, "WiFi Cipher", Option<String>;
                    wifi_signal_percent: Number, Value, "WiFi Signal", Option<f64>;
                    wifi_rssi_dbm: Number, Value, "WiFi RSSI", Option<f64>;
                    wifi_rx_phy_mbps: Number, Throughput, "WiFi RX PHY Rate (Mbps)", Option<f64>;
                    wifi_tx_phy_mbps: Number, Throughput, "WiFi TX PHY Rate (Mbps)", Option<f64>;
                    wifi_rx_mbps: Number, Throughput, "WiFi RX Throughput (Mbps)", Option<f64>;
                    wifi_tx_mbps: Number, Throughput, "WiFi TX Throughput (Mbps)", Option<f64>;
                    wifi_channel: Integer, State, "WiFi Channel", Option<i64>;
                    wifi_band: Text, State, "WiFi Band", Option<String>;
                }
            }
            WifiScan {
                id: "host.wifi.scan",
                label: "Host WiFi Scan",
                columns: {
                    sample_index: Integer, Counter, "Sample Index", Option<i64>;
                    scan_status: Text, State, "Scan Status", Option<String>;
                    scan_error: Text, State, "Scan Error", Option<String>;
                    wifi_adapter: Text, State, "WiFi Adapter", Option<String>;
                    wifi_ssid: Text, State, "WiFi SSID", Option<String>;
                    wifi_bssid: Text, State, "WiFi BSSID", Option<String>;
                    wifi_signal_percent: Number, Value, "WiFi Signal", Option<f64>;
                    wifi_rssi_dbm: Number, Value, "WiFi RSSI", Option<f64>;
                    wifi_channel: Integer, State, "WiFi Channel", Option<i64>;
                    wifi_band: Text, State, "WiFi Band", Option<String>;
                    wifi_security: Text, State, "WiFi Security", Option<String>;
                    associated: Boolean, State, "Associated", Option<bool>;
                    network_count: Integer, Value, "Channel Networks", Option<i64>;
                    scan_network_count: Integer, Value, "Scan Networks", Option<i64>;
                    active_wifi_channel: Integer, State, "Active WiFi Channel", Option<i64>;
                    active_wifi_band: Text, State, "Active WiFi Band", Option<String>;
                    active_wifi_ssid: Text, State, "Active WiFi SSID", Option<String>;
                    wifi_network_congestion_score: Number, Value, "WiFi Congestion", Option<f64>;
                    congestion_model: Text, State, "Congestion Model", Option<String>;
                }
            }
            WifiLatency {
                id: "host.wifi.latency",
                label: "Host WiFi Latency",
                columns: {
                    sample_index: Integer, Counter, "Sample Index", Option<i64>;
                    probe_status: Text, State, "Probe Status", Option<String>;
                    probe_error: Text, State, "Probe Error", Option<String>;
                    wifi_latency_host: Text, State, "WiFi Latency Host", Option<String>;
                    wifi_latency_ms: Number, Latency, "WiFi Latency", Option<f64>;
                    wifi_packet_loss_percent: Number, Value, "WiFi Packet Loss", Option<f64>;
                }
            }
            Bluetooth {
                id: "host.bluetooth",
                label: "Host Bluetooth",
                columns: {
                    sample_index: Integer, Counter, "Sample Index", Option<i64>;
                    probe_status: Text, State, "Probe Status", Option<String>;
                    probe_error: Text, State, "Probe Error", Option<String>;
                    bluetooth_adapter: Text, State, "Bluetooth Adapter", Option<String>;
                    bluetooth_adapter_count: Integer, Value, "Bluetooth Adapters", Option<i64>;
                    bluetooth_connected_device_count: Integer, Value, "Connected Bluetooth Devices", Option<i64>;
                    bluetooth_present_device_count: Integer, Value, "Present Bluetooth Devices", Option<i64>;
                    bluetooth_recv_mb_per_second: Number, Throughput, "Bluetooth RX", Option<f64>;
                    bluetooth_sent_mb_per_second: Number, Throughput, "Bluetooth TX", Option<f64>;
                    bluetooth_radio_state: Text, State, "Bluetooth Radio State", Option<String>;
                    bluetooth_device_names: Array, State, "Bluetooth Devices", Vec<String>;
                }
            }
        }
    };
}

pub(super) use host_streams;

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
