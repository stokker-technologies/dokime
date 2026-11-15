/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Uploads Create

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::files::transfer;
use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::files::{UploadsCreateRequest, UploadsCreateResponse};
use harmos_protocol::model::OperationContract;
use harmos_runtime::builtins::storage::{CreateFileProduction, CreateFileProductionRequest};
use harmos_runtime::storage::FileMetadata as NativeFileMetadata;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Create";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<UploadsCreateRequest, UploadsCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsCreateRequest) -> Result<UploadsCreateResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let mut metadata = request.metadata.ok_or_else(|| Error::Invalid(format!("{ID}: Supply File Metadata")))?;
        if let Some(caller) = &context.caller {
            let execution = caller.resolve_execution(metadata.scope.as_deref(), ID)?;
            metadata.scope = Some(execution.to_owned());
        }
        let mutation = transfer::mutation(context, ID, &request.request_id).await?;
        if metadata.id.is_empty() {
            metadata.id = mutation.key.to_string();
        }
        let metadata = NativeFileMetadata::try_from(metadata).map_err(|cause| Error::Invalid(format!("{ID}: {cause}")))?;
        let response = context
            .storage::<CreateFileProduction>(CreateFileProductionRequest {
                metadata: Some(metadata),
                mutation: Some(mutation),
            })
            .await?;
        let upload = response.production.map(transfer::upload);
        Ok(UploadsCreateResponse {
            upload,
            receipt: response.receipt,
            chunk_bytes: transfer::chunk_bytes(context).await? as u64,
            message_bytes: context.settings.message_bytes as u64,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(UploadsCreateRequest, UploadsCreateResponse, context, "dokime.host.v1.files", "Uploads", "Create");
