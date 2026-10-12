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
use super::{identifier, metadata};
use dokime::core::{Identifier, ValueType};
use dokime::records::{RecordContract, RecordField, RecordSeverity};
use dokime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn fixture_record_contract() -> RecordContract {
    RecordContract {
        record_type: identifier("fixture.event"),
        schema_version: 1,
        metadata: metadata("fixture.event", "Fixture Event"),
        display: Default::default(),
        template_id: None,
        owner: None,
        default_severity: RecordSeverity::Info,
        fields: Vec::new(),
        validation_mode: None,
        allow_extra_fields: true,
    }
}

pub fn fixture_telemetry_contract() -> TelemetryContract {
    TelemetryContract {
        stream: identifier("fixture.telemetry"),
        schema_version: 1,
        metadata: metadata("fixture.telemetry", "Fixture Telemetry"),
        template_id: None,
        owner: None,
        columns: vec![TelemetryColumn::new(
            metadata("value", "Value"),
            ValueType::Number,
            TelemetryRole::Value,
        )],
        default_x: Some("timestamp".to_owned()),
        interval: None,
        plot_window: None,
        allow_extra_fields: false,
        scalar: false,
        validation_mode: None,
        foreign: false,
    }
}

pub fn tracking_warning_contract() -> RecordContract {
    RecordContract {
        record_type: Identifier::new("tracking.fixture_warning").unwrap(),
        schema_version: 1,
        metadata: metadata("tracking.fixture_warning", "Tracking Fixture Warning"),
        display: Default::default(),
        template_id: None,
        owner: None,
        default_severity: RecordSeverity::Warning,
        fields: vec![RecordField::new(
            metadata("warning_code", "Warning Code"),
            ValueType::Text,
        )],
        validation_mode: None,
        allow_extra_fields: false,
    }
}

pub fn custom_record_contract() -> RecordContract {
    RecordContract {
        record_type: identifier("custom.measurement"),
        schema_version: 1,
        metadata: metadata("custom.measurement", "Custom Measurement"),
        display: Default::default(),
        template_id: None,
        owner: None,
        default_severity: RecordSeverity::Info,
        fields: vec![RecordField::new(
            metadata("value", "Value"),
            ValueType::Number,
        )],
        validation_mode: None,
        allow_extra_fields: false,
    }
}

pub fn tracking_record_contracts() -> Vec<RecordContract> {
    let mut contracts = vec![custom_record_contract()];
    contracts.extend(
        dokime::records::builtin_record_contracts()
            .into_iter()
            .filter(|contract| {
                matches!(
                    contract.record_type.as_str(),
                    "target.lost"
                        | "target.acquired"
                        | "tool.connection.lost"
                        | "tool.connection.acquired"
                )
            }),
    );
    contracts
}

pub fn suite_telemetry_contracts() -> Vec<TelemetryContract> {
    let mut contracts = vec![tracking_contract()];
    contracts.extend(fixture_host_contracts());
    contracts
}

pub fn fixture_host_contracts() -> Vec<TelemetryContract> {
    vec![
        fixture_host_contract(
            "host.system",
            "Host System",
            vec![
                fixture_host_column("cpu_load_percent", "CPU Load"),
                fixture_host_column("memory_pressure_percent", "Memory Pressure"),
                fixture_host_column("battery_percent", "Battery"),
            ],
        ),
        fixture_host_contract(
            "host.wifi",
            "Host WiFi",
            vec![
                fixture_host_column("wifi_signal_percent", "WiFi Signal"),
                fixture_host_column("wifi_rx_phy_mbps", "WiFi RX PHY"),
                fixture_host_column("wifi_tx_phy_mbps", "WiFi TX PHY"),
                fixture_host_column("wifi_rx_mbps", "WiFi RX Throughput"),
                fixture_host_column("wifi_tx_mbps", "WiFi TX Throughput"),
                fixture_host_column("wifi_network_congestion_score", "WiFi Congestion"),
            ],
        ),
        fixture_host_contract(
            "host.wifi.latency",
            "Host WiFi Latency",
            vec![fixture_host_column("wifi_latency_ms", "WiFi Latency")],
        ),
        fixture_host_contract(
            "host.wifi.channel_occupancy",
            "Host WiFi Channel Occupancy",
            vec![
                fixture_host_column("active_wifi_channel", "Active WiFi Channel"),
                fixture_host_column("wifi_channel", "WiFi Channel"),
                fixture_host_column("network_count", "Network Count"),
                fixture_host_column("scan_network_count", "Scan Network Count"),
                fixture_host_column("wifi_network_congestion_score", "WiFi Congestion"),
            ],
        ),
        fixture_host_contract(
            "host.bluetooth",
            "Host Bluetooth",
            vec![
                fixture_host_column("bluetooth_adapter_count", "Bluetooth Adapters"),
                fixture_host_column(
                    "bluetooth_connected_device_count",
                    "Connected Bluetooth Devices",
                ),
                fixture_host_column(
                    "bluetooth_present_device_count",
                    "Present Bluetooth Devices",
                ),
            ],
        ),
    ]
}

