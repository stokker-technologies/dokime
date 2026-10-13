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
    Identifier, MemorySize, Message,
    storage::{
        QueryOptions, QueryValue, ReadLimits, RecordWrite, SchemaKey, StorageProduction,
        StreamQuery, StreamQueryResult, StreamWrite,
    },
};

// Local Imports
use super::execution::ExecutionView;
use crate::{Error, Result};
use dokime::storage::{
    Manager,
    telemetry::{self, TelemetryEntry, TelemetryRow},
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
    payload: Vec<u8>,
}

pub struct TelemetryReader {
    reader: StreamQueryResult<TelemetryRow>,
    view: ExecutionView,
    context: bool,
    projection: NonZeroUsize,
    pending: Option<QueryValue<Vec<PositionedTelemetry>>>,
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
                source_time: None,
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
        let query = selected_query(
            view,
            QueryOptions {
                page: Some(page),
                decoded_bytes: Some(projection),
                ..QueryOptions::default()
            },
        )?;
        let mut reader = storage.query_in(view.view(), query)?;
        reader.retain_view()?;
        Ok(Self {
            reader,
            view: view.clone(),
            context,
            projection,
            pending: None,
        })
    }

    /// Opens one page of the execution's telemetry after `after` as its own
    /// bounded query, so a capture of any length reads under per-page budgets.
    pub(super) async fn open_after(
        storage: &Manager,
        view: &ExecutionView,
        page: ReadLimits,
        projection: NonZeroUsize,
        after: u64,
    ) -> Result<Self> {
        let query = selected_query(
            view,
            QueryOptions {
                after,
                page: Some(page),
                decoded_bytes: Some(projection),
                ..QueryOptions::default()
            },
        )?;
        let mut reader = storage.query(query).await?;
        // The final page exhausts the reader; its view must outlive that
        // page for the page's own projection.
        reader.retain_view()?;
        Ok(Self {
            reader,
            view: view.clone(),
            context: false,
            projection,
            pending: None,
        })
    }

    /// The next page and the storage position it ends at.
    pub(super) async fn next_after(
        &mut self,
    ) -> Result<Option<(u64, QueryValue<Vec<TelemetryEntry>>)>> {
        let Some(positioned) = self.next_positioned().await? else {
            return Ok(None);
        };
        // The query selects this execution's scope, so every stored row on
        // the page belongs to it; an empty selection cannot advance a cursor.
        let last = positioned
            .last()
            .map(|row| row.position)
            .ok_or_else(|| invalid("execution telemetry page selected no rows"))?;
        self.pending = Some(positioned);
        Ok(self.next().await?.map(|entries| (last, entries)))
    }

    pub(super) fn open_page(
        storage: &Manager,
        view: &ExecutionView,
        stream: &Identifier,
        after: u64,
    ) -> Result<Self> {
        let execution = view.execution().test_execution_id.as_str();
        let selection = stream
            .memory_size()
            .checked_add(std::mem::size_of::<Identifier>())
            .and_then(|bytes| bytes.checked_add(execution.len()))
            .and_then(NonZeroUsize::new)
            .ok_or_else(exhausted)?;
        let _selection = storage.prepare_production(selection)?;
        let mut query = selected_query(
            view,
            QueryOptions {
                after,
                ..QueryOptions::default()
            },
        )?;
        query.stream = Some(stream.clone());
        let mut reader = storage.query_in(view.view(), query)?;
        reader.retain_view()?;
        let projection = reader.decoded_bytes();
        Ok(Self {
            reader,
            view: view.clone(),
            context: false,
            projection,
            pending: None,
        })
    }

    pub(super) fn allowance(&self) -> NonZeroUsize {
        self.projection
    }

    pub(super) fn project<T: MemorySize, U: MemorySize>(
        &mut self,
        value: &QueryValue<T>,
        map: impl FnOnce(&T, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        Ok(self.reader.project(value, self.projection, map)?)
    }

    pub async fn next(&mut self) -> Result<Option<QueryValue<Vec<TelemetryEntry>>>> {
        if self.pending.is_none() {
            self.pending = self.next_positioned().await?;
        }
        let Some(positioned) = &self.pending else {
            return Ok(None);
        };
        let projected = self
            .reader
            .project(positioned, self.projection, |rows, allowance| {
                let memory = rows
                    .iter()
                    .try_fold(std::mem::size_of::<Vec<TelemetryEntry>>(), |bytes, row| {
                        bytes.checked_add(row.captured.memory_size())
                    })
                    .ok_or_else(exhausted)?;
                if memory > allowance.get() {
                    return Err(exhausted());
                }
                let mut entries = Vec::new();
                entries
                    .try_reserve_exact(rows.len())
                    .map_err(|_| exhausted())?;
                if entries.capacity() > rows.len() {
                    return Err(exhausted());
                }
                entries.extend(rows.iter().map(|row| row.captured.clone()));
                Ok(entries)
            })?;
        self.pending = None;
        Ok(Some(projected))
    }

    pub(super) async fn next_positioned(
        &mut self,
    ) -> Result<Option<QueryValue<Vec<PositionedTelemetry>>>> {
        if self.pending.is_some() {
            return Err(invalid(
                "telemetry reader has an unsettled domain projection",
            ));
        }
        let execution = self.view.execution();
        let context = self.context;
        let selected = self
            .reader
            .next_page_projected(self.projection, |rows, allowance| {
                let slots = rows
                    .len()
                    .checked_mul(std::mem::size_of::<PositionedTelemetry>())
                    .and_then(|bytes| {
                        bytes.checked_add(std::mem::size_of::<Vec<PositionedTelemetry>>())
                    })
                    .ok_or_else(|| {
                        codec_error(dokime::storage::Error::Invalid(
                            "telemetry page slot memory overflow",
                        ))
                    })?;
                let bytes = rows
                    .iter()
                    .try_fold(slots, |bytes, row| {
                        bytes
                            .checked_add(row.record.payload.projection_memory()?)
                            .and_then(|bytes| bytes.checked_add(row.record.payload.memory_size()))
                            .and_then(|bytes| bytes.checked_add(row.record.item.memory_size()))
                            .and_then(|bytes| bytes.checked_add(row.record.source.memory_size()))
                            .ok_or(dokime::storage::Error::Invalid(
                                "telemetry projection memory overflow",
                            ))
                    })
                    .map_err(codec_error)?;
                if bytes > allowance.get() {
                    return Err(harmos::Error::ResourceExhausted {
                        resource: "telemetry projection memory",
                    });
                }
                let mut selected = Vec::with_capacity(rows.len());
                let capacity = selected
                    .capacity()
                    .saturating_sub(rows.len())
                    .checked_mul(std::mem::size_of::<PositionedTelemetry>())
                    .and_then(|unused| bytes.checked_add(unused));
                if capacity.is_none_or(|bytes| bytes > allowance.get()) {
                    return Err(harmos::Error::ResourceExhausted {
                        resource: "telemetry page capacity",
                    });
                }
                for row in rows {
                    let entry = row
                        .record
                        .payload
                        .clone()
                        .into_entry(allowance)
                        .map_err(codec_error)?;
                    let scope = entry.sample.scope.as_ref().and_then(|scope| {
                        scope
                            .test_execution_id
                            .as_ref()
                            .map(|id| id.as_str())
                            .or_else(|| scope.test_cycle_id.as_ref().map(|id| id.as_str()))
                    });
                    if row.sequence != entry.ordinal
                        || row.stream != entry.sample.stream
                        || row.record.item.as_str() != entry.sample.sample_id.as_str()
                        || row.record.source.as_ref().map(|source| source.as_str())
                            != Some(entry.epoch.as_str())
                        || row.record.scope.as_ref().map(|scope| scope.as_str()) != scope
                        || row.record.source_time.is_some()
                    {
                        return Err(codec_error(dokime::storage::Error::Invalid(
                            "telemetry envelope differs from original producer identity",
                        )));
                    }
                    let matches = entry.sample.scope.as_ref().is_some_and(|scope| {
                        scope.test_execution_id.as_ref() == Some(&execution.test_execution_id)
                            && scope.test_cycle_id.as_ref() == Some(&execution.test_cycle_id)
                    }) || (context
                        && super::context_records::matches(
                            Some(&execution.test_cycle_id),
                            entry.sample.scope.as_ref(),
                        ));
                    if matches {
                        selected.push(PositionedTelemetry {
                            position: row.record.position.sequence,
                            captured: entry,
                        });
                    }
                }
                Ok(selected)
            })
            .await?;
        Ok(selected)
    }
}

