/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Files Uploads Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::client::files::{UploadsGetRequest, UploadsGetResponse};
use harmos_protocol::v1::OperationContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub const ID: &str = "dokime.client.v1.files.Uploads/Get";
pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    super::declaration::<UploadsGetRequest, UploadsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: UploadsGetRequest) -> Result<UploadsGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let file = request.file.as_ref().ok_or_else(|| Error::Invalid("Get Upload: Supply Its Identity and Store History".into()))?;
        let key = format!("upload/{}", file.id);
        let (mut response, cut) = super::views::read(context, &key, request.at.as_ref(), async {
            let (_, progress, generation) = context.host.store().file_upload(file)?;
            let length = progress.length;
            Ok(UploadsGetResponse {
                production: Some(progress),
                writer_generation: generation,
                accepted_length: length,
                durable_length: length,
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
