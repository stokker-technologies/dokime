/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "history.rs"]
mod history;

/* === Re-Exports === */
pub use history::CapturedRecord;

/* === Imports === */

// Standard Library Imports
use std::{
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

// Third Party Imports
use futures_util::StreamExt;
use harmos::{
    Identifier, MemorySize, Message,
    storage::{QueryOptions, ReadLimits, RecordWrite, SchemaKey, SchemaQuery},
};
use tokio::sync::watch;
use tracing::Instrument;

// Local Imports
use super::error;
use crate::{Error, Result, records::RecordEntry};
use dokime::storage::records;

/* === Constants === */
const FRAME_BYTES: usize = 1024 * 1024;
const MAX_GROUP_ROWS: usize = 256;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Source {
    storage: dokime::storage::Manager,
    reads: crate::config::StorageReadSettings,
    schema: SchemaKey,
    epoch: Identifier,
    closed: watch::Sender<bool>,
    origin: AtomicU64,
}

pub(crate) struct Projection(pub(crate) Arc<Source>);

struct ProjectedRecord {
    item: Identifier,
    payload: Vec<u8>,
}

#[derive(serde::Deserialize)]
struct PersistedRecord {
    dokime_encoding: u32,
    payload: crate::records::Record,
}

struct NumberBytes(usize);

#[derive(Default)]
pub(crate) struct Process {
    history: Option<history::History>,
    through: u64,
    before: u64,
    position: u64,
    physical: u64,
    last: Option<history::CapturedRecord>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl std::fmt::Write for NumberBytes {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.0 = self.0.checked_add(value.len()).ok_or(std::fmt::Error)?;
        Ok(())
    }
}

impl MemorySize for ProjectedRecord {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(
                self.item
                    .memory_size()
                    .saturating_sub(std::mem::size_of::<Identifier>()),
            )
            .saturating_add(self.payload.capacity())
    }
}

impl Source {
    pub(crate) fn new(
        epoch: &dokime::core::SessionId,
        storage: dokime::storage::Manager,
        reads: crate::config::StorageReadSettings,
    ) -> Result<Self> {
        reads.validate()?;
        Ok(Self {
            storage,
            reads,
            schema: records::schema(),
            epoch: Identifier::new(epoch.to_string()).map_err(error)?,
            closed: watch::channel(false).0,
            origin: AtomicU64::new(0),
        })
    }

    pub(crate) fn page_limits(&self) -> ReadLimits {
        ReadLimits {
            rows: self.reads.page_rows,
            bytes: self.reads.page_bytes,
        }
    }

    pub(crate) async fn initialize(&self) -> Result<()> {
        let mut query = SchemaQuery::new();
        query.schema = Some(self.schema.clone());
        query.options = QueryOptions {
            page: Some(self.page_limits()),
            execution: Some(self.reads.execution),
            decoded_bytes: Some(self.reads.decoded_bytes),
            ..QueryOptions::default()
        };
        let mut schemas = self.storage.query(query).await.map_err(Error::from)?;
        let marker_memory = NonZeroUsize::new(std::mem::size_of::<bool>()).unwrap();
        let mut found = false;
        while let Some(validated) = schemas
            .next_page_projected(marker_memory, |rows, _| {
                if found || rows.len() != 1 || rows[0].fields != records::fields() {
                    return Err(projection_error("stored capture schema differs"));
                }
                Ok(true)
            })
            .await
            .map_err(Error::from)?
        {
            found = *validated;
        }
        drop(schemas);
        if !found {
            let declared = self
                .storage
                .write_schema(&self.schema, records::fields())
                .await
                .map_err(Error::from)?;
            self.storage
                .flush_until(&declared)
                .await
                .map_err(Error::from)?;
        }
        Ok(())
    }

    fn close(&self) {
        self.closed.send_replace(true);
    }

    pub(crate) fn begin(&self, origin: harmos::Position) {
        self.origin.store(origin.0, Ordering::Release);
    }

    pub(crate) fn epoch(&self) -> &str {
        self.epoch.as_str()
    }

