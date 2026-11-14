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

use super::{ClientSettings, Context};
use crate::Result;
use dokime_protocol::client::files::{UploadsAppendRequest, UploadsAppendResponse};
use harmos_protocol::v1::OperationContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub const ID: &str = "dokime.client.v1.files.Uploads/Append";
pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<UploadsAppendRequest, UploadsAppendResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsAppendRequest) -> Result<UploadsAppendResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        Ok(context.host.store().append_file_upload(request, super::files_transfer::chunk_bytes(context))?)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
