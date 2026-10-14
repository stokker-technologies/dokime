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
use std::{collections::BTreeMap, str::FromStr};
// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use crate::core::{Identifier, RecordScope, Timestamp, Value};
use crate::records::{BuiltinRecordType, Record, RecordSeverity, error::RecordFailure};

/* === Constants === */
pub const PRODUCT_RECORD_SCHEMA_VERSION: u32 = 2;

/* === Macros === */
macro_rules! product_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $($variant:ident => $value:literal),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }

        impl From<$name> for Value {
            fn from(value: $name) -> Self {
                Value::from(value.as_str())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = crate::Error;

            fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
                let normalized = normalize_state_value(value);
                match normalized.as_str() {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(RecordFailure::InvalidProductState {
                        field: stringify!($name).to_owned(),
                        value: value.to_owned(),
                    }
                    .into()),
                }
            }
        }
    };
}

product_enum! {
    pub enum ProductConnectionState {
        Connected => "connected",
        Disconnected => "disconnected",
    }
}

product_enum! {
    pub enum ProductReadinessState {
        Ready => "ready",
        NotReady => "not_ready",
        Busy => "busy",
        Unknown => "unknown",
    }
}

product_enum! {
    pub enum ProductTargetLockState {
        Locked => "locked",
        Lost => "lost",
        Unknown => "unknown",
    }
}

product_enum! {
    pub enum ProductCalibrationKind {
        Field => "field",
        Tilt => "tilt",
    }
}

