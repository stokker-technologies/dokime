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

use super::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::client::{FilesGetRequest, FilesGetResponse};
use harmos_protocol::v1::OperationContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub const ID: &str = "dokime.client.v1.Files/Get";
pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<FilesGetRequest, FilesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: FilesGetRequest) -> Result<FilesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let selected = request.file.as_ref().ok_or_else(|| Error::Invalid("Get File: Supply Its Reference".into()))?;
        let id = super::files_transfer::identifier(Some(selected))?;
        let (mut response, cut) = super::views::read(context, &format!("file/{id}"), request.at.as_ref(), async {
            let file = super::files_transfer::reference(context, selected)?;
            Ok(FilesGetResponse {
                file: Some(file),
                chunk_bytes: super::files_transfer::chunk_bytes(context) as u64,
                message_bytes: context.settings.message_bytes as u64,
                resolved_at: None,
            })
        })
        .await?;
        response.resolved_at = Some(cut);
        super::bounded_reply(context, response)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
