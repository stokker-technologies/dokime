/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Storage Sources Set

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_protocol::v1 as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;

/* === Constants === */

pub const ID: &str = "dokime.v1.storage.Sources/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<h::OperationContract> {
    super::declaration::<v1::storage::SourcesSetRequest, v1::storage::SourcesSetResponse>(ID, settings.mutation_timeout)
}
pub async fn handle(context: &Context, request: v1::storage::SourcesSetRequest) -> Result<v1::storage::SourcesSetResponse> {
    super::complete(ID, context.settings.mutation_timeout, async {
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Storage Sources: Supply the Whole Writable Value".into()))?;
        let mut identifiers = std::collections::BTreeSet::new();
        if value.sources.iter().any(|source| source.identifier.is_empty() || !identifiers.insert(&source.identifier)) {
            return Err(Error::Invalid("Set Storage Sources: Supply Unique Identifiers".into()));
        }
        let fingerprint = super::views::hex(&super::paging::bytes(&request));
        let (payload, revision, receipt) = context.host.store().replace_administration_value(
            "storage-sources",
            &request.expected_revision,
            &request.request_id,
            fingerprint.as_bytes(),
            &value.encode_to_vec(),
        )?;
        context.host.signal_change();
        Ok(v1::storage::SourcesSetResponse {
            value: Some(v1::StorageSources::decode(payload.as_slice())?),
            revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