    pub(crate) fn origin(&self) -> u64 {
        self.origin.load(Ordering::Acquire)
    }

    fn healthy(&self, state_manager: &crate::app::StateManager) -> Result<()> {
        if let Some(failure) = self.storage.status().failure {
            return Err(error(failure));
        }
        state_manager.capture_status().map_err(error)
    }

    pub(crate) async fn wait_through(
        &self,
        state_manager: &crate::app::StateManager,
        frontier: u64,
    ) -> Result<()> {
        self.healthy(state_manager)?;
        state_manager
            .wait_until_captured(harmos::Position(frontier))
            .await
            .map_err(error)?;
        self.healthy(state_manager)
    }

    pub(crate) async fn wait_after(
        &self,
        state_manager: &crate::app::StateManager,
        after: u64,
    ) -> Result<Option<u64>> {
        let mut progress = self.closed.subscribe();
        loop {
            self.healthy(state_manager)?;
            let frontier = state_manager.applied().0;
            if frontier > after {
                self.wait_through(state_manager, frontier).await?;
                return Ok(Some(frontier));
            }
            if *progress.borrow_and_update() {
                return Ok(None);
            }
            let watched = state_manager
                .watch::<harmos::Recorded<crate::app::Dokime>>(harmos::Position(after));
            tokio::pin!(watched);
            tokio::select! {
                _ = progress.changed() => {},
                entry = watched.next() => {
                    // A racing burst may evict the watched tail. Only the applied
                    // frontier is needed here; primary rows own the actual replay.
                    if state_manager.applied().0 <= after && !*progress.borrow() {
                        match entry {
                            Some(Err(failure)) => return Err(error(failure)),
                            None => return Ok(None),
                            Some(Ok(_)) => {},
                        }
                    }
                }
            }
        }
    }
}

