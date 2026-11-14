/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Read

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::FilesReadRequest;
use dokime_protocol::host::FilesReadResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Files/Read";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<FilesReadRequest, FilesReadResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: FilesReadRequest) -> Result<FilesReadResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let maximum = crate::host::files::transfer::chunk_bytes(context);
        let maximum = if request.max_bytes == 0 {
            maximum
        } else {
            usize::try_from(request.max_bytes).map_err(|_| Error::Bound("Read File: Respect Its Chunk Bound".into()))?
        };
        if maximum > crate::host::files::transfer::chunk_bytes(context) {
            return Err(Error::Bound("Read File: Respect the Returned Chunk Bound".into()));
        }
        let selected = crate::host::files::get::handle(context, FilesGetRequest { file: request.file, at: request.at }).await?;
        let file = selected.file.ok_or_else(|| Error::Invalid("Read File: Select Its Reference".into()))?;
        let data = crate::host::files::transfer::read(context, &file, request.offset, maximum)?;
        crate::host::bounded_reply(
            context,
            FilesReadResponse {
                data,
                total_length: file.length,
                sha256: file.sha256,
                resolved_at: selected.resolved_at,
            },
        )
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
