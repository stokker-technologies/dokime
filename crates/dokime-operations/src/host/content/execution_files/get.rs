/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Execution Files Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::OperationContract;

// Local Imports
use crate::host::ClientSettings;
use crate::{Result, host::Context};
use dokime_protocol::host::content::ExecutionFilesGetRequest;
use dokime_protocol::host::content::ExecutionFilesGetResponse;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.ExecutionFiles/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ExecutionFilesGetRequest, ExecutionFilesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: ExecutionFilesGetRequest) -> Result<ExecutionFilesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let owner = Scope::Execution(&request.parent).identifier()?;
        let (mut reply, cut) =
            crate::host::views::read(context, owner, &format!("file/{}/{}", request.parent, request.name), request.at.as_ref(), async {
                let entry = crate::host::content::execution_file(context, &request.parent, &request.name).await?;
                Ok(ExecutionFilesGetResponse {
                    metadata: entry.metadata,
                    file: entry.file,
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

crate::native::unary!(ExecutionFilesGetRequest, ExecutionFilesGetResponse, context, "dokime.host.v1.content", "ExecutionFiles", "Get");