pub fn fixture_host_contract(
    stream: &str,
    label: &str,
    columns: Vec<TelemetryColumn>,
) -> TelemetryContract {
    TelemetryContract {
        stream: identifier(stream),
        schema_version: 1,
        metadata: metadata(stream, label),
        template_id: None,
        owner: None,
        columns,
        default_x: Some("timestamp".to_owned()),
        // A host stream declares its cadence the way the real ones do, so a
        // fixture is leasable on the same terms.
        interval: Some(dokime::core::DurationMillis(1_000)),
        plot_window: None,
        allow_extra_fields: true,
        scalar: false,
        validation_mode: None,
        foreign: false,
    }
}

pub fn fixture_host_column(name: &str, label: &str) -> TelemetryColumn {
    TelemetryColumn::new(
        metadata(name, label),
        ValueType::Number,
        TelemetryRole::Value,
    )
    .optional()
}

pub fn tracking_contract() -> TelemetryContract {
    TelemetryContract {
        stream: identifier("tracking"),
        schema_version: 1,
        metadata: metadata("tracking", "Tracking"),
        template_id: None,
        owner: None,
        columns: vec![
            TelemetryColumn::new(
                metadata("elapsed_s", "Elapsed"),
                ValueType::Number,
                TelemetryRole::Value,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("distance_m", "Distance"),
                ValueType::Number,
                TelemetryRole::Distance,
            ),
            TelemetryColumn::new(
                metadata("position_x_m", "Position X"),
                ValueType::Number,
                TelemetryRole::Distance,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("position_y_m", "Position Y"),
                ValueType::Number,
                TelemetryRole::Distance,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("speed_m_per_s", "Speed"),
                ValueType::Number,
                TelemetryRole::Value,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("velocity_x_m_per_s", "Velocity X"),
                ValueType::Number,
                TelemetryRole::Value,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("velocity_y_m_per_s", "Velocity Y"),
                ValueType::Number,
                TelemetryRole::Value,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("target_locked", "Target Locked"),
                ValueType::Boolean,
                TelemetryRole::State,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("tool_connected", "Tool Connected"),
                ValueType::Boolean,
                TelemetryRole::State,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("target_reacquire_time_s", "Target Reacquire Time"),
                ValueType::Number,
                TelemetryRole::Latency,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("connection_acquired_time_s", "Connection Acquired Time"),
                ValueType::Number,
                TelemetryRole::Latency,
            )
            .optional(),
            TelemetryColumn::new(
                metadata("connection_loss_distance_m", "Connection Loss Distance"),
                ValueType::Number,
                TelemetryRole::Distance,
            )
            .optional(),
            TelemetryColumn::new(
                metadata(
                    "connection_acquired_distance_m",
                    "Connection Acquired Distance",
                ),
                ValueType::Number,
                TelemetryRole::Distance,
            )
            .optional(),
        ],
        default_x: Some("timestamp".to_owned()),
        interval: None,
        plot_window: None,
        allow_extra_fields: false,
        scalar: false,
        validation_mode: None,
        foreign: false,
    }
}

pub fn minimal_tracking_contract() -> TelemetryContract {
    TelemetryContract {
        columns: tracking_contract()
            .columns
            .into_iter()
            .filter(|column| {
                matches!(
                    column.metadata.identifier.as_str(),
                    "elapsed_s" | "target_locked"
                )
            })
            .map(|column| TelemetryColumn {
                required: true,
                ..column
            })
            .collect(),
        default_x: Some("elapsed_s".to_owned()),
        ..tracking_contract()
    }
}

pub fn conflicting_tracking_contract() -> TelemetryContract {
    TelemetryContract {
        columns: vec![TelemetryColumn::new(
            metadata("distance_m", "Distance"),
            ValueType::Text,
            TelemetryRole::Distance,
        )],
        ..tracking_contract()
    }
}

pub fn wifi_occupancy_contract() -> TelemetryContract {
    TelemetryContract {
        stream: identifier("host.wifi.channel_occupancy"),
        schema_version: 1,
        metadata: metadata("host.wifi.channel_occupancy", "WiFi Occupancy"),
        template_id: None,
        owner: None,
        columns: vec![
            TelemetryColumn::new(
                metadata("sample_index", "Sample Index"),
                ValueType::Integer,
                TelemetryRole::Value,
            ),
            TelemetryColumn::new(
                metadata("wifi_channel", "WiFi Channel"),
                ValueType::Integer,
                TelemetryRole::Value,
            ),
            TelemetryColumn::new(
                metadata("network_count", "Network Count"),
                ValueType::Integer,
                TelemetryRole::Value,
            ),
        ],
        default_x: Some("sample_index".to_owned()),
        interval: None,
        plot_window: None,
        allow_extra_fields: false,
        scalar: false,
        validation_mode: None,
        foreign: false,
    }
}

/* === Private Functions === */

/* === Tests === */
