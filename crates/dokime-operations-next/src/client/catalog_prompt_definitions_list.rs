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

pub const ID: &str = "dokime.v1.catalog.PromptDefinitions/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::catalog::PromptDefinitionsListRequest, v1::catalog::PromptDefinitionsListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::catalog::PromptDefinitionsListRequest) -> Result<v1::catalog::PromptDefinitionsListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
let (mut reply,cut)=super::views::read(context,&format!("catalog/prompts/{}",request.parent),request.at.as_ref(),async { Ok(v1::catalog::PromptDefinitionsListResponse {values:super::prompt_catalog::definitions(context)?,..Default::default()}) }).await?;
reply.resolved_at=Some(cut);Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
