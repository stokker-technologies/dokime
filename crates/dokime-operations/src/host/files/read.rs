/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Read

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::files::transfer;
use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::{FilesReadRequest, FilesReadResponse};
use harmos_protocol::model::OperationContract;
use harmos_runtime::builtins::storage::{GetFileBytes, GetFileBytesRequest};
use harmos_runtime::storage::FileReference as NativeFileReference;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Files/Read";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<FilesReadRequest, FilesReadResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: FilesReadRequest) -> Result<FilesReadResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let file = request.file.ok_or_else(|| Error::Invalid(format!("{ID}: Supply Its Reference")))?;
        transfer::authorize(context, &file)?;
        let maximum = transfer::chunk_bytes(context).await? as u64;
        let max_bytes = if request.max_bytes == 0 { maximum } else { request.max_bytes };
        if max_bytes > maximum {
            return Err(Error::Bound(format!("{ID}: Respect the Returned Chunk Bound")));
        }
        let file = NativeFileReference::try_from(file).map_err(|cause| Error::Invalid(format!("{ID}: {cause}")))?;
        let response = context
            .storage::<GetFileBytes>(GetFileBytesRequest {
                file: Some(file),
                offset: request.offset,
                max_bytes,
                at: request.at,
            })
            .await?;
        crate::host::bounded_reply(
            context,
            FilesReadResponse {
                data: response.data,
                total_length: response.total_length,
                sha256: response.sha256,
                resolved_at: response.resolved_at,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(FilesReadRequest, FilesReadResponse, context, "dokime.host.v1", "Files", "Read");
