/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Uploads Publish

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_operations::storage::{PublishFileProduction, PublishFileProductionRequest};

// Local Imports
use crate::Result;
use crate::host::Context;
use crate::host::files::transfer;
use dokime_protocol::host::files::{UploadsPublishRequest, UploadsPublishResponse};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Publish";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: UploadsPublishRequest) -> Result<UploadsPublishResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let session = transfer::session(request.upload)?;
        let mutation = transfer::mutation(context, ID, &request.request_id).await?;
        let mut native = PublishFileProductionRequest {
            session: Some(session),
            length: request.length,
            sha256: request.sha256,
            mutation: Some(mutation),
        };
        let response = match context.storage::<PublishFileProduction>(native.clone()).await {
            Err(source) if transfer::expired(&source) => {
                native.mutation = Some(transfer::mutation(context, ID, &request.request_id).await?);
                context.storage::<PublishFileProduction>(native).await?
            }
            response => response?,
        };
        Ok(UploadsPublishResponse {
            upload: response.file.map(Into::into),
            receipt: response.receipt,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(UploadsPublishRequest, UploadsPublishResponse, context, "dokime.host.v1.files", "Uploads", "Publish");
