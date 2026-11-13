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
use harmos_protocol::v1 as h;

// Local Imports
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.content.ExecutionFiles/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::content::ExecutionFilesListRequest, v1::content::ExecutionFilesListResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::content::ExecutionFilesListRequest) -> Result<v1::content::ExecutionFilesListResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        super::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let (saved, cut) = super::views::read(context, &format!("files/{}", request.parent), request.at.as_ref(), async {
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
            Ok(v1::content::ExecutionFilesListResponse { files, ..Default::default() })
        })
        .await?;
        let (files, next_page_token) =
            super::views::page(saved.files, &cut, request.page.as_ref(), context.settings.page_items, context.settings.message_bytes)?;
        super::bounded_reply(
            context,
            v1::content::ExecutionFilesListResponse {
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
