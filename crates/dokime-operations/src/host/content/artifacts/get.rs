/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Artifacts Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::content::ArtifactsGetRequest;
use dokime_protocol::host::content::ArtifactsGetResponse;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.Artifacts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ArtifactsGetRequest, ArtifactsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: ArtifactsGetRequest) -> Result<ArtifactsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = crate::host::views::read(context, &format!("artifact/{}", request.name), request.at.as_ref(), async {
            let stored = context.host.store().artifact(&request.name)?;
            Ok(ArtifactsGetResponse {
                metadata: stored.artifact,
                file: stored.request.and_then(|request| request.file),
                ..Default::default()
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ArtifactsGetRequest, ArtifactsGetResponse, context, "dokime.host.v1.content", "Artifacts", "Get");
