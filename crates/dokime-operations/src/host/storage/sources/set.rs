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
use harmos_protocol::model as h;
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.storage.Sources/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &crate::host::ClientSettings) -> Result<h::OperationContract> {
    crate::host::declaration::<dokime_protocol::host::storage::SourcesSetRequest, dokime_protocol::host::storage::SourcesSetResponse>(
        ID,
        settings.mutation_timeout,
    )
}
pub async fn handle(
    context: &Context, request: dokime_protocol::host::storage::SourcesSetRequest,
) -> Result<dokime_protocol::host::storage::SourcesSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Storage Sources: Supply the Whole Writable Value".into()))?;
        let mut identifiers = std::collections::BTreeSet::new();
        if value.sources.iter().any(|source| source.identifier.is_empty() || !identifiers.insert(&source.identifier)) {
            return Err(Error::Invalid("Set Storage Sources: Supply Unique Identifiers".into()));
        }
        let fingerprint = crate::host::views::hex(&crate::host::paging::bytes(&request));
        let (payload, revision, receipt) = context.host.store().replace_administration_value(
            "storage-sources",
            &request.expected_revision,
            &request.request_id,
            fingerprint.as_bytes(),
            &value.encode_to_vec(),
        )?;
        context.host.signal_change();
        Ok(dokime_protocol::host::storage::SourcesSetResponse {
            value: Some(dokime_protocol::host::storage::StorageSources::decode(payload.as_slice())?),
            revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */
