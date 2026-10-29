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

#[harmos::operation(resources = [crate::hosting::Host<GetTelemetrySchema>], response = TelemetrySchema, id = "dokime.catalog.get_telemetry_schema",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Catalog", method = "GetTelemetrySchema"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTelemetrySchema {
    pub stream: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetTelemetrySchema);
}
