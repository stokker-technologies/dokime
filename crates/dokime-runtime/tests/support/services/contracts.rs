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
use dokime_runtime::records::{RecordContract, RecordField, RecordSeverity};
use dokime_runtime::telemetry::{TelemetryColumn, TelemetryContract, TelemetryRole, TelemetryRow};
use dokime_runtime::{Identifier, Value, ValueType};
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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn fixture_schema(stream: &str) -> Option<Schema> {
    let settings = harmos::config::SchemaSettings::default();
    let definition = match stream {
        "fixture.telemetry" | "synthetic.samples" => FixtureValueRow::DEFINITION,
        "tracking" | "plt400.ract.tracking.tracking" => TrackingRow::DEFINITION,
        "host.system" => plugin_host::System::DEFINITION,
        "host.network" => plugin_host::Network::DEFINITION,
        "host.wifi.link" => plugin_host::WifiLink::DEFINITION,
        "host.wifi.latency" => plugin_host::WifiLatency::DEFINITION,
        "host.wifi.scan" => plugin_host::WifiScan::DEFINITION,
        "host.bluetooth" => plugin_host::Bluetooth::DEFINITION,
        _ => return None,
    };
    Some(Schema::from_declaration(definition, &settings).unwrap())
}

pub fn typed_minimal_tracking_sample(mut sample: dokime_runtime::telemetry::TelemetrySample) -> dokime_runtime::telemetry::TelemetrySample {
    sample.storage_payload = Some(fixture_payload::<MinimalTrackingRow>(&sample));
    sample
}

pub fn provider_tracking_row(elapsed_s: f64, target_locked: bool) -> dokime_runtime::transport::ProviderTelemetryRow {
    let row = MinimalTrackingRow { elapsed_s, target_locked };
    dokime_runtime::transport::ProviderTelemetryRow {
        values: std::collections::BTreeMap::from([
            ("elapsed_s".into(), dokime_runtime::Value::Number(elapsed_s)),
            ("target_locked".into(), dokime_runtime::Value::Bool(target_locked)),
        ]),
        payload: row.encode(),
    }
}

pub fn typed_fixture_sample(mut sample: dokime_runtime::telemetry::TelemetrySample) -> dokime_runtime::telemetry::TelemetrySample {
    let bytes = match sample.stream.as_str() {
        "fixture.telemetry" | "synthetic.samples" => fixture_payload::<FixtureValueRow>(&sample),
        "tracking" | "plt400.ract.tracking.tracking" => fixture_payload::<TrackingRow>(&sample),
        "host.system" => plugin_host::System::from_row(&sample.values).unwrap().encode(),
        "host.network" => plugin_host::Network::from_row(&sample.values).unwrap().encode(),
        "host.wifi.link" => plugin_host::WifiLink::from_row(&sample.values).unwrap().encode(),
        "host.wifi.latency" => plugin_host::WifiLatency::from_row(&sample.values).unwrap().encode(),
        "host.wifi.scan" => plugin_host::WifiScan::from_row(&sample.values).unwrap().encode(),
        "host.bluetooth" => plugin_host::Bluetooth::from_row(&sample.values).unwrap().encode(),
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
        columns: vec![TelemetryColumn::new(metadata("value", "Value"), ValueType::Number, TelemetryRole::Value)],
        default_x: Some("timestamp".to_owned()),
        interval: None,
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
        fields: vec![RecordField::new(metadata("warning_code", "Warning Code"), ValueType::Text)],
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
        fields: vec![RecordField::new(metadata("value", "Value"), ValueType::Number)],
        validation_mode: None,
        allow_extra_fields: false,
    }
}

pub fn tracking_record_contracts() -> Vec<RecordContract> {
    let mut contracts = vec![custom_record_contract()];
    contracts.extend(dokime_runtime::records::builtin_record_contracts().into_iter().filter(|contract| {
        matches!(contract.record_type.as_str(), "target.lost" | "target.acquired" | "tool.connection.lost" | "tool.connection.acquired")
    }));
    contracts
}

