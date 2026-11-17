/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Content Execution Files List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model::OperationContract;

// Local Imports
use crate::host::ClientSettings;
use crate::{Error, Result, host::Context};
use dokime_protocol::host::content::ExecutionFilesListRequest;
use dokime_protocol::host::content::ExecutionFilesListResponse;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.content.ExecutionFiles/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<ExecutionFilesListRequest, ExecutionFilesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: ExecutionFilesListRequest) -> Result<ExecutionFilesListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let owner = Scope::Execution(&request.parent).identifier()?;
        let (saved, cut) = crate::host::views::read(context, owner, &format!("files/{}", request.parent), request.at.as_ref(), async {
            let mut files = Vec::new();
            for entry in crate::host::content::execution_files(context, &request.parent).await? {
                files.push(entry.metadata.ok_or_else(|| Error::Precondition("List Execution Files: Restore Its Metadata".into()))?);
            }
            Ok(ExecutionFilesListResponse { files, ..Default::default() })
        })
        .await?;
        let (files, next_page_token) = crate::host::views::page(
            saved.files,
            &cut,
            request.page.as_ref(),
            context.settings.page_items,
            context.settings.message_bytes,
        )?;
        crate::host::bounded_reply(
            context,
            ExecutionFilesListResponse {
                files,
                resolved_at: Some(cut),
                next_page_token,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(ExecutionFilesListRequest, ExecutionFilesListResponse, context, "dokime.host.v1.content", "ExecutionFiles", "List");
