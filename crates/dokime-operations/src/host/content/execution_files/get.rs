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

/* === Constants === */

pub const ID: &str = "dokime.client.v1.content.ExecutionFiles/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<
        dokime_protocol::client::content::ExecutionFilesGetRequest,
        dokime_protocol::client::content::ExecutionFilesGetResponse,
    >(ID, settings.read_timeout)
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::content::ExecutionFilesGetRequest,
) -> Result<dokime_protocol::client::content::ExecutionFilesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) =
            super::views::read(context, &format!("file/{}/{}", request.parent, request.name), request.at.as_ref(), async {
                let entry = context.host.store().execution_file(&request.parent, &request.name)?;
                Ok(dokime_protocol::client::content::ExecutionFilesGetResponse {
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
