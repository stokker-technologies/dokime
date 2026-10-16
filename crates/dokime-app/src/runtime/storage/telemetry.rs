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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{
    Identifier, MemorySize,
    storage::{QueryValue, ReadLimits, RecordWrite, SchemaKey, StorageProduction, StreamWrite},
};

// Local Imports
use super::execution::ExecutionView;
use crate::{Error, Result};
use dokime::storage::{
    Manager,
    telemetry::{self, TelemetryEntry},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(super) type PositionedTelemetry = super::paging::Positioned<TelemetryEntry>;

/* === Structs === */
pub(crate) struct PreparedTelemetry {
    schema: SchemaKey,
    stream: Identifier,
    item: Identifier,
    source: Identifier,
    scope: Option<Identifier>,
    ordinal: u64,
    source_time: std::time::SystemTime,
    payload: Vec<u8>,
}

pub struct TelemetryReader {
    storage: Manager,
    readers: Vec<TypedRead>,
    page: ReadLimits,
    after: u64,
    _view: ExecutionView,
    projection: NonZeroUsize,
    pending: Option<QueryValue<Vec<PositionedTelemetry>>>,
}

struct TypedRead {
    query: harmos::storage::query::Query,
    codec: telemetry::TypedCodec,
    head: Option<QueryValue<Vec<PositionedTelemetry>>>,
    offset: usize,
    finished: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl MemorySize for PreparedTelemetry {
    fn memory_size(&self) -> usize {
        let fields = [
            (self.schema.memory_size(), std::mem::size_of::<SchemaKey>()),
            (self.stream.memory_size(), std::mem::size_of::<Identifier>()),
            (self.item.memory_size(), std::mem::size_of::<Identifier>()),
            (self.source.memory_size(), std::mem::size_of::<Identifier>()),
            (
                self.scope.memory_size(),
                std::mem::size_of::<Option<Identifier>>(),
            ),
        ];
        fields.into_iter().fold(
            std::mem::size_of::<Self>().saturating_add(self.payload.capacity()),
            |bytes, (owned, inline)| bytes.saturating_add(owned.saturating_sub(inline)),
        )
    }
}

impl PreparedTelemetry {
    pub(crate) fn write(&self) -> StreamWrite<'_> {
        StreamWrite {
            stream: &self.stream,
            sequence: self.ordinal,
            record: RecordWrite {
                schema: &self.schema,
                scope: self.scope.as_ref(),
                item: &self.item,
                source: Some(&self.source),
                source_time: Some(self.source_time),
                payload: &self.payload,
            },
        }
    }
}

impl TelemetryReader {
    pub(crate) fn open(
        storage: &Manager,
        view: &ExecutionView,
        page: ReadLimits,
        projection: NonZeroUsize,
        context: bool,
    ) -> Result<Self> {
        Self::selected(storage, view, page, projection, context, None, 0)
    }

    fn selected(
        storage: &Manager,
        view: &ExecutionView,
        page: ReadLimits,
        projection: NonZeroUsize,
        _context: bool,
        stream: Option<&Identifier>,
        after: u64,
    ) -> Result<Self> {
        use harmos::storage::query::{Query, RowSlice, Source, StreamSelection};
        let captured = view
            .captured_source()
            .ok_or_else(|| invalid("typed telemetry read requires captured producer schemas"))?;
        let scope = Identifier::new(view.execution().test_execution_id.as_str())
            .map_err(|source| codec_error(source.into()))?;
        let mut readers = Vec::new();
        for (id, versions) in &captured.telemetry_contracts {
            if stream.is_some_and(|selected| selected != id) {
                continue;
            }
            for contract in versions.values() {
                let codec = telemetry::codec(contract)?;
                let query = Query {
                    at: view.evidence_through()?.clone(),
                    source: Source::Stream(StreamSelection {
                        schema: codec.schema().clone(),
                        scope: Some(scope.clone()),
                        item: None,
                        stream: Some(contract.stream.clone()),
                        source_start: None,
                        source_end: None,
                    }),
                    transforms: vec![],
                    slice: RowSlice {
                        offset: 0,
                        limit: None,
                    },
                };
                readers.push(TypedRead {
                    query,
                    codec,
                    head: None,
                    offset: 0,
                    finished: false,
                });
            }
        }
        Ok(Self {
            storage: storage.clone(),
            readers,
            page,
            after,
            _view: view.clone(),
            projection,
            pending: None,
        })
    }

    pub(super) fn open_page(
        storage: &Manager,
        view: &ExecutionView,
        stream: &Identifier,
        after: u64,
    ) -> Result<Self> {
        let projection = view.view().decoded_bytes()?;
        let page = ReadLimits {
            rows: NonZeroUsize::new(65_536).unwrap(),
            bytes: projection,
        };
        Self::selected(storage, view, page, projection, false, Some(stream), after)
    }

    pub(super) fn allowance(&self) -> NonZeroUsize {
        self.projection
    }

    pub(super) fn project<T: MemorySize, U: MemorySize>(
        &mut self,
        value: &QueryValue<T>,
        map: impl FnOnce(&T, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        Ok(value.try_project(self.projection, map)?)
    }

    pub async fn next(&mut self) -> Result<Option<QueryValue<Vec<TelemetryEntry>>>> {
        if self.pending.is_none() {
            self.pending = self.next_positioned().await?;
        }
        let Some(positioned) = self.pending.take() else {
            return Ok(None);
        };
        Ok(Some(positioned.try_map(self.projection, |rows, _| {
            Ok(rows.into_iter().map(|row| row.captured).collect())
        })?))
    }

    pub(super) async fn next_positioned(
        &mut self,
    ) -> Result<Option<QueryValue<Vec<PositionedTelemetry>>>> {
        if self.pending.is_some() {
            return Err(invalid(
                "telemetry reader has an unsettled domain projection",
            ));
        }
        // Each Head Retains One Bounded Batch; Merge By Durable Storage Position.
        let mut selected = Vec::new();
        let claim = self.storage.prepare_production(self.projection)?;
        let mut bytes = std::mem::size_of::<Vec<PositionedTelemetry>>();
        while selected.len() < self.page.rows.get() {
            for source in &mut self.readers {
                while !source.finished && source.head.is_none() {
                    // Finish Each Bounded Query Before Opening Another Schema's Head.
                    // Holding Readers For Every Schema Can Exhaust Admission Slots.
                    source.query.slice.limit = Some(self.page.rows.get() as u64);
                    let mut reader = self.storage.query(source.query.clone()).await?;
                    let retained = self.storage.prepare_production(self.projection)?;
                    let mut rows = Vec::new();
                    let mut count = 0u64;
                    let mut decoded_bytes = std::mem::size_of::<Vec<PositionedTelemetry>>();
                    while let Some(output) = reader.next().await? {
                        let harmos::storage::query::QueryOutput::Batch(batch) = output else {
                            continue;
                        };
                        let raw = batch.as_record_batch();
                        count = count
                            .checked_add(raw.num_rows() as u64)
                            .ok_or_else(exhausted)?;
                        let metadata = raw
                            .column_by_name("_storage")
                            .and_then(|column| {
                                column.as_any().downcast_ref::<arrow_array::StructArray>()
                            })
                            .ok_or_else(|| invalid("typed telemetry storage metadata is absent"))?;
                        let positions = metadata
                            .column_by_name("position")
                            .and_then(|column| {
                                column.as_any().downcast_ref::<arrow_array::UInt64Array>()
                            })
                            .ok_or_else(|| invalid("typed telemetry positions are absent"))?;
                        let decoded =
                            telemetry::decode_batch(&source.codec, &batch, self.projection)?;
                        for (index, captured) in decoded.into_iter().enumerate() {
                            if positions.value(index) <= self.after {
                                continue;
                            }
                            let row = PositionedTelemetry {
                                position: positions.value(index),
                                captured,
                            };
                            decoded_bytes = decoded_bytes
                                .checked_add(row.memory_size())
                                .ok_or_else(exhausted)?;
                            if decoded_bytes > self.projection.get() {
                                return Err(exhausted().into());
                            }
                            rows.try_reserve_exact(1).map_err(|_| exhausted())?;
                            rows.push(row);
                        }
                    }
                    reader.completion()?;
                    source.query.slice.offset = source
                        .query
                        .slice
                        .offset
                        .checked_add(count)
                        .ok_or_else(exhausted)?;
                    source.finished = count < self.page.rows.get() as u64;
                    if !rows.is_empty() {
                        source.head = Some(retained.finish(rows)?.try_into_value()?);
                        source.offset = 0;
                    }
                }
            }
            let next = self
                .readers
                .iter()
                .enumerate()
                .filter_map(|(index, source)| {
                    source
                        .head
                        .as_ref()
                        .map(|rows| (index, rows[source.offset].position))
                })
                .min_by_key(|(_, position)| *position);
            let Some((index, _)) = next else {
                break;
            };
            let source = &mut self.readers[index];
            let rows = source.head.as_ref().expect("selected head exists");
            let row = &rows[source.offset];
            let needed = bytes.checked_add(row.memory_size()).ok_or_else(exhausted)?;
            if needed > self.projection.get() {
                if selected.is_empty() {
                    return Err(exhausted().into());
                }
                break;
            }
            selected.push(PositionedTelemetry {
                position: row.position,
                captured: row.captured.clone(),
            });
            bytes = needed;
            source.offset += 1;
            if source.offset == rows.len() {
                source.head = None;
            }
        }
        if selected.is_empty() {
            return Ok(None);
        }
        Ok(Some(claim.finish(selected)?.try_into_value()?))
    }
}

pub(crate) struct TelemetryAdmission {
    storage: Manager,
    rows: Vec<StorageProduction<PreparedTelemetry>>,
    _owners: Vec<tokio::sync::OwnedMutexGuard<super::executions::Acquisition>>,
    _memory: harmos::storage::StorageProductionPreparation,
}

pub(crate) struct CommittedTelemetry {
    pub accepted: QueryValue<Vec<bool>>,
    pub failure: Option<Error>,
}

impl TelemetryAdmission {
    pub(crate) async fn commit(
        self,
        sampling: &crate::services::telemetry::Sampling,
    ) -> Result<CommittedTelemetry> {
        let bound = NonZeroUsize::new(
            std::mem::size_of::<Vec<bool>>()
                .checked_add(self.rows.len())
                .ok_or_else(exhausted)?,
        )
        .ok_or_else(exhausted)?;
        let retained = self.storage.prepare_production(bound)?;
        let mut accepted = vec![false; self.rows.len()];
        let mut groups = std::collections::BTreeMap::<&str, Vec<usize>>::new();
        for (index, row) in self.rows.iter().enumerate() {
            let scope = row
                .value()
                .scope
                .as_ref()
                .ok_or_else(|| invalid("native telemetry has no execution owner"))?;
            groups.entry(scope.as_str()).or_default().push(index);
        }
        let mut failure = None;
        'groups: for (scope, indices) in groups {
            let writes = indices
                .iter()
                .map(|index| self.rows[*index].value().write())
                .collect::<Vec<_>>();
            let mut offset = 0;
            while offset < writes.len() {
                if sampling.measurement_closed(scope).await {
                    break;
                }
                let length = match self.storage.stream_batch_len(&writes[offset..]) {
                    Ok(length) => length,
                    Err(source) if accepted.iter().any(|accepted| *accepted) => {
                        failure = Some(source.into());
                        break 'groups;
                    }
                    Err(source) => return Err(source.into()),
                };
                let batch = &writes[offset..offset + length];
                // Harmos Enqueue Has No Await Between Ownership Transfer And Its Ready Receipt.
                // Poll The Receipt First; Cancellation Only Drops An Unaccepted Pending Enqueue.
                tokio::select! {
                    biased;
                    result = self.storage.write_stream(batch) => {
                        match result {
                            Ok(_) => for index in &indices[offset..offset + length] { accepted[*index] = true; },
                            Err(source) if accepted.iter().any(|accepted| *accepted) => {
                                failure = Some(source.into());
                                break 'groups;
                            }
                            Err(source) => return Err(source.into()),
                        }
                    }
                    _ = sampling.wait_measurement_closed(scope) => break,
                }
                offset += length;
            }
        }
        Ok(CommittedTelemetry {
            accepted: retained.finish(accepted)?.try_into_value()?,
            failure,
        })
    }
}

impl super::executions::ExecutionStorage {
    pub(crate) async fn telemetry_scopes(
        &self,
    ) -> Result<
        QueryValue<
            std::collections::BTreeMap<dokime::core::TestExecutionId, dokime::core::RecordScope>,
        >,
    > {
        let limit = NonZeroUsize::new(4 * 1024 * 1024).unwrap();
        let ids = self.active_ids(limit)?;
        let slots = ids
            .len()
            .checked_mul(std::mem::size_of::<
                tokio::sync::OwnedMutexGuard<super::executions::Acquisition>,
            >())
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<
                    Vec<tokio::sync::OwnedMutexGuard<super::executions::Acquisition>>,
                >())
            })
            .and_then(NonZeroUsize::new)
            .ok_or_else(exhausted)?;
        let _guards = self.storage.prepare_production(slots)?;
        let mut owners = Vec::new();
        owners
            .try_reserve_exact(ids.len())
            .map_err(|_| exhausted())?;
        if owners.capacity() > ids.len() {
            return Err(exhausted().into());
        }
        // Sorted Owner Locks Never Hold The Registry Across An Await.
        for id in ids.iter() {
            if let Some(owner) = self.acquisition(id).await {
                owners.push(owner.lock_owned().await);
            }
        }
        let open = || owners.iter().filter(|owner| !owner.finish_requested);
        let bytes = open().try_fold(
            harmos::btree_map_memory::<dokime::core::TestExecutionId, dokime::core::RecordScope>(
                open().count(),
            ),
            |bytes, owner| {
                bytes
                    .checked_add(owner.execution.test_execution_id.memory_size())
                    .and_then(|bytes| bytes.checked_add(owner.execution.scope.memory_size()))
                    .ok_or_else(exhausted)
            },
        )?;
        if bytes > limit.get() {
            return Err(exhausted().into());
        }
        let claim = self
            .storage
            .prepare_production(NonZeroUsize::new(bytes.max(1)).unwrap())?;
        let scopes = open()
            .map(|owner| {
                (
                    owner.execution.test_execution_id.clone(),
                    owner.execution.scope.clone(),
                )
            })
            .collect();
        Ok(claim.finish(scopes)?.try_into_value()?)
    }

    pub(crate) async fn prepare_telemetry(
        &self,
        samples: &[(
            dokime::telemetry::TelemetrySample,
            dokime::telemetry::TelemetryContract,
            u64,
        )],
    ) -> Result<TelemetryAdmission> {
        let slots = samples
            .len()
            .checked_mul(
                std::mem::size_of::<StorageProduction<PreparedTelemetry>>()
                    + std::mem::size_of::<StreamWrite<'_>>()
                    + std::mem::size_of::<
                        tokio::sync::OwnedMutexGuard<super::executions::Acquisition>,
                    >()
                    + 8 * std::mem::size_of::<usize>()
                    + harmos::btree_map_memory::<&str, Vec<usize>>(1),
            )
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<TelemetryAdmission>()))
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| invalid("telemetry admission size overflow"))?;
        let memory = self.storage.prepare_production(slots)?;
        let ids = samples
            .iter()
            .filter_map(|(sample, _, _)| sample.scope.as_ref()?.test_execution_id.as_ref())
            .collect::<std::collections::BTreeSet<_>>();
        let mut owners = Vec::with_capacity(ids.len());
        for id in ids {
            let owner = self
                .acquisition(id)
                .await
                .ok_or_else(|| invalid("telemetry has no execution owner"))?;
            let acquisition = owner.lock_owned().await;
            if acquisition.finish_requested {
                return Err(invalid("telemetry execution admission is closed"));
            }
            for (sample, _, _) in samples {
                if sample
                    .scope
                    .as_ref()
                    .and_then(|scope| scope.test_execution_id.as_ref())
                    == Some(id)
                    && sample.scope.as_ref().is_some_and(|scope| {
                        let owner = &acquisition.execution.scope;
                        scope.session_id != owner.session_id
                            || scope.test_cycle_id != owner.test_cycle_id
                            || scope.test_case_id != owner.test_case_id
                            || scope.test_execution_id != owner.test_execution_id
                            || scope.template_id != owner.template_id
                            || scope.suite_id != owner.suite_id
                    })
                {
                    return Err(invalid("telemetry execution scope differs from its owner"));
                }
            }
            owners.push(acquisition);
        }
        let mut rows = Vec::with_capacity(samples.len());
        let mut codecs = std::collections::BTreeMap::new();
        for (sample, contract, ordinal) in samples {
            if sample
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref())
                .is_none()
            {
                continue;
            }
            let bytes = sample
                .memory_size()
                .saturating_add(self.context_epoch.memory_size())
                .saturating_add(std::mem::size_of::<TelemetryEntry>());
            let entry = self.storage.produce(
                NonZeroUsize::new(bytes).ok_or_else(|| invalid("telemetry input size overflow"))?,
                |_| {
                    Ok(TelemetryEntry {
                        epoch: self.context_epoch.clone(),
                        ordinal: *ordinal,
                        sample: sample.clone(),
                    })
                },
            )?;
            let key = (contract.stream.clone(), contract.schema_version);
            if !codecs.contains_key(&key) {
                let graph = contract.storage_schema.as_ref().ok_or_else(|| {
                    invalid("telemetry producer requires an explicit storage row schema")
                })?;
                let codec = telemetry::TypedCodec::new(
                    &contract.stream,
                    contract.schema_version,
                    graph,
                    &harmos::config::SchemaSettings::default(),
                )?;
                self.storage
                    .write_schema(codec.schema(), codec.graph())
                    .await?;
                codecs.insert(key.clone(), codec);
            }
            let codec = &codecs[&key];
            let limit = NonZeroUsize::new(
                codec
                    .encoding_memory(entry.value())?
                    .saturating_add(entry.value().memory_size())
                    .saturating_add(std::mem::size_of::<PreparedTelemetry>()),
            )
            .ok_or_else(|| invalid("typed telemetry encoding memory overflows"))?;
            rows.push(prepare(&self.storage, entry.value(), codec, limit)?);
        }
        Ok(TelemetryAdmission {
            storage: self.storage.clone(),
            rows,
            _owners: owners,
            _memory: memory,
        })
    }
}

