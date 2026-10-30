/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::storage::StorageCursor;
// Local Imports
use super::invalid;
use crate::runtime::Result;
use crate::storage::telemetry::TypedCodec;
use crate::telemetry::TelemetryContract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(in crate::report) async fn saved_codec(
    ctx: &crate::report::Scope, session: &crate::storage::Manager, at: &StorageCursor, contract: &TelemetryContract,
) -> Result<harmos::storage::ReadBatch<TypedCodec>> {
    let identifier_bytes = "dokime.telemetry."
        .len()
        .checked_add(contract.stream.as_str().len())
        .ok_or_else(|| invalid("report schema identifier size overflows"))?;
    let mut identifier = String::new();
    identifier.try_reserve_exact(identifier_bytes).map_err(|_| invalid("report schema identifier allocation failed"))?;
    identifier.push_str("dokime.telemetry.");
    identifier.push_str(contract.stream.as_str());
    let schema = harmos::storage::SchemaKey {
        id: harmos::Identifier::new(identifier).map_err(|_| invalid("invalid report schema identifier"))?,
        version: std::num::NonZeroU32::new(contract.schema_version).ok_or_else(|| invalid("invalid report schema version"))?,
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
    };
    let codecs =
        crate::runtime::storage::telemetry::saved_codecs(session, at, Some(&schema), &ctx.resources().schema_settings, |id, version| {
            id.strip_prefix("dokime.telemetry.") == Some(contract.stream.as_str()) && version == contract.schema_version
        })
        .await?;
    codecs.try_map(|mut codecs| {
        if codecs.len() != 1 {
            return Err(invalid("captured report schema is missing or ambiguous"));
        }
        Ok(codecs.pop().expect("one report codec"))
    })
}
/* === Tests === */
