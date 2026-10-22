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

#[harmos_domain::message]
#[harmos_domain::operation(response = TelemetrySchema, id = "dokime.catalog.get_telemetry_schema", version = 1,
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Catalog", method = "GetTelemetrySchema"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTelemetrySchema {
    pub stream: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
