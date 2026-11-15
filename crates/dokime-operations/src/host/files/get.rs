/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::files::transfer;
use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::{FilesGetRequest, FilesGetResponse};
use harmos_protocol::model::OperationContract;
use harmos_runtime::builtins::storage::{GetFileBytes, GetFileBytesRequest};
use harmos_runtime::storage::FileReference as NativeFileReference;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Files/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<FilesGetRequest, FilesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: FilesGetRequest) -> Result<FilesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let file = request.file.ok_or_else(|| Error::Invalid(format!("{ID}: Supply Its Reference")))?;
        transfer::authorize(context, &file)?;
        let selected = NativeFileReference::try_from(file.clone()).map_err(|cause| Error::Invalid(format!("{ID}: {cause}")))?;
        let response = context
            .storage::<GetFileBytes>(GetFileBytesRequest {
                file: Some(selected),
                offset: file.length,
                max_bytes: 1,
                at: request.at,
            })
            .await?;
        crate::host::bounded_reply(
            context,
            FilesGetResponse {
                file: Some(file),
                chunk_bytes: transfer::chunk_bytes(context).await? as u64,
                message_bytes: context.settings.message_bytes as u64,
                resolved_at: response.resolved_at,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(FilesGetRequest, FilesGetResponse, context, "dokime.host.v1", "Files", "Get");
