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
use prost::Message;

// Local Imports
use crate::{Resources, Result};
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Prompts/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(resources: &Resources, request: PromptsCreateRequest) -> Result<PromptsCreateResponse> {
    resources.check_action_scope(&resources.execution, ID)?;
    resources.evidence("prompt", request.encode_to_vec(), None)?;
    let answer = resources.prompts.ask(&resources.execution, request).await?;
    resources.evidence("answer", answer.encode_to_vec(), None)?;
    Ok(answer)
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(PromptsCreateRequest, PromptsCreateResponse, resources, "dokime.host.v1", "Prompts", "Create");
