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

use crate::Result;
use crate::host::{ClientSettings, Context};
use dokime_protocol::host::FilesDeleteRequest;
use dokime_protocol::host::FilesDeleteResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Files/Delete";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<FilesDeleteRequest, FilesDeleteResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: FilesDeleteRequest) -> Result<FilesDeleteResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async { Ok(context.host.store().delete_file(request)?) }).await
}

/* === Private Functions === */

/* === Tests === */
