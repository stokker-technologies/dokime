/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use prost::Message;

/* === Constants === */

pub const ID: &str = "dokime.content.list_execution_files";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ContentListExecutionFilesRequest,v1::ListExecutionFilesReply>(ID,settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ContentListExecutionFilesRequest) -> Result<v1::ListExecutionFilesReply> {
    super::complete(ID,context.settings.read_timeout,async {
        let store=context.host.store();
        store.metadata(&request.execution_id)?;
        let cursor=v1::ExecutionFilesCursor {epoch:context.host.identity().epoch,execution_id:request.execution_id.clone(),metadata_sha256:store.content_fingerprint(Some(&request.execution_id))?};
        if request.cursor.as_ref().is_some_and(|given|given!=&cursor) || !request.after_path.is_empty() && request.cursor.is_none() {
            return Err(Error::Invalid("List Execution Files: Restart Pagination After Its Owner, Epoch or Inventory Changed".into()));
        }
        let limit=if request.limit==0 {context.settings.page_items} else {(request.limit as usize).min(context.settings.page_items)};
        let mut files=store.files_page(&request.execution_id,&request.after_path,limit+1)?.iter().map(|entry|entry.file_metadata()).collect::<dokime_storage::Result<Vec<_>>>()?;
        let more=files.len()>limit; files.truncate(limit);
        let next_after_path=if more {files.last().map(|item|item.relative_path.clone())} else {None};
        let reply=v1::ListExecutionFilesReply {files,cursor:Some(cursor),next_after_path};
        if reply.encoded_len()>context.settings.message_bytes {return Err(Error::Invalid("List Execution Files: Reduce the Page Size".into()));}
        Ok(reply)
    }).await
}

/* === Private Functions === */

/* === Tests === */
