/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Uploads Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::files::transfer;
use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::files::{UploadsGetRequest, UploadsGetResponse};
use harmos_protocol::model::OperationContract;
use harmos_runtime::builtins::storage::{GetFileProduction, GetFileProductionRequest};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<UploadsGetRequest, UploadsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: UploadsGetRequest) -> Result<UploadsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let upload = request.upload.ok_or_else(|| Error::Invalid(format!("{ID}: Supply Its Upload")))?;
        let file = transfer::staged(&upload)?;
        let response = context.storage::<GetFileProduction>(GetFileProductionRequest { file: Some(file.clone()), at: request.at }).await?;
        let upload = response
            .production
            .map(|value| transfer::progress(file, value, response.writer_generation, response.accepted_length, response.durable_length));
        Ok(UploadsGetResponse {
            upload,
            chunk_bytes: transfer::chunk_bytes(context).await? as u64,
            message_bytes: context.settings.message_bytes as u64,
            resolved_at: response.resolved_at,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(UploadsGetRequest, UploadsGetResponse, context, "dokime.host.v1.files", "Uploads", "Get");
