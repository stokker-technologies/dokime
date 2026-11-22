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

// Standard Library Imports

// Third Party Imports
use harmos_operations::storage::{CreateFileWriter, CreateFileWriterRequest};

// Local Imports
use crate::Result;
use crate::host::Context;
use crate::host::files::transfer;
use dokime_protocol::host::files::{UploadsResumeRequest, UploadsResumeResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Resume";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: UploadsResumeRequest) -> Result<UploadsResumeResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let session = transfer::session(request.upload)?;
        let (maximum, mutation) = transfer::admission(context, ID, &request.request_id).await?;
        let native = CreateFileWriterRequest {
            session: Some(session),
            mutation: None,
        };
        let response = transfer::write(context, ID, &request.request_id, mutation, |mutation| {
            context.storage::<CreateFileWriter>(CreateFileWriterRequest { mutation: Some(mutation), ..native.clone() })
        })
        .await?;
        Ok(UploadsResumeResponse {
            upload: response.production.map(transfer::upload),
            receipt: response.receipt,
            chunk_bytes: maximum as u64,
            message_bytes: context.settings.message_bytes as u64,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(UploadsResumeRequest, UploadsResumeResponse, context, "dokime.host.v1.files", "Uploads", "Resume");
