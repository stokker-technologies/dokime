/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.prompts.get_prompt";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PromptsGetPromptRequest, v1::PromptView>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PromptsGetPromptRequest) -> Result<v1::PromptView> {
    super::complete(ID, context.settings.read_timeout, async {
        super::prompt_catalog::view(context, &request.prompt_id)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
