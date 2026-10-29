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

#[harmos::operation(resources = [crate::hosting::Host<GetCapturedRecordSchema>], response = RecordSchema, id = "dokime.evidence.get_captured_record_schema",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Evidence", method = "GetCapturedRecordSchema"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetCapturedRecordSchema {
    pub record_type: String,
    pub schema_version: u32,

    #[row(
        description = "Required process identity: a type/version from another process must not silently resolve to this process's potentially different admitted schema."
    )]
    pub epoch: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetCapturedRecordSchema);
}
