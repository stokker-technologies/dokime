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
use std::collections::BTreeSet;

// Third Party Imports
use prost::Message;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::storage::SourcesSetRequest;
use dokime_protocol::host::storage::SourcesSetResponse;
use dokime_protocol::host::storage::StorageSources;
use dokime_storage::persisted::Scope;

/* === Constants === */

pub const ID: &str = "dokime.host.v1.storage.Sources/Set";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: SourcesSetRequest) -> Result<SourcesSetResponse> {
    crate::host::complete(ID, context.settings.mutation_timeout, async {
        let value = request.value.as_ref().ok_or_else(|| Error::Invalid("Set Storage Sources: Supply the Whole Writable Value".into()))?;
        let mut identifiers = BTreeSet::new();
        if value.sources.iter().any(|source| source.identifier.is_empty() || !identifiers.insert(&source.identifier)) {
            return Err(Error::Invalid("Set Storage Sources: Supply Unique Identifiers".into()));
        }
        let (snapshot, receipt) = crate::host::storage::administration::replace(
            context,
            "storage-sources",
            Scope::Root(&context.host.identity().backend_id).identifier()?,
            request.expected_revision,
            &request.request_id,
            value.encode_to_vec(),
        )
        .await?;
        context.host.signal_change();
        Ok(SourcesSetResponse {
            value: Some(StorageSources::decode(snapshot.payload.as_slice())?),
            revision: snapshot.revision,
            receipt: Some(receipt),
        })
    })
    .await
}

/* === Private Functions === */

/* === Tests === */

crate::native::unary!(SourcesSetRequest, SourcesSetResponse, context, "dokime.host.v1.storage", "Sources", "Set");