pub fn suite_telemetry_contracts() -> Vec<TelemetryContract> {
    let mut contracts = vec![tracking_contract()];
    contracts.extend(plugin_host::host_telemetry_contracts());
    contracts
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
            TelemetryColumn::new(metadata("elapsed_s", "Elapsed"), ValueType::Number, TelemetryRole::Value).optional(),
            TelemetryColumn::new(metadata("distance_m", "Distance"), ValueType::Number, TelemetryRole::Distance),
            TelemetryColumn::new(metadata("position_x_m", "Position X"), ValueType::Number, TelemetryRole::Distance).optional(),
            TelemetryColumn::new(metadata("position_y_m", "Position Y"), ValueType::Number, TelemetryRole::Distance).optional(),
            TelemetryColumn::new(metadata("speed_m_per_s", "Speed"), ValueType::Number, TelemetryRole::Value).optional(),
            TelemetryColumn::new(metadata("velocity_x_m_per_s", "Velocity X"), ValueType::Number, TelemetryRole::Value).optional(),
            TelemetryColumn::new(metadata("velocity_y_m_per_s", "Velocity Y"), ValueType::Number, TelemetryRole::Value).optional(),
            TelemetryColumn::new(metadata("target_locked", "Target Locked"), ValueType::Boolean, TelemetryRole::State).optional(),
            TelemetryColumn::new(metadata("tool_connected", "Tool Connected"), ValueType::Boolean, TelemetryRole::State).optional(),
            TelemetryColumn::new(metadata("target_reacquire_time_s", "Target Reacquire Time"), ValueType::Number, TelemetryRole::Latency)
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
                metadata("connection_acquired_distance_m", "Connection Acquired Distance"),
                ValueType::Number,
                TelemetryRole::Distance,
            )
            .optional(),
        ],
        default_x: Some("timestamp".to_owned()),
        interval: None,
        allow_extra_fields: false,
        scalar: false,
        validation_mode: None,
        foreign: false,
    }
}

pub fn minimal_tracking_contract() -> TelemetryContract {
    TelemetryContract {
        storage_schema: Some(Schema::from_declaration(MinimalTrackingRow::DEFINITION, &harmos::config::SchemaSettings::default()).unwrap()),
        columns: tracking_contract()
            .columns
            .into_iter()
            .filter(|column| matches!(column.metadata.identifier.as_str(), "elapsed_s" | "target_locked"))
            .map(|column| TelemetryColumn { required: true, ..column })
            .collect(),
        default_x: Some("elapsed_s".to_owned()),
        ..tracking_contract()
    }
}

pub fn conflicting_tracking_contract() -> TelemetryContract {
    TelemetryContract {
        columns: vec![TelemetryColumn::new(metadata("distance_m", "Distance"), ValueType::Text, TelemetryRole::Distance)],
        ..tracking_contract()
    }
}

pub fn wifi_occupancy_contract() -> TelemetryContract {
    let mut contract = plugin_host::host_telemetry_contract(plugin_host::HostTelemetryStream::WifiScan);
    contract.default_x = Some("sample_index".to_owned());
    contract.allow_extra_fields = false;
    contract
}

/* === Private Functions === */
fn fixture_payload<T: serde::de::DeserializeOwned + Message + Described>(sample: &dokime_runtime::telemetry::TelemetrySample) -> Vec<u8> {
    let mut values = Value::Object(sample.values.clone()).to_plain_json().unwrap();
    let graph = Schema::from_declaration(T::DEFINITION, &harmos::config::SchemaSettings::default()).unwrap();
    if graph.definitions[graph.root].iter().any(|field| field.name == "timestamp") {
        values.as_object_mut().unwrap().insert("timestamp".into(), serde_json::Value::String(sample.timestamp.as_str().to_owned()));
    } else {
        values.as_object_mut().unwrap().remove("timestamp");
    }
    serde_json::from_value::<T>(values).unwrap().encode()
}

/* === Tests === */
