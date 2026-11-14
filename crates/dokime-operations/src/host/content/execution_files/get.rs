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

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::content::ExecutionFilesGetRequest;
use dokime_protocol::host::content::ExecutionFilesGetResponse;

use harmos_protocol::model::OperationContract;

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
        let (mut reply, cut) =
            crate::host::views::read(context, &format!("file/{}/{}", request.parent, request.name), request.at.as_ref(), async {
                let entry = context.host.store().execution_file(&request.parent, &request.name)?;
                Ok(ExecutionFilesGetResponse {
                    metadata: Some(entry.file_metadata()?),
                    file: Some(crate::host::content::artifacts::get::reference(context, &entry)?),
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
