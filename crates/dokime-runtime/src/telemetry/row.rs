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

pub use crate::values::{FieldError, FieldValue};
pub use dokime_domain::model::telemetry::TelemetryRow;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::telemetry::contract::{TelemetryColumn, TelemetryRole};
use dokime_domain::model::values::ValueType;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declared_column(field: harmos::StreamField, value_type: ValueType) -> TelemetryColumn {
    let role = match field.role.as_deref().unwrap_or("value") {
        "counter" => TelemetryRole::Counter,
        "state" => TelemetryRole::State,
        "distance" => TelemetryRole::Distance,
        "latency" => TelemetryRole::Latency,
        "throughput" => TelemetryRole::Throughput,
        "value" => TelemetryRole::Value,
        role => panic!("unsupported Dokime telemetry role `{role}`"),
    };
    let mut column = TelemetryColumn::declared(field.metadata.identifier.as_str(), &field.metadata.label, value_type, role);
    column.metadata = field.metadata;
    column.unit = field.unit.map(crate::definition::authored_unit);

    column
}

/* === Private Functions === */

/* === Tests === */
