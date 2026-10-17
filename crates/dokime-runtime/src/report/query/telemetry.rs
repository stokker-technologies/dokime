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
        query::{Query, RowSlice, Source, StreamSelection, transform::Transform},
    },
};
// Local Imports
use super::invalid;
use crate::runtime::Result;
use crate::{
    core::Identifier,
    storage::telemetry::{TypedCodec, codec, encode_batch},
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
            after: 0,
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
    settings: &crate::config::StorageSettings,
    at: &StorageCursor,
    scope: &str,
    contract: &TelemetryContract,
    transforms: Vec<Transform>,
) -> Result<Vec<TelemetrySample>> {
    let codec = codec(contract)?;
    let query = plan(at, scope, &codec, transforms)?;
    let mut samples = Vec::new();
    for batch in super::batches(ctx, settings, query).await? {
        for row in encode_batch(&codec, &batch)? {
            samples.push(codec.decode(&row, usize::MAX)?.sample);
        }
    }
    sort(&mut samples);
    Ok(samples)
}

pub(super) async fn streams(
    ctx: &Context<'_>,
    settings: &crate::config::StorageSettings,
    at: &StorageCursor,
    scope: &str,
    contracts: &std::collections::BTreeMap<
        Identifier,
        std::collections::BTreeMap<u32, TelemetryContract>,
    >,
    selected: &[String],
) -> Result<Vec<TelemetrySample>> {
    let mut samples = Vec::new();
    for (stream, versions) in contracts {
        if !selected
            .iter()
            .any(|name| crate::plot::stream_matches_alias(stream.as_str(), name))
        {
            continue;
        }
        for contract in versions.values() {
            samples.extend(read(ctx, settings, at, scope, contract, Vec::new()).await?);
        }
    }
    sort(&mut samples);
    Ok(samples)
}

/* === Private Functions === */
fn sort(samples: &mut [TelemetrySample]) {
    samples.sort_by(|left, right| {
        left.timestamp
            .cmp(&right.timestamp)
            .then(left.sequence.cmp(&right.sequence))
    });
}

/* === Tests === */
