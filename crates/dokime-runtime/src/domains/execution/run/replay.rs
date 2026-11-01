/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Replays Typed Arrow Samples With Retained Signal Staging.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroU32;

// Third Party Imports
use harmos::storage::query::StreamSelection;

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
    let codecs = crate::runtime::storage::telemetry::saved_codecs(storage, &at, None, context.runtime.schema_settings(), |id, version| {
        contracts.iter().any(|contract| {
            !contract.foreign
                && contract.schema_version == version
                && id.strip_prefix("dokime.telemetry.") == Some(contract.stream.as_str())
        })
    })
    .await
    .map_err(failed)?;
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
        replay_stream(context, target, execution, &at, contract, &mut position, &codecs).await?;
    }
    Ok(())
}

/* === Private Functions === */

async fn replay_stream(
    context: &ExecutionFixtureContext, target: &Target, execution: &TestExecution, at: &harmos::storage::StorageCursor,
    contract: &crate::telemetry::TelemetryContract, replayed: &mut u64, codecs: &[crate::storage::telemetry::TypedCodec],
) -> Result<(), ProviderExecutionFailure> {
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
    let mut selected = codecs.iter().filter(|codec| codec.schema() == &schema);
    let Some(codec) = selected.next() else {
        return Ok(());
    };
    if selected.next().is_some() {
        return Err(failed("saved replay schema is absent or ambiguous"));
    }
    let mut source = StreamSelection {
        schema: codec.schema().clone(),
        scope: Some(harmos::Identifier::new(execution.test_execution_id.as_str()).map_err(failed)?),
        item: None,
        stream: Some(contract.stream.clone()),
        source_start: None,
        source_end: None,
        after: *replayed,
    };
    let mut scanned = *replayed;
    loop {
        let storage = context.runtime.storage();
        let page = harmos::storage::ScanPage {
            after: source.after,
            rows: storage.settings().response.batch_rows,
            bytes: storage.settings().response.batch_bytes,
        };
        let mut entries = context.runtime.local_storage.telemetry_read(at.clone(), source.clone(), page).await.map_err(failed)?;
        if entries.is_empty() {
            break;
        }
        for (position, entry) in &mut entries {
            if *position <= scanned || *position > at.sequence {
                return Err(failed("replay page differs from the saved cursor"));
            }
            scanned = *position;
            source.after = scanned;
            entry.sample.sequence = entry.ordinal;
            if let Some(scope) =
                entry.sample.captured_scopes.iter().find(|scope| scope.test_execution_id.as_ref() == Some(&execution.test_execution_id))
            {
                entry.sample.scope = Some(scope.clone());
            }
        }
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
                tracing::warn!(stream = %contract.stream, expected = %last.sample_id,
                    received = ?receipt.map(|receipt| receipt.sample_id.as_str()), "replayed sample receipt differs");
                return Err(failed("sample signal did not acknowledge its final row"));
            }
            *replayed = entries[end - 1].0;
            offset = end;
        }
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
