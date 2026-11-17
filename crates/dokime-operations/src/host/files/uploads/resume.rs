/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Uploads Resume

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::Result;
use crate::host::files::transfer;
use crate::host::{ClientSettings, Context};
use dokime_protocol::host::files::{UploadsResumeRequest, UploadsResumeResponse};
use harmos_protocol::model::OperationContract;
use harmos_operations::storage::{CreateFileWriter, CreateFileWriterRequest};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Resume";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<UploadsResumeRequest, UploadsResumeResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsResumeRequest) -> Result<UploadsResumeResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let session = transfer::session(request.upload)?;
        let mutation = transfer::mutation(context, ID, &request.request_id).await?;
        let response = context
            .storage::<CreateFileWriter>(CreateFileWriterRequest {
                session: Some(session),
                mutation: Some(mutation),
            })
            .await?;
        Ok(UploadsResumeResponse {
            upload: response.production.map(transfer::upload),
            receipt: response.receipt,
            chunk_bytes: transfer::chunk_bytes(context).await? as u64,
            message_bytes: context.settings.message_bytes as u64,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(UploadsResumeRequest, UploadsResumeResponse, context, "dokime.host.v1.files", "Uploads", "Resume");
