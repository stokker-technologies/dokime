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
use harmos::{
    operations::Context,
    storage::{
        StorageCursor,
        operations::Query,
        query::{RowSlice, Source, StreamSelection, transform::Transform},
    },
};

// Local Imports
use super::invalid;
use crate::runtime::Result;
use crate::storage::telemetry::TypedCodec;
use crate::storage::telemetry::decode_positioned;
use crate::telemetry::TelemetryContract;
use crate::telemetry::TelemetrySample;
use dokime_domain::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn plan(at: &StorageCursor, scope: &str, codec: &TypedCodec, transforms: Vec<Transform>) -> Result<Query> {
    Ok(Query {
        at: at.clone(),
        source: Source::Stream(StreamSelection {
            schema: codec.schema().clone(),
            scope: Some(harmos::Identifier::new(scope).map_err(|_| invalid("invalid execution storage scope"))?),
            item: None,
            stream: None,
            source_start: None,
            source_end: None,
            after: 0,
        }),
        transforms,
        slice: RowSlice { offset: 0, limit: None },
    })
}

pub(in crate::report) async fn read(
    ctx: &Context<'_>, session: &crate::storage::Manager, at: &StorageCursor, scope: &str, contract: &TelemetryContract,
    transforms: Vec<Transform>,
) -> Result<harmos::storage::ReadBatch<Vec<TelemetrySample>>> {
    let codec = saved_codec(ctx, session, at, contract).await?;
    let query = plan(at, scope, &codec, transforms)?;
    let mut samples = Some(empty()?);
    let mut retained_bytes = 0usize;
    super::visit_batches(session, at, query, |batch| {
        let entries = decode_positioned(&codec, batch)?;
        let page = crate::storage::projection::page(entries, |(_, entry)| Ok(entry.sample))?;
        retained_bytes = retained_bytes
            .checked_add(crate::execution::evidence_rows::encoded_bytes(&*page))
            .filter(|bytes| *bytes <= 128 * 1024 * 1024)
            .ok_or_else(|| invalid("raw report plot workspace exceeds 128 MiB"))?;
        samples = Some(append(samples.take().expect("report sample owner exists"), page)?);
        Ok(())
    })
    .await?;
    sort(samples.expect("completed report sample owner exists"))
}

pub(in crate::report) async fn spool(
    ctx: &Context<'_>, session: &crate::storage::Manager, at: &StorageCursor, scope: &str, contract: &TelemetryContract,
    samples: &mut crate::execution::evidence_rows::EvidenceRows<TelemetrySample>,
) -> Result<()> {
    let codec = saved_codec(ctx, session, at, contract).await?;
    let query = plan(at, scope, &codec, Vec::new())?;
    super::visit_batches(session, at, query, |batch| {
        let entries = decode_positioned(&codec, batch)?;
        for (_, entry) in entries.iter() {
            samples.append(&entry.sample).map_err(crate::runtime::Error::Operation)?;
        }
        Ok(())
    })
    .await
}

pub(super) async fn streams(
    ctx: &Context<'_>, session: &crate::storage::Manager, at: &StorageCursor, scope: &str,
    contracts: &std::collections::BTreeMap<Identifier, std::collections::BTreeMap<u32, TelemetryContract>>, selected: &[String],
) -> Result<crate::plot::spilled::Input> {
    let input = crate::plot::spilled::Input::new()?;
    for (stream, versions) in contracts {
        if !selected.iter().any(|name| crate::plot::stream_matches_alias(stream.as_str(), name)) {
            continue;
        }
        for contract in versions.values() {
            let codec = saved_codec(ctx, session, at, contract).await?;
            let query = plan(at, scope, &codec, Vec::new())?;
            super::visit_batches(session, at, query, |batch| {
                let entries = decode_positioned(&codec, batch)?;
                for (_, entry) in entries.iter() {
                    input.append(&entry.sample)?;
                }
                Ok(())
            })
            .await?;
        }
    }
    Ok(input)
}

/* === Private Functions === */

pub(in crate::report) async fn saved_codec(
    ctx: &Context<'_>, session: &crate::storage::Manager, at: &StorageCursor, contract: &TelemetryContract,
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
    let codecs = crate::runtime::storage::telemetry::saved_codecs(
        session,
        at,
        Some(&schema),
        &ctx.scope().resources::<crate::app::Dokime>().schema_settings,
        |id, version| id.strip_prefix("dokime.telemetry.") == Some(contract.stream.as_str()) && version == contract.schema_version,
    )
    .await?;
    codecs.try_map(|mut codecs| {
        if codecs.len() != 1 {
            return Err(invalid("captured report schema is missing or ambiguous"));
        }
        Ok(codecs.pop().expect("one report codec"))
    })
}
pub(in crate::report) fn empty() -> Result<harmos::storage::ReadBatch<Vec<TelemetrySample>>> {
    Ok(harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::storage::Error>(Vec::new()))?)
}

pub(in crate::report) fn append(
    samples: harmos::storage::ReadBatch<Vec<TelemetrySample>>, page: harmos::storage::ReadBatch<Vec<TelemetrySample>>,
) -> Result<harmos::storage::ReadBatch<Vec<TelemetrySample>>> {
    Ok(crate::runtime::storage::telemetry::append_retained(samples, page)?)
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
