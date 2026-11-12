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

pub const ID: &str = "dokime.execution.log";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<v1::WriteLog, v1::WriteLogResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(resources: &Resources, request: v1::WriteLog) -> Result<v1::WriteLogResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let source = request.at.as_ref().map(crate::resources::timestamp_ns).transpose()?;
    resources.evidence("log", request.encode_to_vec(), source)?;
    Ok(v1::WriteLogResponse {})
}

/* === Private Functions === */

/* === Tests === */
