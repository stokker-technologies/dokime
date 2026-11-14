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
use harmos_protocol::model as h;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.ExecutionFiles/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<
        dokime_protocol::host::content::ExecutionFilesGetRequest,
        dokime_protocol::host::content::ExecutionFilesGetResponse,
    >(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::content::ExecutionFilesGetRequest,
) -> Result<dokime_protocol::host::content::ExecutionFilesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            crate::host::views::read(context, &format!("file/{}/{}", request.parent, request.name), request.at.as_ref(), async {
                let entry = context.host.store().execution_file(&request.parent, &request.name)?;
                Ok(dokime_protocol::host::content::ExecutionFilesGetResponse {
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
