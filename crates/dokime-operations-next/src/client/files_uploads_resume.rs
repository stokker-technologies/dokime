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

use super::{ClientSettings, Context};
use crate::Result;
use dokime_protocol::client::files::{UploadsResumeRequest, UploadsResumeResponse};
use harmos_protocol::v1::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.client.v1.files.Uploads/Resume";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<UploadsResumeRequest, UploadsResumeResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsResumeRequest) -> Result<UploadsResumeResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        Ok(context.host.store().resume_file_upload(
            request,
            super::files_transfer::chunk_bytes(context) as u64,
            context.settings.message_bytes as u64,
        )?)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
