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

use crate::Result;
use crate::host::{ClientSettings, Context};
use dokime_protocol::host::files::UploadsCreateRequest;
use dokime_protocol::host::files::UploadsCreateResponse;
use harmos_protocol::model::OperationContract;

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
        Ok(context.host.store().create_file_upload(
            request,
            crate::host::files::transfer::chunk_bytes(context) as u64,
            context.settings.message_bytes as u64,
        )?)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