/* === Public Functions === */
/// Producer Schemas Are Durable Before An Execution Can Start Acquiring.
pub(crate) async fn register_schemas(
    storage: &Manager,
    contracts: &[dokime::telemetry::TelemetryContract],
) -> Result<()> {
    let mut last = None;
    for contract in contracts.iter().filter(|contract| !contract.foreign) {
        let codec = telemetry::codec(contract)?;
        last = Some(storage.write_schema(codec.schema(), codec.graph()).await?);
    }
    if let Some(cursor) = last {
        storage.flush_until(&cursor).await?;
    }
    Ok(())
}

pub(crate) fn prepare(
    storage: &Manager,
    entry: &TelemetryEntry,
    codec: &telemetry::TypedCodec,
    limit: NonZeroUsize,
) -> Result<StorageProduction<PreparedTelemetry>> {
    let sample = &entry.sample;
    let scope = captured_scope(sample)?;
    let native = codec.encoding_memory(entry)?;
    let metadata = [
        sample.stream.as_str(),
        sample.sample_id.as_str(),
        entry.epoch.as_str(),
        scope,
        "dokime.capture.telemetry",
    ]
    .into_iter()
    .try_fold(std::mem::size_of::<PreparedTelemetry>(), |bytes, text| {
        bytes.checked_add(text.len())
    });
    let output = storage.produce(limit, |allowance| {
        let exhausted = || harmos::Error::ResourceExhausted {
            resource: "native telemetry production memory",
        };
        let metadata = metadata.ok_or_else(exhausted)?;
        let simultaneous = native.checked_add(metadata).ok_or_else(exhausted)?;
        if native == usize::MAX || simultaneous > allowance.get() {
            return Err(exhausted());
        }
        let payload = codec.encode(entry, allowance.get()).map_err(codec_error)?;
        Ok(PreparedTelemetry {
            schema: codec.schema().clone(),
            stream: Identifier::new(sample.stream.as_str())
                .map_err(|source| codec_error(source.into()))?,
            item: Identifier::new(sample.sample_id.as_str())
                .map_err(|source| codec_error(source.into()))?,
            source: Identifier::new(entry.epoch.as_str())
                .map_err(|source| codec_error(source.into()))?,
            scope: Some(Identifier::new(scope).map_err(|source| codec_error(source.into()))?),
            ordinal: entry.ordinal,
            source_time: {
                let at = sample.timestamp.to_datetime().map_err(|_| {
                    codec_error(dokime::storage::Error::Invalid(
                        "typed telemetry timestamp is invalid",
                    ))
                })?;
                let seconds = u64::try_from(at.unix_timestamp()).map_err(|_| exhausted())?;
                std::time::UNIX_EPOCH
                    .checked_add(std::time::Duration::new(seconds, at.nanosecond()))
                    .ok_or_else(exhausted)?
            },
            payload,
        })
    })?;
    Ok(output)
}

