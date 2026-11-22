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

// Standard Library Imports

// Third Party Imports
use harmos_operations::storage::{DeleteFile, DeleteFileRequest};
use harmos_runtime::storage;
use harmos_runtime::storage::StagedFile;

// Local Imports
use crate::host::Context;
use crate::host::files::transfer;
use crate::{Error, Result};
use dokime_protocol::host::{FilesDeleteRequest, FilesDeleteResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Files/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

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
        let native = DeleteFileRequest { file: Some(file), mutation: None };
        let response = transfer::write(context, ID, &request.request_id, mutation, |mutation| {
            context.storage::<DeleteFile>(DeleteFileRequest {
                mutation: Some(mutation),
                ..native.clone()
            })
        })
        .await?;
        Ok(FilesDeleteResponse { receipt: response.receipt })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(FilesDeleteRequest, FilesDeleteResponse, context, "dokime.host.v1", "Files", "Delete");