pub(crate) struct TelemetryAdmission {
    storage: Manager,
    rows: Vec<StorageProduction<PreparedTelemetry>>,
    _owners: Vec<tokio::sync::OwnedMutexGuard<super::executions::Acquisition>>,
    _memory: harmos::storage::StorageProductionPreparation,
}

impl TelemetryAdmission {
    pub(crate) async fn commit(self) -> Result<()> {
        if !self.rows.is_empty() {
            let writes = self
                .rows
                .iter()
                .map(|row| row.value().write())
                .collect::<Vec<_>>();
            self.storage.write_stream(&writes).await?;
        }
        Ok(())
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
                    + harmos::btree_map_memory::<&dokime::core::TestExecutionId, ()>(1),
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
        for (sample, _, ordinal) in samples {
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
            rows.push(prepare(
                &self.storage,
                entry.value(),
                NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            )?);
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
pub(crate) fn prepare(
    storage: &Manager,
    entry: &TelemetryEntry,
    limit: NonZeroUsize,
) -> Result<StorageProduction<PreparedTelemetry>> {
    let sample = &entry.sample;
    let scope = captured_scope(sample)?;
    let native = TelemetryRow::encoding_memory(entry)?;
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
        let row = TelemetryRow::from_entry(entry).map_err(codec_error)?;
        let wire = row.encoded_len();
        if simultaneous
            .checked_add(wire)
            .is_none_or(|bytes| bytes > allowance.get())
        {
            return Err(exhausted());
        }
        let mut payload = Vec::new();
        payload.try_reserve_exact(wire).map_err(|_| exhausted())?;
        if simultaneous
            .checked_add(payload.capacity())
            .is_none_or(|bytes| bytes > allowance.get())
        {
            return Err(exhausted());
        }
        row.encode_into(&mut payload);
        if payload.len() != wire
            || simultaneous
                .checked_add(payload.capacity())
                .is_none_or(|bytes| bytes > allowance.get())
        {
            return Err(exhausted());
        }
        drop(row);
        Ok(PreparedTelemetry {
            schema: telemetry::schema(),
            stream: Identifier::new(sample.stream.as_str())
                .map_err(|source| codec_error(source.into()))?,
            item: Identifier::new(sample.sample_id.as_str())
                .map_err(|source| codec_error(source.into()))?,
            source: Identifier::new(entry.epoch.as_str())
                .map_err(|source| codec_error(source.into()))?,
            scope: Some(Identifier::new(scope).map_err(|source| codec_error(source.into()))?),
            ordinal: entry.ordinal,
            payload,
        })
    })?;
    Ok(output)
}

pub(crate) async fn initialize(storage: &Manager) -> Result<()> {
    use harmos::schema::Described;
    let graph = harmos::schema::graph::Schema::from_declaration(
        telemetry::TelemetryRow::DEFINITION,
        &harmos::config::SchemaSettings::default(),
    )
    .map_err(|error| {
        harmos::Error::backend(
            "resolve native telemetry schema",
            harmos::BackendFailure::Permanent,
            error,
        )
    })?;
    let schema = telemetry::schema();
    let mut selection = harmos::storage::SchemaQuery::new();
    selection.schema = Some(schema.clone());
    selection.options.page = Some(ReadLimits {
        rows: NonZeroUsize::new(1).unwrap(),
        bytes: NonZeroUsize::new(64 * 1024).unwrap(),
    });
    let mut schemas = storage.query(selection).await?;
    let existing = schemas.next_page().await?;
    drop(schemas);
    if let Some(existing) = existing {
        if existing.len() != 1 || existing[0].key != schema || existing[0].schema != graph {
            return Err(invalid("stored native telemetry schema differs"));
        }
        return Ok(());
    }
    let declared = storage.write_schema(&schema, &graph).await?;
    storage.flush_until(&declared).await?;
    Ok(())
}

/* === Private Functions === */
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "telemetry projection memory",
    }
}