impl harmos::storage::RecordProjection for Projection {
    fn project<'a>(
        &'a self,
        envelope: &'a harmos::Envelope<&serde_json::Value>,
        output: &'a mut harmos::storage::ProjectedRecords<'_>,
    ) -> harmos::storage::StorageFuture<'a, ()> {
        let span = tracing::debug_span!(
            target: "dokime_app::record_admission",
            "record_projection"
        );
        Box::pin(
            async move {
                let bytes = projection_memory(envelope.payload, envelope.id.as_str().len())?;
                let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
                let preparation = loop {
                    match self.0.storage.prepare_production(bytes) {
                        Err(harmos::Error::ResourceExhausted {
                            resource: "storage read memory",
                        }) if tokio::time::Instant::now() < deadline => {
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                        result => break result?,
                    }
                };
                let produced = preparation.finish(
                    (|allowance: std::num::NonZeroUsize| -> harmos::Result<_> {
                        let record = if let Some(fact) = harmos::SystemFact::restore(
                            &envelope.id,
                            envelope.version,
                            envelope.payload,
                        )
                        .map_err(projection_error)?
                        {
                            let harmos::FactSource::Sidecar(owner) = fact.source else {
                                return Ok(None);
                            };
                            let persisted: PersistedRecord =
                                serde_json::from_slice(&fact.payload).map_err(projection_error)?;
                            if persisted.dokime_encoding != 2 {
                                return Err(projection_error(
                                    "unsupported Dokime persistence encoding",
                                ));
                            }
                            let record = persisted.payload;
                            if record.source != owner
                                || record.record_type.as_str() != fact.name
                                || record.schema_version != fact.version
                            {
                                return Err(projection_error(
                                    "native fact identity differs from its envelope",
                                ));
                            }
                            record
                        } else {
                            let Some(crate::records::Records::Record(record)) =
                                crate::records::Records::restore(
                                    &envelope.id,
                                    envelope.version,
                                    envelope.payload,
                                )
                                .map_err(projection_error)?
                            else {
                                return Err(projection_error("unknown product record"));
                            };
                            record
                        };
                        let record = RecordEntry::from_record(envelope.position.0, record);
                        let native = records::RecordRow::encoding_memory(&record)
                            .map_err(projection_error)?;
                        let construction = record
                            .memory_size()
                            .checked_add(native)
                            .and_then(|bytes| {
                                bytes.checked_add(std::mem::size_of::<ProjectedRecord>())
                            })
                            .and_then(|bytes| bytes.checked_add(record.record_id.as_str().len()))
                            .filter(|bytes| *bytes < usize::MAX && *bytes <= allowance.get())
                            .ok_or_else(|| {
                                projection_error("record construction exceeds original admission")
                            })?;
                        let row =
                            records::RecordRow::from_entry(&record).map_err(projection_error)?;
                        let length = row.encoded_len();
                        if length > FRAME_BYTES {
                            return Err(projection_error(
                                "record capture frame exceeds byte limit",
                            ));
                        }
                        if construction
                            .checked_add(length)
                            .is_none_or(|bytes| bytes > allowance.get())
                        {
                            return Err(projection_error(
                                "record encoding exceeds original admission",
                            ));
                        }
                        // Final Buffers Stay Within The Already-Admitted Construction Peak
                        let mut payload = Vec::new();
                        payload
                            .try_reserve_exact(length)
                            .map_err(projection_error)?;
                        if payload.capacity() > length {
                            return Err(projection_error(
                                "record encoding capacity exceeds original admission",
                            ));
                        }
                        row.encode_into(&mut payload);
                        if payload.len() != length || payload.capacity() > length {
                            return Err(projection_error(
                                "record encoding differs from its admitted length",
                            ));
                        }

                        let identity = record.record_id.as_str();
                        let mut item = String::new();
                        item.try_reserve_exact(identity.len())
                            .map_err(projection_error)?;
                        if item.capacity() > identity.len() {
                            return Err(projection_error(
                                "record identity capacity exceeds original admission",
                            ));
                        }
                        item.push_str(identity);
                        let item = Identifier::new(item).map_err(projection_error)?;
                        Ok(Some(ProjectedRecord { item, payload }))
                    })(bytes)?,
                )?;
                let Some(record) = produced.value() else {
                    return Ok(());
                };
                output
                    .push(RecordWrite {
                        schema: &self.0.schema,
                        scope: Some(&self.0.epoch),
                        item: &record.item,
                        source: None,
                        source_time: None,
                        payload: &record.payload,
                    })
                    .await
            }
            .instrument(span),
        )
    }
}

impl Drop for Projection {
    fn drop(&mut self) {
        self.0.close();
    }
}

impl Process {
    pub(crate) fn release_completed(&mut self, after: u64) {
        if after >= self.through {
            self.history = None;
            self.last = None;
        }
    }

