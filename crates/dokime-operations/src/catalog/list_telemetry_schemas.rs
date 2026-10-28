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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Current admitted schemas, ordered by type/stream ID. These catalogs exclude
// retired definitions retained only for immutable historical evidence. A cursor
// belongs to one runtime epoch/revision; restart pagination when it expires.
#[harmos::operation(resources = [crate::hosting::Host<ListTelemetrySchemas>], response = ListTelemetrySchemasReply, id = "dokime.catalog.list_telemetry_schemas",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Catalog", method = "ListTelemetrySchemas"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListTelemetrySchemas {
    pub limit: u32,
    pub after_stream: String,
    pub cursor: Option<Cursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListTelemetrySchemas);
}
