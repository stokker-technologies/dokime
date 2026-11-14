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

use super::{ClientSettings, Context};
use crate::Result;
use dokime_protocol::client::files::{UploadsPublishRequest, UploadsPublishResponse};
use harmos_protocol::v1::OperationContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub const ID: &str = "dokime.client.v1.files.Uploads/Publish";
pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<UploadsPublishRequest, UploadsPublishResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: UploadsPublishRequest) -> Result<UploadsPublishResponse> {
    super::complete(ID, context.settings.mutation_timeout, async { Ok(context.host.store().publish_file_upload(request)?) }).await
}

/* === Private Functions === */

/* === Tests === */
