/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Resources, Result};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.execution.progress";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<v1::ReportProgress, v1::ReportProgressResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: v1::ReportProgress) -> Result<v1::ReportProgressResponse> {
    resources.check_scope(&request.execution)?;
    if request.completed > request.total {
        return Err(crate::Error::Invalid("Keep Completed Progress Within Its Total".into()));
    }
    resources.evidence("progress", request.encode_to_vec(), None)?;
    Ok(v1::ReportProgressResponse {})
}

/* === Private Functions === */

/* === Tests === */
