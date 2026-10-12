# ***
#
# Dokime
#
# SPDX-License-Identifier: Apache-2.0
#
# ***

from dataclasses import dataclass
from datetime import datetime
from typing import ClassVar

@dataclass(frozen=True)
class _Stream:
    stream: str

@dataclass(frozen=True)
class Bluetooth:
    __dokime_stream__: ClassVar[_Stream]
    sample_index: int | None = None
    probe_status: str | None = None
    probe_error: str | None = None
    bluetooth_adapter: str | None = None
    bluetooth_adapter_count: int | None = None
    bluetooth_connected_device_count: int | None = None
    bluetooth_present_device_count: int | None = None
    bluetooth_recv_mb_per_second: float | None = None
    bluetooth_sent_mb_per_second: float | None = None
    bluetooth_radio_state: str | None = None
    bluetooth_device_names: list[str] | None = None
    timestamp: datetime | None = None

@dataclass(frozen=True)
class Network:
    __dokime_stream__: ClassVar[_Stream]
    sample_index: int | None = None
    probe_status: str | None = None
    probe_error: str | None = None
    radio_link: str | None = None
    default_interface: str | None = None
    default_gateway: str | None = None
    local_addresses: list[str] | None = None
    local_address_count: int | None = None
    network_interface_count: int | None = None
    network_recv_mb_per_second: float | None = None
    network_sent_mb_per_second: float | None = None
    network_total_recv_bytes: int | None = None
    network_total_sent_bytes: int | None = None
    network_rx_errors: int | None = None
    network_tx_errors: int | None = None
    network_mtu: int | None = None
    network_mac: str | None = None
    network_operational_state: str | None = None
    timestamp: datetime | None = None

@dataclass(frozen=True)
class System:
    __dokime_stream__: ClassVar[_Stream]
    sample_index: int | None = None
    probe_status: str | None = None
    probe_error: str | None = None
    host_name: str | None = None
    user: str | None = None
    git_user_name: str | None = None
    git_user_email: str | None = None
    os_name: str | None = None
    os_version: str | None = None
    kernel_version: str | None = None
    cpu_architecture: str | None = None
    cpu_name: str | None = None
    cpu_brand: str | None = None
    cpu_vendor_id: str | None = None
    cpu_frequency_mhz: int | None = None
    physical_cpu_count: int | None = None
    logical_cpu_count: int | None = None
    cpu_load_percent: float | None = None
    memory_total_bytes: int | None = None
    memory_used_bytes: int | None = None
    memory_pressure_percent: float | None = None
    battery_percent: float | None = None
    battery_state: str | None = None
    battery_power_plugged: bool | None = None
    battery_temperature_c: float | None = None
    power_plan_name: str | None = None
    power_plan_guid: str | None = None
    power_mode: str | None = None
    power_mode_performance_bias: int | None = None
    timestamp: datetime | None = None

@dataclass(frozen=True)
class Wifi:
    __dokime_stream__: ClassVar[_Stream]
    sample_index: int | None = None
    probe_status: str | None = None
    probe_error: str | None = None
    wifi_adapter: str | None = None
    wifi_state: str | None = None
    wifi_ssid: str | None = None
    wifi_bssid: str | None = None
    wifi_profile: str | None = None
    wifi_radio: str | None = None
    wifi_auth: str | None = None
    wifi_cipher: str | None = None
    wifi_signal_percent: float | None = None
    wifi_rssi_dbm: float | None = None
    wifi_rx_phy_mbps: float | None = None
    wifi_tx_phy_mbps: float | None = None
    wifi_rx_mbps: float | None = None
    wifi_tx_mbps: float | None = None
    wifi_channel: int | None = None
    wifi_band: str | None = None
    wifi_network_count: int | None = None
    wifi_network_congestion_score: float | None = None
    congestion_model: str | None = None
    timestamp: datetime | None = None

@dataclass(frozen=True)
class WifiChannelOccupancy:
    __dokime_stream__: ClassVar[_Stream]
    sample_index: int | None = None
    scan_status: str | None = None
    scan_error: str | None = None
    wifi_adapter: str | None = None
    wifi_channel: int | None = None
    wifi_band: str | None = None
    network_count: int | None = None
    scan_network_count: int | None = None
    active_channel: bool | None = None
    overlap_weight: float | None = None
    same_channel_network_count: int | None = None
    overlapping_network_count: int | None = None
    overlap_weighted_network_count: float | None = None
    max_signal_percent: float | None = None
    wifi_network_congestion_score: float | None = None
    congestion_model: str | None = None
    timestamp: datetime | None = None

@dataclass(frozen=True)
class WifiLatency:
    __dokime_stream__: ClassVar[_Stream]
    sample_index: int | None = None
    probe_status: str | None = None
    probe_error: str | None = None
    wifi_latency_host: str | None = None
    wifi_latency_ms: float | None = None
    timestamp: datetime | None = None
