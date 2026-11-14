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

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::content::ExecutionFilesListRequest;
use dokime_protocol::host::content::ExecutionFilesListResponse;

use harmos_protocol::model::OperationContract;

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
        let (saved, cut) = crate::host::views::read(context, &format!("files/{}", request.parent), request.at.as_ref(), async {
            let store = context.host.store();
            store.metadata(&request.parent)?;
            let mut after = String::new();
            let mut files = Vec::new();
            loop {
                let page = store.files_page(&request.parent, &after, context.settings.page_items)?;
                if page.is_empty() {
                    break;
                }
                after = page.last().unwrap().path.clone();
                for entry in page {
                    files.push(entry.file_metadata()?);
                }
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
