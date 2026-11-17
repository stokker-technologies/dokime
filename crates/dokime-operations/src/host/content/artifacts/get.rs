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

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::content::ArtifactsGetRequest;
use dokime_protocol::host::content::ArtifactsGetResponse;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.Artifacts/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ArtifactsGetRequest) -> Result<ArtifactsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let stored = crate::host::content::artifact(context, &request.name).await?;
        let execution = stored
            .request
            .as_ref()
            .and_then(|request| request.scope.as_ref())
            .and_then(|scope| scope.execution_id.as_deref())
            .ok_or_else(|| Error::Precondition("Get Artifact: Restore Its Execution Binding".into()))?;
        let scope = Scope::Execution(execution).identifier()?;
        let (mut reply, cut) =
            crate::host::views::read(context, scope, &format!("artifact/{}", request.name), request.at.as_ref(), async {
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
