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

use crate::Result;
use crate::host::files::transfer;
use crate::host::{ClientSettings, Context};
use dokime_protocol::host::files::{UploadsPublishRequest, UploadsPublishResponse};
use harmos_protocol::model::OperationContract;
use harmos_operations::storage::{PublishFileProduction, PublishFileProductionRequest};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Publish";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<UploadsPublishRequest, UploadsPublishResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsPublishRequest) -> Result<UploadsPublishResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let session = transfer::session(request.upload)?;
        let mutation = transfer::mutation(context, ID, &request.request_id).await?;
        let response = context
            .storage::<PublishFileProduction>(PublishFileProductionRequest {
                session: Some(session),
                length: request.length,
                sha256: request.sha256,
                mutation: Some(mutation),
            })
            .await?;
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
