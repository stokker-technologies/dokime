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
    Identifier,
    storage::{
        RecordWrite, SchemaKey, StreamWrite,
        query::{Query, QueryOutput, RowSlice, Source, StreamSelection},
    },
};

// Local Imports
use super::execution::ExecutionView;
use crate::{Error, Result};
use dokime::storage::{
    Manager,
    telemetry::{self, TelemetryEntry},
};

/* === Constants === */
// Rows Each Typed Head Reads Per Query Before The Merge Consumes Them.
const PAGE_ROWS: usize = 65_536;

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

/// An execution's typed telemetry at its frozen evidence cut, merged by position.
pub struct TelemetryReader {
    storage: Manager,
    readers: Vec<TypedRead>,
}

struct TypedRead {
    query: Query,
    codec: telemetry::TypedCodec,
    head: Option<Vec<PositionedTelemetry>>,
    offset: usize,
    finished: bool,
}

pub(crate) struct TelemetryAdmission {
    storage: Manager,
    rows: Vec<PreparedTelemetry>,
    _owners: Vec<tokio::sync::OwnedMutexGuard<super::executions::Acquisition>>,
}

pub(crate) struct CommittedTelemetry {
    pub accepted: Vec<bool>,
    pub failure: Option<Error>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
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
    pub(crate) fn open(view: &ExecutionView, _context: bool) -> Result<Self> {
        Self::selected(view, None, 0)
    }

    pub(super) fn open_page(view: &ExecutionView, stream: &Identifier, after: u64) -> Result<Self> {
        Self::selected(view, Some(stream), after)
    }

    fn selected(view: &ExecutionView, stream: Option<&Identifier>, after: u64) -> Result<Self> {
        let captured = view
            .captured_source()
            .ok_or_else(|| invalid("typed telemetry read requires captured producer schemas"))?;
        let scope = Identifier::new(view.execution().test_execution_id.as_str())
            .map_err(dokime::storage::Error::from)?;
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
                        after,
                    }),
                    transforms: vec![],
                    slice: RowSlice::default(),
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
            storage: view.storage().clone(),
            readers,
        })
    }

    pub async fn next(&mut self) -> Result<Option<Vec<TelemetryEntry>>> {
        Ok(self
            .next_positioned()
            .await?
            .map(|rows| rows.into_iter().map(|row| row.captured).collect()))
    }

    pub(super) async fn next_positioned(&mut self) -> Result<Option<Vec<PositionedTelemetry>>> {
        // Each Head Retains One Bounded Batch; Merge By Durable Storage Position.
        let mut selected = Vec::new();
        while selected.len() < PAGE_ROWS {
            for source in &mut self.readers {
                while !source.finished && source.head.is_none() {
                    source.head = read_head(&self.storage, source).await?;
                    source.offset = 0;
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
            selected.push(PositionedTelemetry {
                position: row.position,
                captured: row.captured.clone(),
            });
            source.offset += 1;
            if source.offset == rows.len() {
                source.head = None;
            }
        }
        Ok((!selected.is_empty()).then_some(selected))
    }
}

impl TelemetryAdmission {
    pub(crate) async fn commit(
        self,
        closure: Option<&crate::services::telemetry::Sampling>,
    ) -> Result<CommittedTelemetry> {
        let mut accepted = vec![false; self.rows.len()];
        let mut groups = std::collections::BTreeMap::<&str, Vec<usize>>::new();
        for (index, row) in self.rows.iter().enumerate() {
            let scope = row
                .scope
                .as_ref()
                .ok_or_else(|| invalid("native telemetry has no execution owner"))?;
            groups.entry(scope.as_str()).or_default().push(index);
        }
        let mut failure = None;
        'groups: for (scope, indices) in groups {
            let writes = indices
                .iter()
                .map(|index| self.rows[*index].write())
                .collect::<Vec<_>>();
            let mut offset = 0;
            while offset < writes.len() {
                if let Some(sampling) = closure
                    && sampling.measurement_closed(scope).await
                {
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
                    () = closed(closure, scope) => break,
                }
                offset += length;
            }
        }
        Ok(CommittedTelemetry { accepted, failure })
    }
}

impl super::executions::ExecutionStorage {
    pub(crate) async fn telemetry_scopes(
        &self,
    ) -> Result<std::collections::BTreeMap<dokime::core::TestExecutionId, dokime::core::RecordScope>>
    {
        let ids = self.active_ids()?;
        let mut scopes = std::collections::BTreeMap::new();
        // Sorted Owner Locks Never Hold The Registry Across An Await.
        for id in &ids {
            if let Some(owner) = self.acquisition(id).await {
                let owner = owner.lock_owned().await;
                if !owner.finish_requested {
                    scopes.insert(
                        owner.execution.test_execution_id.clone(),
                        owner.execution.scope.clone(),
                    );
                }
            }
        }
        Ok(scopes)
    }

    pub(crate) async fn prepare_telemetry(
        &self,
        samples: &[(
            dokime::telemetry::TelemetrySample,
            dokime::telemetry::TelemetryContract,
            u64,
        )],
    ) -> Result<TelemetryAdmission> {
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
            let entry = TelemetryEntry {
                epoch: self.context_epoch.clone(),
                ordinal: *ordinal,
                sample: sample.clone(),
            };
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
            rows.push(prepare(&entry, &codecs[&key])?);
        }
        Ok(TelemetryAdmission {
            storage: self.storage.clone(),
            rows,
            _owners: owners,
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
    entry: &TelemetryEntry,
    codec: &telemetry::TypedCodec,
) -> Result<PreparedTelemetry> {
    let sample = &entry.sample;
    let scope = captured_scope(sample)?;
    let identifier = |value: &str| Identifier::new(value).map_err(dokime::storage::Error::from);
    let at = sample
        .timestamp
        .to_datetime()
        .map_err(|_| invalid("typed telemetry timestamp is invalid"))?;
    let seconds =
        u64::try_from(at.unix_timestamp()).map_err(|_| invalid("typed telemetry predates 1970"))?;
    Ok(PreparedTelemetry {
        schema: codec.schema().clone(),
        stream: identifier(sample.stream.as_str())?,
        item: identifier(sample.sample_id.as_str())?,
        source: identifier(entry.epoch.as_str())?,
        scope: Some(identifier(scope)?),
        ordinal: entry.ordinal,
        source_time: std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::new(seconds, at.nanosecond()))
            .ok_or_else(|| invalid("typed telemetry timestamp overflows"))?,
        payload: codec.encode(entry, usize::MAX)?,
    })
}

/* === Private Functions === */
/// Resolves When Live Measurement Of The Scope Closes; Never Without A Closure Owner.
async fn closed(closure: Option<&crate::services::telemetry::Sampling>, scope: &str) {
    match closure {
        Some(sampling) => sampling.wait_measurement_closed(scope).await,
        None => std::future::pending().await,
    }
}
// One Bounded Query Per Head, Resuming After The Last Row The Head Read;
// Holding Readers For Every Schema Can Exhaust Admission.
async fn read_head(
    storage: &Manager,
    source: &mut TypedRead,
) -> Result<Option<Vec<PositionedTelemetry>>> {
    source.query.slice.limit = Some(PAGE_ROWS as u64);
    let mut reader = storage.query(source.query.clone()).await?;
    let mut rows = Vec::new();
    while let Some(output) = reader.next().await? {
        let QueryOutput::Batch(batch) = output else {
            continue;
        };
        let raw = batch.as_record_batch();
        let positions = raw
            .column_by_name("_storage")
            .and_then(|column| column.as_any().downcast_ref::<arrow_array::StructArray>())
            .and_then(|metadata| metadata.column_by_name("position"))
            .and_then(|column| column.as_any().downcast_ref::<arrow_array::UInt64Array>())
            .ok_or_else(|| invalid("typed telemetry positions are absent"))?;
        let decoded = telemetry::decode_batch(&source.codec, &batch)?;
        for (index, captured) in decoded.into_iter().enumerate() {
            rows.push(PositionedTelemetry {
                position: positions.value(index),
                captured,
            });
        }
    }
    reader.completion()?;
    source.finished = rows.len() < PAGE_ROWS;
    if let (Some(last), Source::Stream(selection)) = (rows.last(), &mut source.query.source) {
        selection.after = last.position;
    }
    Ok((!rows.is_empty()).then_some(rows))
}

fn captured_scope(sample: &dokime::telemetry::TelemetrySample) -> Result<&str> {
    sample
        .scope
        .as_ref()
        .and_then(|scope| scope.test_execution_id.as_ref())
        .map(|id| id.as_str())
        .ok_or_else(|| invalid("native telemetry capture requires an execution address"))
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

/* === Tests === */

#[cfg(test)]
mod admission_tests {
    use super::*;
    use harmos::{Application, Runtime, storage::StorageBackends};
    use std::{num::NonZeroUsize, sync::Arc};

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
            dokime::storage::Backend::primary(root.path().to_owned())
                .await
                .unwrap(),
        );
        let mut settings = crate::config::storage_defaults();
        settings.request.max_pending_write_bytes = NonZeroUsize::new(128 * 1024).unwrap();
        let state = Arc::new(harmos::storage::state::file::FileBackend::new(
            root.path().to_owned(),
            harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
            dokime::storage::state_settings(),
        ));
        let storage = Manager::new(
            StorageBackends {
                primary: backend,
                state,
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
            rows.push(prepare(&entry, &codec).unwrap());
        }
        let before = storage.cursor_accepted();
        let admission = TelemetryAdmission {
            storage: storage.clone(),
            rows,
            _owners: Vec::new(),
        };
        let result = admission
            .commit(Some(&crate::services::telemetry::Sampling::default()))
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
