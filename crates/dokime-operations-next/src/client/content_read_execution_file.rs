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
use crate::{Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.content.read_execution_file";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ContentReadExecutionFileRequest,v1::ExecutionFileWindow>(ID,settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ContentReadExecutionFileRequest) -> Result<v1::ExecutionFileWindow> {
    super::complete(ID,context.settings.read_timeout,async {
        let maximum=super::content_transfer::maximum(context,request.max_bytes)?;
        let store=context.host.store();
        let entry=store.execution_file(&request.execution_id,&request.relative_path)?;
        let metadata=entry.file_metadata()?;
        let offset=request.offset.min(entry.length);
        let data=store.content_window(&entry.identifier,offset,maximum)?;
        Ok(v1::ExecutionFileWindow {relative_path:entry.path,content_type:metadata.content_type,kind:metadata.kind,state:metadata.state,offset,observed_size_bytes:entry.length,data})
    }).await
}

/* === Private Functions === */

/* === Tests === */
