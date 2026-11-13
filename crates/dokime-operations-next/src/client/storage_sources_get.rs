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
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Result, client::Context};

/* === Constants === */

pub const ID: &str = "dokime.client.v1.storage.Sources/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<dokime_protocol::client::storage::SourcesGetRequest, dokime_protocol::client::storage::SourcesGetResponse>(
        ID,
        settings.read_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::client::storage::SourcesGetRequest,
) -> Result<dokime_protocol::client::storage::SourcesGetResponse> {
    super::complete(ID, context.settings.read_timeout, async {
        let (mut reply, cut) = super::views::read(context, ID, request.at.as_ref(), async {
            let initial = dokime_protocol::client::storage::StorageSources::default();
            let (payload, revision) = context.host.store().administration_value("storage-sources", &initial.encode_to_vec())?;
            Ok(dokime_protocol::client::storage::SourcesGetResponse {
                value: Some(dokime_protocol::client::storage::StorageSources::decode(payload.as_slice())?),
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
