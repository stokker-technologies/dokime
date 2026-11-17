/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Delete

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::files::transfer;
use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::{FilesDeleteRequest, FilesDeleteResponse};
use harmos_protocol::model::OperationContract;
use harmos_runtime::builtins::storage::{DeleteFile, DeleteFileRequest};
use harmos_runtime::storage;
use harmos_runtime::storage::StagedFile;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Files/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<FilesDeleteRequest, FilesDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: FilesDeleteRequest) -> Result<FilesDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let file = request.file.ok_or_else(|| Error::Invalid(format!("{ID}: Supply Its Reference")))?;
        transfer::authorize(context, &file)?;
        let _guard = context.administration.lock().await;
        if crate::host::content::retained(context, &file).await? {
            return Err(Error::Precondition(format!("{ID}: Retain Content Attached to Execution Evidence")));
        }
        let file = storage::FileReference::try_from(file).map_err(|cause| Error::Invalid(format!("{ID}: {cause}")))?;
        let file = StagedFile {
            history: file.position.history,
            id: file.metadata.id,
        };
        let mutation = transfer::mutation(context, ID, &request.request_id).await?;
        let response = context
            .storage::<DeleteFile>(DeleteFileRequest {
                file: Some(file),
                mutation: Some(mutation),
            })
            .await?;
        Ok(FilesDeleteResponse { receipt: response.receipt })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(FilesDeleteRequest, FilesDeleteResponse, context, "dokime.host.v1", "Files", "Delete");
