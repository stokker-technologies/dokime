/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::FilesGetRequest;
use dokime_protocol::host::FilesGetResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Files/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<FilesGetRequest, FilesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: FilesGetRequest) -> Result<FilesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let selected = request.file.as_ref().ok_or_else(|| Error::Invalid("Get File: Supply Its Reference".into()))?;
        let id = crate::host::files::transfer::identifier(Some(selected))?;
        let (mut response, cut) = crate::host::views::read(context, &format!("file/{id}"), request.at.as_ref(), async {
            let file = crate::host::files::transfer::reference(context, selected)?;
            Ok(FilesGetResponse {
                file: Some(file),
                chunk_bytes: crate::host::files::transfer::chunk_bytes(context) as u64,
                message_bytes: context.settings.message_bytes as u64,
                resolved_at: None,
            })
        })
        .await?;
        response.resolved_at = Some(cut);
        crate::host::bounded_reply(context, response)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
