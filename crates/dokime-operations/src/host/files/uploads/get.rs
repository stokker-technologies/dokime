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

use crate::host::{ClientSettings, Context};
use crate::{Error, Result};
use dokime_protocol::host::files::UploadsGetRequest;
use dokime_protocol::host::files::UploadsGetResponse;
use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.files.Uploads/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<UploadsGetRequest, UploadsGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: UploadsGetRequest) -> Result<UploadsGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let file = request.file.as_ref().ok_or_else(|| Error::Invalid("Get Upload: Supply Its Identity and Store History".into()))?;
        let key = format!("upload/{}", file.id);
        let (mut response, cut) = crate::host::views::read(context, &key, request.at.as_ref(), async {
            let (_, progress, generation) = context.host.store().file_upload(file)?;
            let length = progress.length;
            Ok(UploadsGetResponse {
                production: Some(progress),
                writer_generation: generation,
                accepted_length: length,
                durable_length: length,
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