/* === Private Functions === */
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "telemetry projection memory",
    }
}

fn captured_scope(sample: &dokime::telemetry::TelemetrySample) -> Result<&str> {
    sample
        .scope
        .as_ref()
        .and_then(|scope| scope.test_execution_id.as_ref())
        .map(|id| id.as_str())
        .ok_or_else(|| invalid("native telemetry capture requires an execution address"))
}

fn codec_error(source: dokime::storage::Error) -> harmos::Error {
    match source {
        dokime::storage::Error::Runtime(source) => source,
        source => harmos::Error::backend(
            "decode native captured telemetry",
            harmos::BackendFailure::Corrupt,
            source,
        ),
    }
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

/* === Tests === */

#[cfg(test)]
mod admission_tests {
    use super::*;
    use harmos::{Application, Runtime, storage::StorageBackends};
    use std::{sync::Arc, time::Duration};

    struct TestApp;
    impl Application for TestApp {
        type State = ();
        type Transaction = harmos::state::Never;
        type Record = harmos::state::Never;
        type Streams = ();
        type Resources = ();
    }

    #[harmos::message]
    struct Reading {
        value: String,
    }

    #[tokio::test]
    async fn later_refusal_preserves_the_accepted_prefix_without_replay() {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(
            dokime::storage::sqlite::Backend::primary(
                root.path().to_owned(),
                dokime::storage::sqlite::Limits::default(),
            )
            .await
            .unwrap(),
        );
        let settings = crate::config::StorageSettings::default().harmos(
            Duration::from_secs(60),
            NonZeroUsize::new(128 * 1024).unwrap(),
        );
        let state = Arc::new(harmos::storage::state::file::FileBackend::new(
            root.path().to_owned(),
            harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
            &settings,
        ));
        let storage = Manager::new(
            StorageBackends {
                primary: backend,
                state,
                replicas: Vec::new(),
            },
            settings,
        )
        .unwrap();
        let runtime = Runtime::<TestApp>::builder((), ())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        let contract = dokime::telemetry::TelemetryContract::declared("partial.fixture", "Partial")
            .typed::<Reading>();
        let codec = telemetry::codec(&contract).unwrap();
        storage
            .write_schema(codec.schema(), codec.graph())
            .await
            .unwrap();
        let execution = dokime::core::TestExecutionId::from_seed("partial.scope");
        let mut rows = Vec::new();
        for (index, value) in ["first".to_owned(), "x".repeat(256 * 1024)]
            .into_iter()
            .enumerate()
        {
            let row = Reading { value };
            let mut sample = dokime::telemetry::TelemetrySample::new(
                contract.stream.clone(),
                index as u64,
                dokime::core::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH),
                "partial.fixture",
            )
            .typed(&row);
            sample.scope = Some(dokime::core::RecordScope {
                test_execution_id: Some(execution.clone()),
                ..Default::default()
            });
            let entry = TelemetryEntry {
                epoch: dokime::core::SessionId::from_seed("partial.epoch"),
                ordinal: index as u64,
                sample,
            };
            let bound = codec
                .encoding_memory(&entry)
                .unwrap()
                .saturating_add(entry.memory_size())
                .saturating_add(std::mem::size_of::<PreparedTelemetry>());
            rows.push(
                prepare(&storage, &entry, &codec, NonZeroUsize::new(bound).unwrap()).unwrap(),
            );
        }
        let before = storage.cursor_accepted();
        let admission = TelemetryAdmission {
            storage: storage.clone(),
            rows,
            _owners: Vec::new(),
            _memory: storage
                .prepare_production(NonZeroUsize::new(4096).unwrap())
                .unwrap(),
        };
        let result = admission
            .commit(&crate::services::telemetry::Sampling::default())
            .await
            .unwrap();
        assert_eq!(&*result.accepted, &[true, false]);
        assert!(result.failure.is_some());
        assert_eq!(storage.cursor_accepted().sequence, before.sequence + 1);
        drop(result);
        storage.flush().await.unwrap();
        runtime.stop().await.unwrap();
    }
}
