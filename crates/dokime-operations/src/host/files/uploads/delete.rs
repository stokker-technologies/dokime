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

use crate::Result;
use crate::host::files::transfer;
use crate::host::{ClientSettings, Context};
use dokime_protocol::host::files::{UploadsDeleteRequest, UploadsDeleteResponse};
use harmos_protocol::model::OperationContract;
use harmos_runtime::builtins::storage::{DeleteFileProduction, DeleteFileProductionRequest};
use harmos_runtime::storage::StagedFile;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<UploadsDeleteRequest, UploadsDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsDeleteRequest) -> Result<UploadsDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let session = transfer::session(request.upload)?;
        let file = StagedFile {
            history: session.history.clone(),
            id: session.id.clone(),
        };
        let mutation = transfer::mutation(context, ID, &request.request_id).await?;
        let response = context
            .storage::<DeleteFileProduction>(DeleteFileProductionRequest {
                session: Some(session),
                mutation: Some(mutation),
            })
            .await?;
        let upload = response.production.map(|value| transfer::progress(file, value, 0, value.length, value.length));
        Ok(UploadsDeleteResponse { upload, receipt: response.receipt })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
