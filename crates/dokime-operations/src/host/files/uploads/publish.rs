/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Uploads Publish

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::Result;
use crate::host::{ClientSettings, Context};
use dokime_protocol::host::files::UploadsPublishRequest;
use dokime_protocol::host::files::UploadsPublishResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Publish";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<UploadsPublishRequest, UploadsPublishResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsPublishRequest) -> Result<UploadsPublishResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async { Ok(context.host.store().publish_file_upload(request)?) }).await
}

/* === Private Functions === */

/* === Tests === */
