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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.Artifacts/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<
        dokime_protocol::host::content::ArtifactsCreateRequest,
        dokime_protocol::host::content::ArtifactsCreateResponse,
    >(ID, settings.mutation_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::content::ArtifactsCreateRequest,
) -> Result<dokime_protocol::host::content::ArtifactsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let reply = context.host.store().attach_artifact(ID, request, context.host.now_ns())?;
        context.host.signal_change();
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
