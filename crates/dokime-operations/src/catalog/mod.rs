/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod get_record_schema;
mod get_suite;
mod get_telemetry_schema;
mod list_record_schemas;
mod list_suites;
mod list_telemetry_schemas;

/* === Re-Exports === */

pub use get_record_schema::GetRecordSchema;
pub use get_suite::GetSuite;
pub use get_telemetry_schema::GetTelemetrySchema;
pub use list_record_schemas::ListRecordSchemas;
pub use list_suites::ListSuites;
pub use list_telemetry_schemas::ListTelemetrySchemas;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::ORDINARY;

use dokime_domain::model::definition::wire::*;
use dokime_domain::model::execution::wire_cursor::Cursor;

use dokime_domain::model::catalog::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[harmos::catalog]
pub enum Operations {
    ListSuites(ListSuites),
    GetSuite(GetSuite),
    ListRecordSchemas(ListRecordSchemas),
    GetRecordSchema(GetRecordSchema),
    ListTelemetrySchemas(ListTelemetrySchemas),
    GetTelemetrySchema(GetTelemetrySchema),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
