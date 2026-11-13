/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Artifacts Create

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

pub const ID: &str = "dokime.v1.content.Artifacts/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::content::ArtifactsCreateRequest, v1::content::ArtifactsCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::content::ArtifactsCreateRequest) -> Result<v1::content::ArtifactsCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let reply = context.host.store().attach_artifact(ID, request, context.host.now_ns())?;
        context.host.signal_change();
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
