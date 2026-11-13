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

/* === Constants === */

pub const ID: &str = "dokime.context.v1.evidence.Logs/Append";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<dokime_protocol::context::evidence::LogsAppendRequest, dokime_protocol::context::evidence::LogsAppendResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(
    resources: &Resources, request: dokime_protocol::context::evidence::LogsAppendRequest,
) -> Result<dokime_protocol::context::evidence::LogsAppendResponse> {
    resources.check_action_scope(&request.execution, ID)?;
    let source = request.at.as_ref().map(crate::resources::timestamp_ns).transpose()?;
    resources.evidence("log", request.encode_to_vec(), source)?;
    Ok(dokime_protocol::context::evidence::LogsAppendResponse {})
}

/* === Private Functions === */

/* === Tests === */