    pub(crate) async fn next(
        &mut self,
        source: &Arc<Source>,
        state_manager: &crate::app::StateManager,
        after: u64,
        through: u64,
    ) -> Result<Option<history::CapturedRecord>> {
        if after > through {
            return Err(error("invalid record capture range"));
        }
        source.wait_through(state_manager, through).await?;
        if through <= source.origin() || after == through {
            self.release_completed(after);
            return Ok(None);
        }
        let after = after.max(source.origin());
        if self.through == through
            && let Some(last) = &self.last
            && after >= self.before
            && after < last.sequence
        {
            return Ok(Some(last.clone()));
        }
        let rewind = after < self.position;
        if self.history.is_none() || self.through != through || rewind {
            let start = if through >= self.through && after >= self.position && self.physical > 0 {
                history::Start::Physical(self.physical)
            } else if after > source.origin() {
                history::Start::Logical(after)
            } else {
                history::Start::Physical(0)
            };
            // Release the previous view before admitting another bounded reader.
            self.history = None;
            self.last = None;
            self.position = after;
            self.physical = 0;
            self.history = Some(
                source
                    .history_scoped(state_manager, start, through, Some(source.epoch.clone()))
                    .await?,
            );
            self.through = through;
        }
        loop {
            let next = self.history.as_mut().unwrap().next().await;
            let record = match next {
                Ok(Some(row)) => row,
                Ok(None) => break,
                Err(failure) => {
                    // Retry Keeps The Same Fixed View And Original Pending Page
                    return Err(failure);
                }
            };
            self.position = record.sequence;
            self.physical = record.position();
            if record.sequence > after {
                self.before = after;
                self.last = Some(record.clone());
                return Ok(Some(record));
            }
        }
        self.position = through;
        self.release_completed(through);
        Ok(None)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn projection_error(source: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> harmos::Error {
    harmos::Error::Backend {
        operation: "project Dokime record",
        kind: harmos::BackendFailure::Permanent,
        source: source.into(),
    }
}

fn projection_memory(
    input: &serde_json::Value,
    identity_bytes: usize,
) -> harmos::Result<NonZeroUsize> {
    // The Existing Conservative Bound Remains Until Typed Restore Admission
    // Accounts For Each Record Field And Its Deserializer Capacity Policy.
    let input = match input {
        serde_json::Value::Array(bytes)
            if bytes
                .iter()
                .all(|byte| byte.as_u64().is_some_and(|byte| byte <= u8::MAX.into())) =>
        {
            bytes
                .len()
                .checked_mul(std::mem::size_of::<serde_json::Value>())
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<u8>>()))
                .ok_or_else(|| projection_error("record projection memory overflow"))?
        }
        input => json_memory(input, 0)?,
    };
    let bytes = input
        .checked_add(identity_bytes)
        .and_then(|bytes| bytes.checked_mul(8))
        .and_then(|bytes| bytes.checked_add(4 * FRAME_BYTES))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<RecordEntry>() + 4096))
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| projection_error("record projection memory overflow"))?;
    Ok(bytes)
}

fn json_memory(value: &serde_json::Value, depth: usize) -> harmos::Result<usize> {
    if depth > 64 {
        return Err(projection_error(
            "record projection input exceeds nesting bound",
        ));
    }
    let mut bytes = std::mem::size_of::<serde_json::Value>();
    match value {
        serde_json::Value::String(value) => {
            bytes = bytes
                .checked_add(value.len())
                .ok_or_else(|| projection_error("record projection memory overflow"))?;
        }
        serde_json::Value::Number(value) => {
            // Covers Arbitrary-Precision Number Buffers Without Formatting A String
            let mut number = NumberBytes(0);
            std::fmt::write(&mut number, format_args!("{value}"))
                .map_err(|_| projection_error("record number memory overflow"))?;
            bytes = bytes
                .checked_add(number.0)
                .ok_or_else(|| projection_error("record projection memory overflow"))?;
        }
        serde_json::Value::Array(values) => {
            let slots = values
                .capacity()
                .checked_mul(std::mem::size_of::<serde_json::Value>())
                .ok_or_else(|| projection_error("record projection memory overflow"))?;
            bytes = bytes
                .checked_add(slots)
                .ok_or_else(|| projection_error("record projection memory overflow"))?;
            for value in values {
                let owned = json_memory(value, depth + 1)?
                    .saturating_sub(std::mem::size_of::<serde_json::Value>());
                bytes = bytes
                    .checked_add(owned)
                    .ok_or_else(|| projection_error("record projection memory overflow"))?;
            }
        }
        serde_json::Value::Object(values) => {
            let nodes = harmos::btree_map_memory::<String, serde_json::Value>(values.len());
            bytes = bytes
                .checked_add(nodes)
                .ok_or_else(|| projection_error("record projection memory overflow"))?;
            for (key, value) in values {
                let owned = json_memory(value, depth + 1)?
                    .saturating_sub(std::mem::size_of::<serde_json::Value>());
                bytes = bytes
                    .checked_add(key.len())
                    .and_then(|bytes| bytes.checked_add(owned))
                    .ok_or_else(|| projection_error("record projection memory overflow"))?;
            }
        }
        _ => {}
    }
    Ok(bytes)
}

pub(crate) fn group_limits() -> ReadLimits {
    ReadLimits {
        rows: NonZeroUsize::new(MAX_GROUP_ROWS).unwrap(),
        bytes: NonZeroUsize::new(4 * FRAME_BYTES).unwrap(),
    }
}

/* === Tests === */
