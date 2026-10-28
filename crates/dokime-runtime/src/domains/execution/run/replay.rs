/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Replays Saved Execution Samples With Retained Decode And Signal Staging.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroU32;

// Third Party Imports
use harmos::storage::Query;
use harmos::storage::query::{QueryArrow, RowSlice, Source, StreamSelection};

// Local Imports
use super::{ExecutionFixtureContext, ExecutionSignal, ProviderExecutionFailure, Target, TestExecution, failure};

/* === Constants === */

const REPLAY_SIGNAL_BYTES: usize = crate::transport::MAX_ARTIFACT_CHUNK_BYTES;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct JsonCount(usize);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::io::Write for JsonCount {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self.0.checked_add(bytes.len()).ok_or_else(|| std::io::Error::other("replay JSON length overflows"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/* === Public Functions === */

pub(super) async fn replay_sample_inputs(
    context: &ExecutionFixtureContext, target: &Target, execution: &TestExecution,
) -> Result<(), ProviderExecutionFailure> {
    // Reconcile The Durable Checkpoint With The Receiver Before Reading History.
    // A Replacement Receiver Has No Receipts And Must Rebuild Its Evidence.
    let storage = context.runtime.storage();
    let cut = storage.flush().await.map_err(|error| failure("sidecar_samples", error.to_string()))?;
    let mut at = storage.cursor_saved();
    at.sequence = cut.sequence;
    let acknowledged = target
        .signal(ExecutionSignal {
            execution_token: execution.test_execution_id.to_string(),
            ..Default::default()
        })
        .await?;
    let contracts = context.runtime.state_read().telemetry_contracts();
    for contract in contracts.iter().filter(|contract| !contract.foreign) {
        let mut position = match acknowledged.samples.iter().find(|receipt| receipt.stream == contract.stream.as_str()) {
            Some(receipt) => context
                .runtime
                .local_storage
                .acknowledge_sample(execution.test_execution_id.to_string(), receipt.stream.clone(), receipt.sample_id.clone(), at.sequence)
                .await
                .map_err(failed)?,
            None => 0,
        };
        replay_stream(context, target, execution, &at, contract, &mut position).await?;
    }
    Ok(())
}

/* === Private Functions === */

async fn replay_stream(
    context: &ExecutionFixtureContext, target: &Target, execution: &TestExecution, at: &harmos::storage::StorageCursor,
    contract: &crate::telemetry::TelemetryContract, replayed: &mut u64,
) -> Result<(), ProviderExecutionFailure> {
    let session = context.runtime.storage().clone();
    let key_bytes = "dokime.telemetry.".len().checked_add(contract.stream.as_str().len()).ok_or_else(overflow)?;
    let mut schema_id = String::new();
    schema_id.try_reserve_exact(key_bytes).map_err(failed)?;
    schema_id.push_str("dokime.telemetry.");
    schema_id.push_str(contract.stream.as_str());
    let schema = harmos::storage::SchemaKey {
        id: harmos::Identifier::new(&schema_id).map_err(failed)?,
        version: NonZeroU32::new(contract.schema_version).ok_or_else(|| failed("telemetry schema version is zero"))?,
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
    };
    let codecs =
        crate::runtime::storage::telemetry::saved_codecs(&session, at, Some(&schema), context.runtime.schema_settings(), |_, _| true)
            .await
            .map_err(failed)?;
    if codecs.is_empty() {
        return Ok(());
    }
    if codecs.len() != 1 {
        return Err(failed("saved replay schema is absent or ambiguous"));
    }
    let codec = &codecs[0];
    let mut reader = session
        .query(Query {
            at: at.clone(),
            source: Source::Stream(StreamSelection {
                schema: codec.schema().clone(),
                scope: Some(harmos::Identifier::new(execution.test_execution_id.as_str()).map_err(failed)?),
                item: None,
                stream: Some(contract.stream.clone()),
                source_start: None,
                source_end: None,
                after: *replayed,
            }),
            transforms: Vec::new(),
            slice: RowSlice::default(),
        })
        .await
        .map_err(failed)?;
    let mut announced = None;
    let mut consumed = 0_u64;
    while let Some(output) = reader.next().await.map_err(failed)? {
        let batch = match output {
            QueryArrow::Schema(schema) => {
                if announced.is_some() || consumed != 0 {
                    return Err(failed("replay query emitted a repeated or late schema"));
                }
                announced = Some(schema);
                continue;
            }
            QueryArrow::Batch(batch) => batch,
        };
        let schema = announced.as_ref().ok_or_else(|| failed("replay query emitted data before its schema"))?;
        batch.validate_schema(schema.as_schema()).map_err(failed)?;
        consumed = consumed.checked_add(u64::try_from(batch.as_record_batch().num_rows()).map_err(failed)?).ok_or_else(overflow)?;
        // The Receiver Orders Each Stream By The Ordinal Its Capture Scope Saved, Never By A Producer's Own Count.
        let decoded = crate::storage::telemetry::decode_positioned(codec, batch).map_err(failed)?;
        let entries = decoded
            .try_map(|mut entries| {
                for (_, entry) in &mut entries {
                    entry.sample.sequence = entry.ordinal;
                }
                Ok::<_, crate::storage::Error>(entries)
            })
            .map_err(failed)?;
        let mut offset = 0;
        while offset < entries.len() {
            let mut end = offset;
            let mut bytes = 0_usize;
            while let Some((_, entry)) = entries.get(end) {
                let captured = entry.sample.scope.as_ref().ok_or_else(|| failed("captured sample has no execution scope"))?;
                let owner = &execution.scope;
                if captured.session_id != owner.session_id
                    || captured.test_cycle_id != owner.test_cycle_id
                    || captured.test_case_id != owner.test_case_id
                    || captured.test_execution_id != owner.test_execution_id
                    || captured.template_id != owner.template_id
                    || captured.suite_id != owner.suite_id
                {
                    return Err(failed("captured sample differs from its execution owner"));
                }
                let next = bytes.checked_add(json_len(&entry.sample)?).ok_or_else(overflow)?;
                if end != offset && next > REPLAY_SIGNAL_BYTES {
                    break;
                }
                bytes = next;
                end += 1;
            }
            let mut samples = Vec::new();
            samples.try_reserve_exact(end - offset).map_err(failed)?;
            for (_, entry) in &entries[offset..end] {
                samples.push(json(&entry.sample)?);
            }
            let scope_json = json(&execution.scope)?;
            let acknowledged = target
                .signal(ExecutionSignal {
                    execution_token: execution.test_execution_id.to_string(),
                    samples: Some(crate::transport::DeliverSamplesRequest { scope_json, samples, gaps: Vec::new() }),
                    ..ExecutionSignal::default()
                })
                .await?;
            // The Receipt Advances Only After The Child Acknowledges This Chunk.
            let last = &entries[end - 1].1.sample;
            let receipt = acknowledged.samples.iter().find(|receipt| receipt.stream == contract.stream.as_str());
            if !receipt.is_some_and(|receipt| receipt.sample_id == last.sample_id.as_str()) {
                return Err(failed("sample signal did not acknowledge its final row"));
            }
            *replayed = context
                .runtime
                .local_storage
                .acknowledge_sample(
                    execution.test_execution_id.to_string(),
                    contract.stream.to_string(),
                    last.sample_id.to_string(),
                    at.sequence,
                )
                .await
                .map_err(failed)?;
            offset = end;
        }
    }
    let complete = reader.completion().map_err(failed)?;
    if announced.is_none() || complete.at != *at || complete.rows != consumed {
        return Err(failed("replay query completion differs from the consumed evidence"));
    }
    Ok(())
}

fn json_len(value: &impl serde::Serialize) -> Result<usize, ProviderExecutionFailure> {
    let mut count = JsonCount::default();
    serde_json::to_writer(&mut count, value).map_err(failed)?;
    Ok(count.0)
}

fn json(value: &impl serde::Serialize) -> Result<String, ProviderExecutionFailure> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(json_len(value)?).map_err(failed)?;
    serde_json::to_writer(&mut bytes, value).map_err(failed)?;
    String::from_utf8(bytes).map_err(failed)
}

fn failed(error: impl std::fmt::Display) -> ProviderExecutionFailure {
    failure("sidecar_samples", error.to_string())
}

fn overflow() -> ProviderExecutionFailure {
    failed("replay staging memory overflows")
}

/* === Tests === */
