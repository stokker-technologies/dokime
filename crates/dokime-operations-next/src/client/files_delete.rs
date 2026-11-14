/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Delete

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::{ClientSettings, Context};
use crate::Result;
use dokime_protocol::client::{FilesDeleteRequest, FilesDeleteResponse};
use harmos_protocol::v1::OperationContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub const ID: &str = "dokime.client.v1.Files/Delete";
pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<FilesDeleteRequest, FilesDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: FilesDeleteRequest) -> Result<FilesDeleteResponse> {
    super::complete(ID, context.settings.mutation_timeout, async { Ok(context.host.store().delete_file(request)?) }).await
}

/* === Private Functions === */

/* === Tests === */
