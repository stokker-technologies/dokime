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
pub use crate::core::{CompatibilityStatus, ValidationMode};
pub use crate::values::field;
pub use capture::capture_payload;
pub use contract::{
    TelemetryColumn, TelemetryCondition, TelemetryConditionEvaluation, TelemetryConditionOperator,
    TelemetryContract, TelemetryFilter, TelemetryRole, evaluate_telemetry_condition,
    validate_normalized_telemetry_sample, validate_telemetry_sample,
};
pub use dokime_contract::telemetry::{
    SamplingScope, TelemetryBatch, TelemetryCaptureLease, TelemetrySample, TelemetrySampleKey,
    TelemetrySamplingRequest, TelemetrySourceDescriptor, TelemetrySourceKind,
};
pub use dokime_contract::telemetry::{evidence, sampling};
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