product_enum! {
    pub enum ProductCalibrationPhase {
        Started => "started",
        Activity => "activity",
        Completed => "completed",
    }
}

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductConnectionDetails {
    pub resource_id: String,
    pub resource_kind: String,
    pub connection_state: ProductConnectionState,
    pub previous_connection_state: Option<ProductConnectionState>,
    pub readiness_state: Option<ProductReadinessState>,
    pub target_lock_state: Option<ProductTargetLockState>,
    pub connection_transport: Option<String>,
    pub connection_endpoint: Option<String>,
    pub status_source: Option<String>,
    pub status_message: Option<String>,
    pub device_timestamp: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductTargetLockDetails {
    pub resource_id: String,
    pub resource_kind: String,
    pub target_lock_state: ProductTargetLockState,
    pub connection_state: Option<ProductConnectionState>,
    pub readiness_state: Option<ProductReadinessState>,
    pub status_source: Option<String>,
    pub status_message: Option<String>,
    pub device_timestamp: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProductCalibrationDetails {
    pub resource_id: String,
    pub resource_kind: String,
    pub calibration_kind: ProductCalibrationKind,
    pub calibration_phase: ProductCalibrationPhase,
    pub calibration_state: Option<String>,
    pub calibration_source: Option<String>,
    pub calibration_confidence: Option<f64>,
    pub connection_state: Option<ProductConnectionState>,
    pub readiness_state: Option<ProductReadinessState>,
    pub status_source: Option<String>,
    pub status_message: Option<String>,
    pub device_timestamp: Option<String>,
    pub raw_calibration_kind: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductRecordDraft {
    pub record_type: Identifier,
    pub source: String,
    pub severity: RecordSeverity,
    pub message: Option<String>,
    pub details: BTreeMap<String, Value>,
    pub scope: Option<RecordScope>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ProductConnectionDetails {
    pub fn new(resource_id: impl Into<String>, connection_state: ProductConnectionState) -> Self {
        Self {
            resource_id: resource_id.into(),
            resource_kind: "tool".to_owned(),
            connection_state,
            previous_connection_state: None,
            readiness_state: None,
            target_lock_state: None,
            connection_transport: None,
            connection_endpoint: None,
            status_source: None,
            status_message: None,
            device_timestamp: None,
        }
    }

    pub fn record_type(&self) -> Identifier {
        match self.connection_state {
            ProductConnectionState::Connected => {
                BuiltinRecordType::ToolConnectionAcquired.identifier()
            }
            ProductConnectionState::Disconnected => {
                BuiltinRecordType::ToolConnectionLost.identifier()
            }
        }
    }

    pub fn into_draft(self, source: impl Into<String>) -> ProductRecordDraft {
        ProductRecordDraft::new(
            self.record_type(),
            source,
            Some("Product connection state changed.".to_owned()),
            self.into_details(),
        )
    }

    pub fn into_details(self) -> BTreeMap<String, Value> {
        let mut details = BTreeMap::from([
            ("resource_id".to_owned(), Value::from(self.resource_id)),
            ("resource_kind".to_owned(), Value::from(self.resource_kind)),
            (
                "connection_state".to_owned(),
                Value::from(self.connection_state),
            ),
        ]);
        insert_enum(
            &mut details,
            "previous_connection_state",
            self.previous_connection_state,
        );
        insert_enum(&mut details, "readiness_state", self.readiness_state);
        insert_enum(&mut details, "target_lock_state", self.target_lock_state);
        insert_text(
            &mut details,
            "connection_transport",
            self.connection_transport,
        );
        insert_text(
            &mut details,
            "connection_endpoint",
            self.connection_endpoint,
        );
        insert_text(&mut details, "status_source", self.status_source);
        insert_text(&mut details, "status_message", self.status_message);
        insert_text(&mut details, "device_timestamp", self.device_timestamp);
        details
    }
}

impl ProductConnectionState {
    pub fn from_connected(connected: bool) -> Self {
        if connected {
            Self::Connected
        } else {
            Self::Disconnected
        }
    }
}

impl ProductTargetLockDetails {
    pub fn new(resource_id: impl Into<String>, target_lock_state: ProductTargetLockState) -> Self {
        Self {
            resource_id: resource_id.into(),
            resource_kind: "tool".to_owned(),
            target_lock_state,
            connection_state: None,
            readiness_state: None,
            status_source: None,
            status_message: None,
            device_timestamp: None,
        }
    }

    pub fn record_type(&self) -> Identifier {
        match self.target_lock_state {
            ProductTargetLockState::Lost => BuiltinRecordType::TargetLost.identifier(),
            ProductTargetLockState::Locked | ProductTargetLockState::Unknown => {
                BuiltinRecordType::TargetAcquired.identifier()
            }
        }
    }

    pub fn into_draft(self, source: impl Into<String>) -> ProductRecordDraft {
        ProductRecordDraft::new(
            self.record_type(),
            source,
            Some("Product target lock state changed.".to_owned()),
            self.into_details(),
        )
    }

    pub fn into_details(self) -> BTreeMap<String, Value> {
        let mut details = BTreeMap::from([
            ("resource_id".to_owned(), Value::from(self.resource_id)),
            ("resource_kind".to_owned(), Value::from(self.resource_kind)),
            (
                "target_lock_state".to_owned(),
                Value::from(self.target_lock_state),
            ),
        ]);
        insert_enum(&mut details, "connection_state", self.connection_state);
        insert_enum(&mut details, "readiness_state", self.readiness_state);
        insert_text(&mut details, "status_source", self.status_source);
        insert_text(&mut details, "status_message", self.status_message);
        insert_text(&mut details, "device_timestamp", self.device_timestamp);
        details
    }
}

impl ProductTargetLockState {
    pub fn from_locked(locked: Option<bool>) -> Self {
        match locked {
            Some(true) => Self::Locked,
            Some(false) => Self::Lost,
            None => Self::Unknown,
        }
    }
}

impl ProductCalibrationDetails {
    pub fn new(
        resource_id: impl Into<String>,
        calibration_kind: ProductCalibrationKind,
        calibration_phase: ProductCalibrationPhase,
    ) -> Self {
        Self {
            resource_id: resource_id.into(),
            resource_kind: "tool".to_owned(),
            calibration_kind,
            calibration_phase,
            calibration_state: None,
            calibration_source: None,
            calibration_confidence: None,
            connection_state: None,
            readiness_state: None,
            status_source: None,
            status_message: None,
            device_timestamp: None,
            raw_calibration_kind: None,
        }
    }

    pub fn record_type(&self) -> Identifier {
        identifier(format!(
            "calibration.{}.{}",
            self.calibration_kind.as_str(),
            self.calibration_phase.as_str()
        ))
    }

    pub fn into_draft(self, source: impl Into<String>) -> ProductRecordDraft {
        ProductRecordDraft::new(
            self.record_type(),
            source,
            Some("Product calibration state changed.".to_owned()),
            self.into_details(),
        )
    }

    pub fn into_details(self) -> BTreeMap<String, Value> {
        let mut details = BTreeMap::from([
            ("resource_id".to_owned(), Value::from(self.resource_id)),
            ("resource_kind".to_owned(), Value::from(self.resource_kind)),
            (
                "calibration_kind".to_owned(),
                Value::from(self.calibration_kind),
            ),
            (
                "calibration_phase".to_owned(),
                Value::from(self.calibration_phase),
            ),
        ]);
        insert_text(&mut details, "calibration_state", self.calibration_state);
        insert_text(&mut details, "calibration_source", self.calibration_source);
        if let Some(confidence) = self.calibration_confidence {
            details.insert("calibration_confidence".to_owned(), Value::from(confidence));
        }
        insert_enum(&mut details, "connection_state", self.connection_state);
        insert_enum(&mut details, "readiness_state", self.readiness_state);
        insert_text(&mut details, "status_source", self.status_source);
        insert_text(&mut details, "status_message", self.status_message);
        insert_text(&mut details, "device_timestamp", self.device_timestamp);
        insert_text(
            &mut details,
            "raw_calibration_kind",
            self.raw_calibration_kind,
        );
        details
    }
}

impl ProductRecordDraft {
    pub fn new(
        record_type: Identifier,
        source: impl Into<String>,
        message: Option<String>,
        details: BTreeMap<String, Value>,
    ) -> Self {
        Self {
            record_type,
            source: source.into(),
            severity: RecordSeverity::Info,
            message,
            details,
            scope: None,
        }
    }

    pub fn with_scope(mut self, scope: RecordScope) -> Self {
        self.scope = Some(scope);
        self
    }
    pub fn into_record(self, timestamp: Timestamp) -> Record {
        let mut record = Record::new(
            timestamp,
            self.record_type,
            PRODUCT_RECORD_SCHEMA_VERSION,
            self.source,
            self.severity,
        );
        record.message = self.message;
        record.details = self.details;
        record.scope = self.scope;
        record
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn normalize_state_value(value: &str) -> String {
    let normalized = value.trim().to_ascii_lowercase().replace(['-', ' '], "_");

    normalize_product_alias(&normalized)
}

fn normalize_product_alias(value: &str) -> String {
    match value {
        "fieldcalibration" | "field_calibration" | "field_cal" => "field".to_owned(),
        "tiltmeasure" | "tilt_measure" | "tilt_calibration" | "tiltcalibration" => {
            "tilt".to_owned()
        }
        "done" | "complete" | "success" | "succeeded" => "completed".to_owned(),
        "start" | "begin" | "began" => "started".to_owned(),
        "active" | "running" | "in_progress" | "progress" => "activity".to_owned(),
        "target_locked" | "lock" | "found" | "target_found" => "locked".to_owned(),
        "target_lost" | "lost_target" | "not_locked" | "not_found" => "lost".to_owned(),
        "notready" | "not_ready" => "not_ready".to_owned(),
        _ => value.to_owned(),
    }
}

fn identifier(value: impl Into<String>) -> Identifier {
    Identifier::new(value).expect("product record identifiers are valid")
}

fn insert_enum<T>(details: &mut BTreeMap<String, Value>, key: &str, value: Option<T>)
where
    T: Into<Value>,
{
    if let Some(value) = value {
        details.insert(key.to_owned(), value.into());
    }
}

fn insert_text(details: &mut BTreeMap<String, Value>, key: &str, value: Option<String>) {
    let Some(value) = value else {
        return;
    };
    if value.is_empty() {
        return;
    }
    details.insert(key.to_owned(), Value::from(value));
}

/* === Tests === */
