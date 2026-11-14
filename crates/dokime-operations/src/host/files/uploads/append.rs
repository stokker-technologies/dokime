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

use crate::Result;
use crate::host::{ClientSettings, Context};
use dokime_protocol::host::files::UploadsAppendRequest;
use dokime_protocol::host::files::UploadsAppendResponse;
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
        Ok(context.host.store().append_file_upload(request, crate::host::files::transfer::chunk_bytes(context))?)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
