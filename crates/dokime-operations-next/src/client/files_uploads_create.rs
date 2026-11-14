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

use super::{ClientSettings, Context};
use crate::Result;
use dokime_protocol::client::files::{UploadsCreateRequest, UploadsCreateResponse};
use harmos_protocol::v1::OperationContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub const ID: &str = "dokime.client.v1.files.Uploads/Create";
pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<UploadsCreateRequest, UploadsCreateResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsCreateRequest) -> Result<UploadsCreateResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        Ok(context.host.store().create_file_upload(
            request,
            super::files_transfer::chunk_bytes(context) as u64,
            context.settings.message_bytes as u64,
        )?)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
