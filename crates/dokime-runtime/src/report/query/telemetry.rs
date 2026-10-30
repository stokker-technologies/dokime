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
use harmos::storage::{StorageCursor, query::StreamSelection};

// Local Imports
use super::invalid;
use crate::runtime::Result;
use crate::storage::telemetry::TypedCodec;
use crate::telemetry::TelemetryContract;
use crate::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::report) async fn visit(
    ctx: &crate::report::Scope, session: &crate::storage::Manager, at: &StorageCursor, scope: &str, contract: &TelemetryContract,
    mut visit: impl FnMut(&TelemetrySample) -> Result<()>,
) -> Result<()> {
    let codec = saved_codec(ctx, session, at, contract).await?;
    let mut source = StreamSelection {
        schema: codec.schema().clone(),
        scope: Some(harmos::Identifier::new(scope).map_err(|_| invalid("invalid telemetry scope"))?),
        item: None,
        stream: Some(contract.stream.clone()),
        source_start: None,
        source_end: None,
        after: 0,
    };
    loop {
        let page = harmos::storage::ScanPage {
            after: source.after,
            rows: session.settings().response.batch_rows,
            bytes: session.settings().response.batch_bytes,
        };
        let rows = ctx.local.telemetry_read(at.clone(), source.clone(), page).await?;
        if rows.is_empty() {
            break;
        }
        for (position, entry) in rows {
            source.after = position;
            visit(&entry.sample)?;
        }
    }
    Ok(())
}

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
pub(in crate::report) fn sort(
    samples: harmos::storage::ReadBatch<Vec<TelemetrySample>>,
) -> Result<harmos::storage::ReadBatch<Vec<TelemetrySample>>> {
    samples.try_map(|mut samples| {
        let mut order = Vec::new();
        order.try_reserve_exact(samples.len()).map_err(|_| invalid("report ordering allocation failed"))?;
        order.extend(0..samples.len());
        order.sort_unstable_by(|left, right| {
            samples[*left]
                .timestamp
                .cmp(&samples[*right].timestamp)
                .then(samples[*left].sequence.cmp(&samples[*right].sequence))
                .then(left.cmp(right))
        });
        let mut destinations = Vec::new();
        destinations.try_reserve_exact(samples.len()).map_err(|_| invalid("report ordering allocation failed"))?;
        destinations.resize(samples.len(), 0);
        for (destination, source) in order.into_iter().enumerate() {
            destinations[source] = destination;
        }
        for index in 0..samples.len() {
            while destinations[index] != index {
                let destination = destinations[index];
                samples.swap(index, destination);
                destinations.swap(index, destination);
            }
        }
        Ok(samples)
    })
}
/* === Tests === */
