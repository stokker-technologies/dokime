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

#[harmos_domain::operation(response = RecordSchema, id = "dokime.evidence.get_captured_record_schema",
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Evidence", method = "GetCapturedRecordSchema"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCapturedRecordSchema {
    pub record_type: String,
    pub schema_version: u32,

    // Required process identity: a type/version from another process must not
    // silently resolve to this process's potentially different admitted schema.
    #[row(
        description = "Required process identity: a type/version from another process must not silently resolve to this process's potentially different admitted schema."
    )]
    pub epoch: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
