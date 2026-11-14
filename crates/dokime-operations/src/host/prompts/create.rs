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
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Resources, Result};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Prompts/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration() -> Result<h::OperationContract> {
    crate::declaration::<dokime_protocol::host::PromptsCreateRequest, dokime_protocol::host::PromptsCreateResponse>(
        ID,
        prost_types::FileDescriptorSet::decode(dokime_protocol::FILE_DESCRIPTOR_SET)?,
    )
}
pub async fn handle(
    resources: &Resources, request: dokime_protocol::host::PromptsCreateRequest,
) -> Result<dokime_protocol::host::PromptsCreateResponse> {
    resources.check_action_scope(&resources.execution, ID)?;
    resources.evidence("prompt", request.encode_to_vec(), None)?;
    let answer = resources.prompts.ask(&resources.execution, request).await?;
    resources.evidence("answer", answer.encode_to_vec(), None)?;
    Ok(answer)
}

/* === Private Functions === */

/* === Tests === */
