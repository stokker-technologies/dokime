/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Execution Progress at a Returned Durable Position

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;


/* === Constants === */

pub const ID: &str = "dokime.v1.Prompts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::PromptsGetRequest, v1::PromptsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PromptsGetRequest) -> Result<v1::PromptsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let (prompt,cut)=super::views::read(context,&format!("prompt/{}",request.name),request.at.as_ref(),async {super::prompt_catalog::view(context,&request.name)}).await?;
Ok(v1::PromptsGetResponse {prompt:Some(prompt),resolved_at:Some(cut)})
    }).await
}

/* === Private Functions === */

/* === Tests === */
