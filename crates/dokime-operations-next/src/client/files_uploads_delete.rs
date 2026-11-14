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

use super::{ClientSettings, Context};
use crate::Result;
use dokime_protocol::client::files::{UploadsDeleteRequest, UploadsDeleteResponse};
use harmos_protocol::v1::OperationContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub const ID: &str = "dokime.client.v1.files.Uploads/Delete";
pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<UploadsDeleteRequest, UploadsDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsDeleteRequest) -> Result<UploadsDeleteResponse> {
    super::complete(ID, context.settings.mutation_timeout, async { Ok(context.host.store().delete_file_upload(request)?) }).await
}

/* === Private Functions === */

/* === Tests === */
