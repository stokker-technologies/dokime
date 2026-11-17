/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Uploads Append

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::files::transfer;
use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::files::{UploadsAppendRequest, UploadsAppendResponse};
use harmos_operations::storage::{AppendFileBytes, AppendFileBytesRequest};
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Append";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<UploadsAppendRequest, UploadsAppendResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsAppendRequest) -> Result<UploadsAppendResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        if request.data.len() > transfer::chunk_bytes(context).await? {
            return Err(Error::Bound(format!("{ID}: Respect the Returned Chunk Bound")));
        }
        let session = transfer::session(request.upload)?;
        let mutation = transfer::mutation(context, ID, &request.request_id).await?;
        let response = context
            .storage::<AppendFileBytes>(AppendFileBytesRequest {
                session: Some(session),
                offset: request.offset,
                data: request.data,
                mutation: Some(mutation),
            })
            .await?;
        Ok(UploadsAppendResponse {
            upload: response.production.map(transfer::upload),
            receipt: response.receipt,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(UploadsAppendRequest, UploadsAppendResponse, context, "dokime.host.v1.files", "Uploads", "Append");
