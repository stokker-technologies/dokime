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
    MemorySize,
    operations::Context,
    storage::{
        QueryValue, StorageCursor,
        query::{Query, RowSlice, Source, StreamSelection, transform::Transform},
    },
};
// Local Imports
use super::{exhausted, invalid, nonzero};
use crate::Result;
use dokime::{
    core::Identifier,
    storage::{
        Manager,
        telemetry::{
            TypedCodec, codec, decoding_memory_rows as decoding_memory, encode_batch as encoded,
        },
    },
    telemetry::{TelemetryContract, TelemetrySample},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn plan(
    at: &StorageCursor,
    scope: &str,
    codec: &TypedCodec,
    transforms: Vec<Transform>,
) -> Result<Query> {
    Ok(Query {
        at: at.clone(),
        source: Source::Stream(StreamSelection {
            schema: codec.schema().clone(),
            scope: Some(
                harmos::Identifier::new(scope)
                    .map_err(|_| invalid("invalid execution storage scope"))?,
            ),
            item: None,
            stream: None,
            source_start: None,
            source_end: None,
        }),
        transforms,
        slice: RowSlice {
            offset: 0,
            limit: None,
        },
    })
}

pub(in crate::report) async fn read(
    ctx: &Context<'_>,
    storage: &Manager,
    settings: &crate::config::StorageSettings,
    at: &StorageCursor,
    scope: &str,
    contract: &TelemetryContract,
    transforms: Vec<Transform>,
) -> Result<QueryValue<Vec<TelemetrySample>>> {
    let codec = codec(contract)?;
    let query = plan(at, scope, &codec, transforms)?;
    let batches = super::batches(ctx, storage, settings, query).await?;
    let mut samples = super::empty(storage)?;
    for batch in batches.iter() {
        let rows = batch.as_record_batch().num_rows();
        let encoding_bytes = settings
            .response
            .batch_bytes
            .get()
            .checked_add(
                rows.checked_mul(std::mem::size_of::<Vec<u8>>())
                    .ok_or_else(exhausted)?,
            )
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<Vec<u8>>>()))
            .ok_or_else(exhausted)?;
        let _encoding = storage.prepare_production(nonzero(encoding_bytes)?)?;
        let encoded = encoded(&codec, batch, nonzero(encoding_bytes)?)?;
        let bound = decoding_memory(&codec, &encoded)?;
        let decoded = storage
            .produce(nonzero(bound)?, |allowance| {
                encoded
                    .iter()
                    .map(|row| {
                        codec
                            .decode(row, allowance.get())
                            .map(|entry| entry.sample)
                            .map_err(codec_error)
                    })
                    .collect::<harmos::Result<Vec<_>>>()
            })?
            .try_into_value()?;
        let bound = samples
            .memory_size()
            .checked_add(decoded.memory_size())
            .ok_or_else(exhausted)?;
        samples = samples.try_map(nonzero(bound)?, |mut samples, _| {
            samples
                .try_reserve_exact(decoded.len())
                .map_err(|_| exhausted())?;
            samples.extend(decoded.iter().cloned());
            Ok(samples)
        })?;
    }
    let bound = samples.memory_size();
    samples = samples.try_map(nonzero(bound)?, |mut samples, _| {
        samples.sort_by(|left, right| {
            left.timestamp
                .cmp(&right.timestamp)
                .then(left.sequence.cmp(&right.sequence))
        });
        Ok(samples)
    })?;
    Ok(samples)
}

pub(super) async fn streams(
    ctx: &Context<'_>,
    storage: &Manager,
    settings: &crate::config::StorageSettings,
    at: &StorageCursor,
    scope: &str,
    contracts: &std::collections::BTreeMap<
        Identifier,
        std::collections::BTreeMap<u32, TelemetryContract>,
    >,
    selected: &[String],
) -> Result<QueryValue<Vec<TelemetrySample>>> {
    let mut samples = super::empty(storage)?;
    for (stream, versions) in contracts {
        if !selected
            .iter()
            .any(|name| crate::plot::stream_matches_alias(stream.as_str(), name))
        {
            continue;
        }
        for contract in versions.values() {
            let values = read(ctx, storage, settings, at, scope, contract, Vec::new()).await?;
            let bound = samples
                .memory_size()
                .checked_add(values.memory_size())
                .ok_or_else(exhausted)?;
            samples = samples.try_map(nonzero(bound)?, |mut samples, _| {
                samples
                    .try_reserve_exact(values.len())
                    .map_err(|_| exhausted())?;
                samples.extend(values.iter().cloned());
                Ok(samples)
            })?;
        }
    }
    let bound = samples.memory_size();
    samples = samples.try_map(nonzero(bound)?, |mut samples, _| {
        samples.sort_by(|left, right| {
            left.timestamp
                .cmp(&right.timestamp)
                .then(left.sequence.cmp(&right.sequence))
        });
        Ok(samples)
    })?;
    Ok(samples)
}

/* === Private Functions === */
fn codec_error(source: dokime::storage::Error) -> harmos::Error {
    match source {
        dokime::storage::Error::Runtime(source) => source,
        source => harmos::Error::backend(
            "report telemetry projection",
            harmos::BackendFailure::Corrupt,
            source,
        ),
    }
}

/* === Tests === */
