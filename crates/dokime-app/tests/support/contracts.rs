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
use dokime::core::{Identifier, Value, ValueType};
use dokime::records::{RecordContract, RecordField, RecordSeverity};
use dokime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole};
use harmos::{
    Message,
    schema::{Described, graph::Schema},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinimalTrackingRow {
    pub elapsed_s: f64,
    pub target_locked: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureValueRow {
    pub timestamp: String,
    pub value: Option<f64>,
    pub sample_index: Option<i64>,
    pub ordinal: Option<u64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrackingRow {
    pub timestamp: String,
    pub elapsed_s: Option<f64>,
    pub distance_m: Option<f64>,
    pub position_x_m: Option<f64>,
    pub position_y_m: Option<f64>,
    pub speed_m_per_s: Option<f64>,
    pub velocity_x_m_per_s: Option<f64>,
    pub velocity_y_m_per_s: Option<f64>,
    pub target_locked: Option<bool>,
    pub tool_connected: Option<bool>,
    pub target_reacquire_time_s: Option<f64>,
    pub connection_acquired_time_s: Option<f64>,
    pub connection_loss_distance_m: Option<f64>,
    pub connection_acquired_distance_m: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemRow {
    pub timestamp: String,
    pub cpu_load_percent: Option<f64>,
    pub memory_pressure_percent: Option<f64>,
    pub battery_percent: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WifiRow {
    pub timestamp: String,
    pub wifi_signal_percent: Option<f64>,
    pub wifi_rx_phy_mbps: Option<f64>,
    pub wifi_tx_phy_mbps: Option<f64>,
    pub wifi_rx_mbps: Option<f64>,
    pub wifi_tx_mbps: Option<f64>,
    pub wifi_network_congestion_score: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LatencyRow {
    pub timestamp: String,
    pub wifi_latency_ms: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OccupancyRow {
    pub timestamp: String,
    pub sample_index: Option<i64>,
    pub active_wifi_channel: Option<i64>,
    pub wifi_channel: Option<i64>,
    pub network_count: Option<i64>,
    pub scan_network_count: Option<i64>,
    pub wifi_network_congestion_score: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BluetoothRow {
    pub timestamp: String,
    pub bluetooth_adapter_count: Option<f64>,
    pub bluetooth_connected_device_count: Option<f64>,
    pub bluetooth_present_device_count: Option<f64>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn fixture_schema(stream: &str) -> Option<Schema> {
    let settings = harmos::config::SchemaSettings::default();
    let definition = match stream {
        "fixture.telemetry" | "synthetic.samples" => FixtureValueRow::DEFINITION,
        "tracking" | "plt400.ract.tracking.tracking" => TrackingRow::DEFINITION,
        "host.system" => SystemRow::DEFINITION,
        "host.wifi" => WifiRow::DEFINITION,
        "host.wifi.latency" => LatencyRow::DEFINITION,
        "host.wifi.channel_occupancy" => OccupancyRow::DEFINITION,
        "host.bluetooth" => BluetoothRow::DEFINITION,
        _ => return None,
    };
    Some(Schema::from_declaration(definition, &settings).unwrap())
}

pub fn typed_minimal_tracking_sample(
    mut sample: dokime::telemetry::TelemetrySample,
) -> dokime::telemetry::TelemetrySample {
    sample.storage_payload = Some(fixture_payload::<MinimalTrackingRow>(&sample));
    sample
}

pub fn provider_tracking_row(
    elapsed_s: f64,
    target_locked: bool,
) -> dokime::transport::ProviderTelemetryRow {
    let row = MinimalTrackingRow {
        elapsed_s,
        target_locked,
    };
    dokime::transport::ProviderTelemetryRow {
        values: std::collections::BTreeMap::from([
            ("elapsed_s".into(), dokime::core::Value::Number(elapsed_s)),
            (
                "target_locked".into(),
                dokime::core::Value::Bool(target_locked),
            ),
        ]),
        payload: row.encode_to_vec(),
    }
}

pub fn typed_fixture_sample(
    mut sample: dokime::telemetry::TelemetrySample,
) -> dokime::telemetry::TelemetrySample {
    let bytes = match sample.stream.as_str() {
        "fixture.telemetry" | "synthetic.samples" => fixture_payload::<FixtureValueRow>(&sample),
        "tracking" | "plt400.ract.tracking.tracking" => fixture_payload::<TrackingRow>(&sample),
        "host.system" => fixture_payload::<SystemRow>(&sample),
        "host.wifi" => fixture_payload::<WifiRow>(&sample),
        "host.wifi.latency" => fixture_payload::<LatencyRow>(&sample),
        "host.wifi.channel_occupancy" => fixture_payload::<OccupancyRow>(&sample),
        "host.bluetooth" => fixture_payload::<BluetoothRow>(&sample),
        stream => panic!("fixture stream has no explicit typed row: {stream}"),
    };
    sample.storage_payload = Some(bytes);
    sample
}

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
        storage_schema: fixture_schema("fixture.telemetry"),
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
        storage_schema: fixture_schema(stream),
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
        storage_schema: fixture_schema("tracking"),
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
        storage_schema: Some(
            Schema::from_declaration(
                MinimalTrackingRow::DEFINITION,
                &harmos::config::SchemaSettings::default(),
            )
            .unwrap(),
        ),
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
        storage_schema: fixture_schema("host.wifi.channel_occupancy"),
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
fn fixture_payload<T: serde::de::DeserializeOwned + Message + Described>(
    sample: &dokime::telemetry::TelemetrySample,
) -> Vec<u8> {
    let mut values = Value::Object(sample.values.clone())
        .to_plain_json()
        .unwrap();
    let graph = Schema::from_declaration(T::DEFINITION, &harmos::config::SchemaSettings::default())
        .unwrap();
    if graph.definitions[graph.root]
        .iter()
        .any(|field| field.name == "timestamp")
    {
        values.as_object_mut().unwrap().insert(
            "timestamp".into(),
            serde_json::Value::String(sample.timestamp.as_str().to_owned()),
        );
    } else {
        values.as_object_mut().unwrap().remove("timestamp");
    }
    serde_json::from_value::<T>(values).unwrap().encode_to_vec()
}

/* === Tests === */
