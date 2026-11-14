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
use crate::host::{ClientSettings, Context};
use dokime_protocol::host::files::UploadsDeleteRequest;
use dokime_protocol::host::files::UploadsDeleteResponse;
use harmos_protocol::model::OperationContract;

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
    crate::host::complete(ID, context.settings.mutation_timeout, async { Ok(context.host.store().delete_file_upload(request)?) }).await
}

/* === Private Functions === */

/* === Tests === */
