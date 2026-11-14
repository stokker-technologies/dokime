/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Sources Get

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.storage.Sources/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::storage::SourcesGetRequest, dokime_protocol::host::storage::SourcesGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::storage::SourcesGetRequest,
) -> Result<dokime_protocol::host::storage::SourcesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = crate::host::views::read(context, ID, request.at.as_ref(), async {
            let initial = dokime_protocol::host::storage::StorageSources::default();
            let (payload, revision) = context.host.store().administration_value("storage-sources", &initial.encode_to_vec())?;
            Ok(dokime_protocol::host::storage::SourcesGetResponse {
                value: Some(dokime_protocol::host::storage::StorageSources::decode(payload.as_slice())?),
                revision,
                ..Default::default()
            })
        })
        .await?;
        reply.resolved_at = Some(cut);
        Ok(reply)
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
