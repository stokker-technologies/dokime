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

use crate::host::ClientSettings;

// Standard Library Imports

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Result, host::Context};

use dokime_protocol::host::storage::SourcesGetRequest;
use dokime_protocol::host::storage::SourcesGetResponse;
use dokime_protocol::host::storage::StorageSources;

use harmos_protocol::model::OperationContract;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.storage.Sources/Get";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &ClientSettings) -> Result<OperationContract> {
    crate::host::declaration::<SourcesGetRequest, SourcesGetResponse>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: SourcesGetRequest) -> Result<SourcesGetResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        let initial = StorageSources::default();
        let snapshot = crate::host::storage::administration::read(context, "storage-sources", &initial.encode_to_vec(), request.at).await?;
        Ok(SourcesGetResponse {
            value: Some(StorageSources::decode(snapshot.payload.as_slice())?),
            revision: snapshot.revision,
            resolved_at: snapshot.resolved_at,
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
