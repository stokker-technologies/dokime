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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.content.ExecutionFiles/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::content::ExecutionFilesGetRequest, v1::content::ExecutionFilesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::content::ExecutionFilesGetRequest) -> Result<v1::content::ExecutionFilesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            super::views::read(context, &format!("file/{}/{}", request.parent, request.name), request.at.as_ref(), async {
                let entry = context.host.store().execution_file(&request.parent, &request.name)?;
                Ok(v1::content::ExecutionFilesGetResponse {
                    metadata: Some(entry.file_metadata()?),
                    file: Some(super::content_artifacts_get::reference(context, &entry)?),
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
