/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<ListTelemetrySchemas>], response = ListTelemetrySchemasReply, id = "dokime.catalog.list_telemetry_schemas",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Catalog", method = "ListTelemetrySchemas"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListTelemetrySchemas {
    pub limit: u32,
    pub after_stream: String,
    pub cursor: Option<Cursor>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(ListTelemetrySchemas);
}
