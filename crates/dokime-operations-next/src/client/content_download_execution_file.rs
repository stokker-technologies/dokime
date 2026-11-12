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

pub const ID: &str = "dokime.content.download_execution_file";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::ContentDownloadExecutionFileRequest,v1::ContentDownloadExecutionFileOutput>(ID,settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::ContentDownloadExecutionFileRequest) -> Result<super::Output<v1::ContentDownloadExecutionFileOutput>> {
    super::complete(ID,context.settings.read_timeout,async {
        let store=context.host.store();
        let entry=store.execution_file(&request.execution_id,&request.relative_path)?;
        let file=entry.file_metadata()?;
        let metadata=v1::ExecutionFileDownloadMetadata {relative_path:entry.path,content_type:file.content_type,kind:file.kind,state:file.state,observed_size_bytes:entry.length};
        let source=super::content_transfer::Source::Stored {store,identifier:entry.identifier};
        let length=entry.length;
        let digest=if entry.sha256.is_empty(){None}else{Some(entry.sha256)};
        super::content_transfer::download(context,source,length,digest,
            v1::ContentDownloadExecutionFileOutput {value:Some(v1::content_download_execution_file_output::Value::Item(v1::ExecutionFileDownloadFrame {payload:Some(v1::ExecutionFileDownloadFramePayload {kind:Some(v1::execution_file_download_frame_payload::Kind::Metadata(metadata))})}))},
            |chunk|v1::ContentDownloadExecutionFileOutput {value:Some(v1::content_download_execution_file_output::Value::Item(v1::ExecutionFileDownloadFrame {payload:Some(v1::ExecutionFileDownloadFramePayload {kind:Some(v1::execution_file_download_frame_payload::Kind::Chunk(chunk))})}))},
            |complete|v1::ContentDownloadExecutionFileOutput {value:Some(v1::content_download_execution_file_output::Value::Item(v1::ExecutionFileDownloadFrame {payload:Some(v1::ExecutionFileDownloadFramePayload {kind:Some(v1::execution_file_download_frame_payload::Kind::Complete(complete))})}))},
            ||v1::ContentDownloadExecutionFileOutput {value:Some(v1::content_download_execution_file_output::Value::Response(v1::ClientEmpty {}))})

    }).await
}

/* === Private Functions === */

/* === Tests === */
