/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod capture;
pub mod contract;
pub mod lease;
pub mod row;
pub mod stream;

/* === Re-Exports === */

pub use crate::values::field;
pub use capture::capture_payload;
pub use contract::{
    TelemetryColumn, TelemetryCondition, TelemetryConditionEvaluation, TelemetryConditionOperator, TelemetryContract, TelemetryFilter,
    TelemetryRole, evaluate_telemetry_condition, validate_normalized_telemetry_sample, validate_telemetry_sample,
};
pub use dokime_domain::model::telemetry::SamplingScope;
pub use dokime_domain::model::telemetry::TelemetryBatch;
pub use dokime_domain::model::telemetry::TelemetryCaptureLease;
pub use dokime_domain::model::telemetry::TelemetrySample;
pub use dokime_domain::model::telemetry::TelemetrySampleKey;
pub use dokime_domain::model::telemetry::TelemetrySamplingRequest;
pub use dokime_domain::model::telemetry::TelemetrySourceDescriptor;
pub use dokime_domain::model::telemetry::TelemetrySourceKind;
pub use dokime_domain::model::telemetry::evidence;
pub use dokime_domain::model::telemetry::sampling;
pub use dokime_domain::model::validation::CompatibilityStatus;
pub use dokime_domain::model::validation::ValidationMode;
pub use lease::{LeaseGap, LeaseRequest};
pub use row::{FieldError, FieldValue, TelemetryRow};
pub use stream::{OwnedStream, Pushing, SampleSink, StreamSelection, Streams, TelemetryStream};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
