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
use super::Endpoint;
use super::ORDINARY;

use dokime_domain::model::definition::wire::*;
use dokime_domain::model::execution::wire_cursor::Cursor;

use dokime_domain::model::catalog::wire::*;

/* === Constants === */

/* === Macros === */

#[cfg(feature = "runtime")]
crate::hosted!(GetRecordSchema, GetSuite, GetTelemetrySchema, ListRecordSchemas, ListSuites, ListTelemetrySchemas);

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

impl Endpoint for ListSuites {
    const PATH: &'static str = "/dokime.v1.Catalog/ListSuites";
}

impl Endpoint for GetSuite {
    const PATH: &'static str = "/dokime.v1.Catalog/GetSuite";
}

impl Endpoint for ListRecordSchemas {
    const PATH: &'static str = "/dokime.v1.Catalog/ListRecordSchemas";
}

impl Endpoint for GetRecordSchema {
    const PATH: &'static str = "/dokime.v1.Catalog/GetRecordSchema";
}

impl Endpoint for ListTelemetrySchemas {
    const PATH: &'static str = "/dokime.v1.Catalog/ListTelemetrySchemas";
}

impl Endpoint for GetTelemetrySchema {
    const PATH: &'static str = "/dokime.v1.Catalog/GetTelemetrySchema";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