fn selected_query(
    view: &ExecutionView,
    options: QueryOptions,
) -> Result<StreamQuery<TelemetryRow>> {
    let through = view.evidence_through()?;
    if !view.view().saved()?.covers(through)? {
        return Err(invalid(
            "telemetry evidence exceeds the original saved view",
        ));
    }
    let mut query = native_query(options);
    query.options.through = Some(through.sequence);
    // Native capture admits only execution-addressed samples and files each
    // under its execution, so the envelope scope selects this execution's
    // rows in storage instead of decoding every execution's history.
    query.scope = Some(
        Identifier::new(view.execution().test_execution_id.as_str())
            .map_err(|source| codec_error(source.into()))?,
    );
    Ok(query)
}

pub(crate) fn native_query(options: QueryOptions) -> StreamQuery<TelemetryRow> {
    let mut query = StreamQuery::new(telemetry::schema(), |bytes, allowance| {
        let bound = TelemetryRow::decoding_memory(bytes).map_err(codec_error)?;
        if bound == usize::MAX || bound > allowance.get() {
            return Err(harmos::Error::ResourceExhausted {
                resource: "native telemetry decoding memory",
            });
        }
        TelemetryRow::from_bytes(bytes, allowance.get()).map_err(codec_error)
    });
    query.options = options;
    query
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
