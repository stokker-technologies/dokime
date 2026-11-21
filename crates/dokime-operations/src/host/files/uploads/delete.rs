/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Uploads Delete

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_operations::storage::{DeleteFileProduction, DeleteFileProductionRequest};
use harmos_runtime::storage::StagedFile;

// Local Imports
use crate::Result;
use crate::host::Context;
use crate::host::files::transfer;
use dokime_protocol::host::files::{UploadsDeleteRequest, UploadsDeleteResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: UploadsDeleteRequest) -> Result<UploadsDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let session = transfer::session(request.upload)?;
        let file = StagedFile {
            history: session.history.clone(),
            id: session.id.clone(),
        };
        let mutation = transfer::mutation(context, ID, &request.request_id).await?;
        let mut native = DeleteFileProductionRequest {
            session: Some(session),
            mutation: Some(mutation),
        };
        let response = match context.storage::<DeleteFileProduction>(native.clone()).await {
            Err(source) if transfer::expired(&source) => {
                native.mutation = Some(transfer::mutation(context, ID, &request.request_id).await?);
                context.storage::<DeleteFileProduction>(native).await?
            }
            response => response?,
        };
        let upload = response.production.map(|value| transfer::progress(file, value, 0, value.length, value.length));
        Ok(UploadsDeleteResponse { upload, receipt: response.receipt })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(UploadsDeleteRequest, UploadsDeleteResponse, context, "dokime.host.v1.files", "Uploads", "Delete");
