/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod reader;

/* === Re-Exports === */

pub use reader::TelemetryReader;

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use harmos::{
    Identifier,
    storage::{
        Query, RecordInput, SchemaKey, StreamInput,
        query::{QueryArrow, RowSlice, Source, StreamSelection},
    },
};

// Local Imports
use crate::storage::{
    Manager,
    telemetry::{self, TelemetryEntry},
};
use crate::{runtime::Error, runtime::Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub(super) type PositionedTelemetry = super::paging::Positioned<TelemetryEntry>;

/* === Structs === */

#[derive(Default)]
pub(crate) struct ReadOptions<'a> {
    pub scan_limit: Option<usize>,
    pub window: Option<(&'a str, f64)>,
    pub visible: Option<&'a dokime_domain::model::execution::RecordScope>,
    pub sequence: Sequence,
    // Storage Position At Or Below Which Every Selected Row Is Already Known; The Scan Starts After It.
    pub after: u64,
}

/// One Bounded Page In Storage Order: Rows Through The Last Position Are Settled For The Selection.
pub struct TelemetryPage {
    pub samples: harmos::storage::ReadBatch<Vec<crate::telemetry::TelemetrySample>>,
    pub positions: Vec<u64>,
    pub limited: bool,
}

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

// One Retained Admission Row; Scope References Never Own Payload Or Value Copies.
struct SharedTelemetry {
    epoch: Arc<dokime_domain::model::identity::SessionId>,
    sample: Arc<crate::telemetry::TelemetrySample>,
    codec: Arc<telemetry::TypedCodec>,
}

pub(crate) struct ScopedTelemetry {
    row: Arc<SharedTelemetry>,
    scope: Option<dokime_domain::model::execution::RecordScope>,
    ordinal: u64,
}

pub(crate) struct TelemetryAdmission {
    storage: Manager,
    rows: Vec<(usize, ScopedTelemetry)>,
    _owners: Vec<tokio::sync::OwnedMutexGuard<super::executions::Acquisition>>,
}

pub(crate) struct CommittedTelemetry {
    pub accepted: Vec<bool>,
    pub failure: Option<Error>,
}

/* === Enums === */

#[derive(Clone, Copy, Default)]
pub(crate) enum Sequence {
    #[default]
    StreamOrdinal,
    StoragePosition,
}

/* === Traits === */

/* === Implementations === */

impl Sequence {
    fn sample(self, mut entry: TelemetryEntry, position: u64) -> crate::telemetry::TelemetrySample {
        entry.sample.sequence = match self {
            Self::StreamOrdinal => entry.ordinal,
            Self::StoragePosition => position,
        };
        entry.sample.values.retain(|_, value| !matches!(value, dokime_domain::model::values::Value::Null));
        entry.sample
    }
}

impl PreparedTelemetry {
    pub(crate) fn into_input(self) -> StreamInput {
        StreamInput {
            stream: self.stream,
            sequence: self.ordinal,
            record: RecordInput {
                schema: self.schema,
                scope: self.scope,
                item: self.item,
                source: Some(self.source),
                source_time: Some(self.source_time),
                payload: self.payload,
            },
        }
    }
}

impl ScopedTelemetry {
    fn execution(&self) -> Option<&dokime_domain::model::identity::TestExecutionId> {
        self.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref())
    }

    fn into_input(self) -> Result<StreamInput> {
        let prepared = prepare_staged(&self.row.epoch, &self.row.sample, self.scope.as_ref(), self.ordinal, &self.row.codec)?;
        Ok(prepared.into_input())
    }
}

impl TelemetryAdmission {
    pub(crate) async fn commit(mut self, closure: Option<&crate::services::telemetry::Sampling>) -> Result<CommittedTelemetry> {
        self.rows.sort_unstable_by(|left, right| left.1.execution().cmp(&right.1.execution()).then(left.0.cmp(&right.0)));
        let mut accepted = crate::domains::telemetry::telemetry_slots(self.rows.len())?;
        accepted.resize(self.rows.len(), false);
        let mut rows = self.rows.into_iter();
        let mut failure = None;
        'groups: while let Some((first_index, first)) = rows.next() {
            let count = 1 + rows.as_slice().iter().take_while(|(_, row)| row.execution() == first.execution()).count();
            let scope = first.execution().map(ToString::to_string);
            let mut indices = crate::domains::telemetry::telemetry_slots(count)?;
            let mut writes = crate::domains::telemetry::telemetry_slots(count)?;
            indices.push(first_index);
            writes.push(first);
            for _ in 1..count {
                let (index, row) = rows.next().ok_or_else(|| invalid("telemetry scope group is incomplete"))?;
                indices.push(index);
                writes.push(row);
            }
            // Materialize Only The Scope Being Committed, Releasing It Before The Next Scope.
            let mut inputs = crate::domains::telemetry::telemetry_slots(count)?;
            for row in writes {
                match row.into_input() {
                    Ok(input) => inputs.push(input),
                    Err(source) if accepted.iter().any(|accepted| *accepted) => {
                        failure = Some(source);
                        break 'groups;
                    }
                    Err(source) => return Err(source),
                }
            }
            let mut writes = inputs.into_iter();
            let mut offset = 0;
            while writes.len() > 0 {
                if let (Some(sampling), Some(scope)) = (closure, scope.as_deref())
                    && sampling.measurement_closed(scope).await
                {
                    break;
                }
                let length = match self.storage.stream_batch_len(writes.as_slice()) {
                    Ok(length) => length,
                    Err(source) if accepted.iter().any(|accepted| *accepted) => {
                        failure = Some(source.into());
                        break 'groups;
                    }
                    Err(source) => return Err(source.into()),
                };
                if length == 0 {
                    return Err(invalid("native telemetry batch made no progress"));
                }
                let mut batch = crate::domains::telemetry::telemetry_slots(length)?;
                batch.extend(writes.by_ref().take(length));
                // Once Ownership Transfers, Settle Durability Before Reporting Accepted Rows.
                tokio::select! {
                    biased;
                    result = self.storage.write_stream(batch) => {
                        match result {
                            Ok(cursor) => {
                                if let Err(source) = self.storage.flush_until(&cursor).await {
                                    if accepted.iter().any(|accepted| *accepted) {
                                        failure = Some(source.into());
                                        break 'groups;
                                    }
                                    return Err(source.into());
                                }
                                for index in &indices[offset..offset + length] { accepted[*index] = true; }
                            },
                            Err(source) if accepted.iter().any(|accepted| *accepted) => { failure = Some(source.into()); break 'groups; }
                            Err(source) => return Err(source.into()),
                        }
                    }
                    () = closed(closure, scope.as_deref()) => break,
                }
                offset += length;
            }
        }
        Ok(CommittedTelemetry { accepted, failure })
    }
}

impl super::executions::ExecutionStorage {
    pub(crate) fn with_telemetry_scopes<T>(
        &self,
        read: impl FnOnce(
            &std::collections::BTreeMap<dokime_domain::model::identity::TestExecutionId, dokime_domain::model::execution::RecordScope>,
        ) -> T,
    ) -> T {
        let open = self.open_scopes.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        read(&open)
    }

    // Saves Each Owned Typed Schema Once Per Process; Every Start Repeats The Request.
    pub(crate) async fn register_schemas(&self, contracts: &[crate::telemetry::TelemetryContract]) -> Result<()> {
        let storage = &self.storage;
        let mut last = None;
        let mut written = Vec::new();
        for contract in contracts.iter().filter(|contract| !contract.foreign) {
            let Some(codec) = self.codecs.unregistered(contract, &self.schema_settings)? else {
                continue;
            };
            let mutation = harmos::storage::MutationId {
                window: storage.mutation_window()?.id,
                key: Identifier::new(format!("{}.v{}", codec.schema().id, codec.schema().version)).map_err(crate::storage::Error::from)?,
            };
            last = Some(storage.write_schema_once(&mutation, codec.schema().clone(), Arc::clone(codec.graph())).await?);
            written.push((contract, codec));
        }
        if let Some(cursor) = last {
            storage.flush_until(&cursor).await?;
        }
        for (contract, codec) in written {
            self.codecs.mark_registered(contract, &codec);
        }
        Ok(())
    }

    // The Saved Codec Native Ingest Projects Payloads With, So Their Admission Reads Them No Further.
    pub(crate) fn registered_codec(&self, contract: &crate::telemetry::TelemetryContract) -> Result<Arc<telemetry::TypedCodec>> {
        Ok(self.codecs.registered(contract)?)
    }

    pub(crate) async fn prepare_telemetry(
        &self, samples: &[crate::domains::telemetry::PreparedSample], contracts: &[crate::telemetry::TelemetryContract],
        payloads: &crate::domains::telemetry::Payloads,
    ) -> Result<TelemetryAdmission> {
        use crate::domains::telemetry::telemetry_slots as slots;
        let mut ids = std::collections::BTreeSet::new();
        for row in samples {
            let Some(id) = row.scope().and_then(|scope| scope.test_execution_id.as_ref()) else {
                continue;
            };
            if ids.contains(id) {
                continue;
            }
            ids.insert(id);
        }
        let mut owners = slots(ids.len())?;
        for id in ids {
            let owner = self.acquisition(id).await.ok_or_else(|| invalid("telemetry has no execution owner"))?;
            let acquisition = owner.lock_owned().await;
            if acquisition.finish_requested {
                return Err(invalid("telemetry execution admission is closed"));
            }
            if &acquisition.execution.test_execution_id != id {
                return Err(invalid("telemetry acquisition belongs to a different execution"));
            }
            owners.push(acquisition);
        }
        for row in samples {
            let Some(scope) = row.scope() else {
                continue;
            };
            let Some(id) = &scope.test_execution_id else {
                continue;
            };
            let index = owners
                .binary_search_by(|owner| owner.execution.test_execution_id.cmp(id))
                .map_err(|_| invalid("telemetry execution owner is missing"))?;
            let owner = &owners[index].execution.scope;
            if scope.session_id != owner.session_id
                || scope.test_cycle_id != owner.test_cycle_id
                || scope.test_case_id != owner.test_case_id
                || scope.test_execution_id != owner.test_execution_id
                || scope.template_id != owner.template_id
                || scope.suite_id != owner.suite_id
            {
                return Err(invalid("telemetry execution scope differs from its owner"));
            }
        }
        let mut codecs = slots(contracts.len())?;
        for contract in contracts {
            codecs.push(self.codecs.registered(contract)?);
        }
        let rows = stage_rows(&self.context_epoch, samples, &codecs, payloads)?;
        Ok(TelemetryAdmission {
            storage: self.storage.clone(),
            rows,
            _owners: owners,
        })
    }
}

/* === Public Functions === */

/// Reads Only The Explicit Durable Frontier; Accepted Queue Rows Are Never Visible.
pub(crate) async fn saved_codecs(
    session: &crate::storage::Manager, at: &harmos::storage::StorageCursor, schema: Option<&SchemaKey>,
    settings: &harmos::config::SchemaSettings, selected: impl Fn(&str, u32) -> bool + Send + Sync,
) -> Result<harmos::storage::ReadBatch<Vec<telemetry::TypedCodec>>> {
    let mut catalog = session
        .query(Query {
            at: at.clone(),
            source: Source::Schemas(harmos::storage::query::SchemaSelection { schema: schema.cloned() }),
            transforms: Vec::new(),
            slice: RowSlice::default(),
        })
        .await?;
    let mut codecs = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(Vec::<telemetry::TypedCodec>::new()))?;
    let mut announced = None;
    let mut rows = 0_u64;
    while let Some(output) = Box::pin(catalog.next()).await? {
        match output {
            QueryArrow::Schema(schema) => {
                if announced.is_some() || rows != 0 {
                    return Err(invalid("schema query emitted a repeated or late schema"));
                }
                announced = Some(schema);
            }
            QueryArrow::Batch(batch) => {
                let schema = announced.as_ref().ok_or_else(|| invalid("schema query emitted data before its schema"))?;
                batch.validate_schema(schema.as_schema())?;
                let count = u64::try_from(batch.as_record_batch().num_rows()).map_err(|_| invalid("schema query row count overflows"))?;
                rows = rows.checked_add(count).ok_or_else(|| invalid("schema query row count overflows"))?;
                let page = telemetry::stored_codecs_where(batch, settings, &selected)?;
                codecs = append_retained(codecs, page)?;
            }
        }
    }
    let completed = catalog.completion()?;
    if announced.is_none() || completed.at != *at || completed.rows != rows {
        return Err(invalid("schema query completion differs from the consumed evidence"));
    }
    codecs.try_map(|mut codecs| {
        codecs.sort_unstable_by(|left, right| {
            left.schema().id.cmp(&right.schema().id).then(left.schema().version.cmp(&right.schema().version))
        });
        codecs.dedup_by(|left, right| left.schema() == right.schema());
        Ok::<_, Error>(codecs)
    })
}

/// Reads Only The Explicit Durable Frontier; Accepted Queue Rows Are Never Visible.
pub(crate) async fn collect_saved(
    session: &crate::storage::Manager, at: &harmos::storage::StorageCursor, selection: &crate::evidence_runtime::TelemetrySelection,
    settings: &harmos::config::SchemaSettings, options: ReadOptions<'_>, local: &crate::storage::sqlite::State,
) -> Result<harmos::storage::ReadBatch<Vec<crate::telemetry::TelemetrySample>>> {
    let mut samples = Vec::new();
    visit_saved(session, at, selection, settings, options, local, |sample| {
        samples.push(sample);
        Ok(())
    })
    .await?;
    samples.sort_unstable_by_key(|sample| sample.sequence);
    if let Some(limit) = selection.limit {
        if selection.latest && samples.len() > limit {
            samples.drain(..samples.len() - limit);
        }
        samples.truncate(limit);
    }
    Ok(harmos::storage::ReadBatch::new(samples))
}

/// Reads One Page In Storage Order After `options.after`; Each Schema Stops Once It Holds The Page Limit.
pub(crate) async fn collect_page(
    session: &crate::storage::Manager, at: &harmos::storage::StorageCursor, selection: &crate::evidence_runtime::TelemetrySelection,
    settings: &harmos::config::SchemaSettings, options: ReadOptions<'_>, local: &crate::storage::sqlite::State,
) -> Result<TelemetryPage> {
    let limit = selection.limit.unwrap_or(usize::MAX);
    let mut rows = std::collections::VecDeque::new();
    let mut held = (usize::MAX, 0_usize);
    let mut limited = false;
    visit_positioned(session, at, selection, settings, options, local, |schema, position, sample| {
        if held.0 != schema {
            held = (schema, 0);
        }
        // The Latest Page Keeps A Sliding Tail Per Schema Instead Of The Whole History.
        if selection.latest && held.1 == limit {
            rows.remove(rows.len() - limit);
            held.1 -= 1;
            limited = true;
        }

        rows.push_back((position, sample));
        held.1 += 1;
        let full = !selection.latest && held.1 == limit;
        limited |= full;
        Ok(if full { std::ops::ControlFlow::Break(()) } else { std::ops::ControlFlow::Continue(()) })
    })
    .await?;
    let mut rows = Vec::from(rows);
    rows.sort_unstable_by_key(|(position, _)| *position);
    let excess = rows.len().saturating_sub(limit);
    // Schemas Under The Limit Can Still Overfill The Merged Page Together; Its Tail Is Not Yet Delivered.
    limited |= excess > 0;
    if selection.latest {
        rows.drain(..excess);
    } else {
        rows.truncate(rows.len() - excess);
    }
    let (positions, samples) = rows.into_iter().unzip();
    Ok(TelemetryPage {
        samples: harmos::storage::ReadBatch::new(samples),
        positions,
        limited,
    })
}

/// Visits The Explicit Durable Frontier In Bounded Pages.
pub(crate) async fn visit_saved(
    session: &crate::storage::Manager, at: &harmos::storage::StorageCursor, selection: &crate::evidence_runtime::TelemetrySelection,
    settings: &harmos::config::SchemaSettings, options: ReadOptions<'_>, local: &crate::storage::sqlite::State,
    mut visit: impl FnMut(crate::telemetry::TelemetrySample) -> Result<()>,
) -> Result<()> {
    visit_positioned(session, at, selection, settings, options, local, |_, _, sample| {
        visit(sample).map(|()| std::ops::ControlFlow::Continue(()))
    })
    .await
}

// Visits Selected Rows With Their Schema Index And Storage Position; A Break Ends That Schema's Scan.
async fn visit_positioned(
    session: &crate::storage::Manager, at: &harmos::storage::StorageCursor, selection: &crate::evidence_runtime::TelemetrySelection,
    settings: &harmos::config::SchemaSettings, options: ReadOptions<'_>, local: &crate::storage::sqlite::State,
    mut visit: impl FnMut(usize, u64, crate::telemetry::TelemetrySample) -> Result<std::ops::ControlFlow<()>>,
) -> Result<()> {
    let ReadOptions {
        scan_limit,
        window,
        visible,
        sequence,
        after,
    } = options;
    let codecs = saved_codecs(session, at, None, settings, |id, _| telemetry::selected_schema(id, &selection.streams)).await?;

    let mut end = None;
    if let Some((anchor, _)) = window {
        for codec in codecs.iter() {
            if !selection.schema_versions.is_empty() && !selection.schema_versions.contains(&codec.schema().version.get()) {
                continue;
            }
            if codec.schema().id.as_str().strip_prefix("dokime.telemetry.") != Some(anchor) {
                continue;
            }
            let terms = telemetry_terms(selection, visible)
                .map(|(field, value)| {
                    let value = match value {
                        TelemetryTerm::Text(value) => rusqlite::types::Value::Text(value.to_owned()),
                        TelemetryTerm::Unsigned(value) => rusqlite::types::Value::Integer(value as i64),
                    };
                    (field, value)
                })
                .collect();
            if let Some(latest) =
                local.latest_telemetry_time(at.clone(), codec.schema().clone(), terms, selection.clone(), codec.clone()).await?
            {
                let latest: time::OffsetDateTime = latest.into();
                let latest =
                    i64::try_from(latest.unix_timestamp_nanos()).map_err(|_| invalid("telemetry timestamp exceeds query range"))?;
                end = Some(end.map_or(latest, |prior: i64| prior.max(latest)));
            }
        }
    }
    if window.is_some() && end.is_none() {
        return Ok(());
    }
    let start = window
        .map(|(_, seconds)| seconds)
        .zip(end)
        .map(|(window, end)| {
            let duration = crate::plot::plot_window_duration(window).ok_or_else(|| invalid("plot window is invalid"))?;
            let end =
                time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(end)).map_err(|_| invalid("telemetry timestamp is invalid"))?;
            let start = end.checked_sub(duration).ok_or_else(|| invalid("plot window timestamp overflows"))?;
            Ok::<std::time::SystemTime, Error>(start.into())
        })
        .transpose()?;
    let mut count = 0usize;
    'schemas: for (schema, codec) in codecs.iter().enumerate() {
        if !selection.schema_versions.is_empty() && !selection.schema_versions.contains(&codec.schema().version.get()) {
            continue;
        }
        let secondary = window.is_some_and(|(anchor, _)| codec.schema().id.as_str().strip_prefix("dokime.telemetry.") != Some(anchor));
        if let Some(start) = start.filter(|_| secondary) {
            let options = ReadOptions { visible, sequence, ..Default::default() };
            telemetry_predecessors(at, codec, selection, start, options, local, |position, sample| {
                if let Some(sample) = admit_sample(sample, &mut count, scan_limit, selection)? {
                    let _ = visit(schema, position, sample)?;
                }
                Ok(())
            })
            .await?;
        }
        let mut query = telemetry_query(at, codec, selection, visible, after, 0)?;
        if let Source::Stream(source) = &mut query.source
            && start.is_some()
        {
            source.source_start = source.source_start.max(start);
            close_window(source);
        }
        if let Some(limit) = scan_limit {
            query.slice.limit = Some(limit.saturating_add(1) as u64);
        }
        // Native Reads Avoid An Arrow Encode/Decode Round Trip When No Transform Is Needed.
        if query.transforms.is_empty() {
            let Source::Stream(source) = query.source else { unreachable!("telemetry uses stream storage") };
            let mut reader = session.read_stream(harmos::storage::StreamRead { at: at.clone(), selection: source }).await?;
            while let Some(rows) = Box::pin(reader.next()).await? {
                let page = crate::storage::projection::page(rows, |row| {
                    let position = row.record.position.sequence;
                    Ok((position, sequence.sample(codec.decode_stored(&row)?, position)))
                })?;
                for (position, sample) in page.iter() {
                    let Some(sample) = admit_sample(sample.clone(), &mut count, scan_limit, selection)? else { continue };
                    if visit(schema, *position, sample)?.is_break() {
                        continue 'schemas;
                    }
                }
            }
            continue;
        }
        let mut reader = session.query(query).await?;
        while let Some(output) = Box::pin(reader.next()).await? {
            let QueryArrow::Batch(batch) = output else {
                continue;
            };
            let entries = telemetry::decode_positioned(codec, batch)?;
            let page = crate::storage::projection::page(entries, |(position, entry)| Ok((position, sequence.sample(entry, position))))?;
            for (position, sample) in page.iter() {
                let Some(sample) = admit_sample(sample.clone(), &mut count, scan_limit, selection)? else { continue };
                if visit(schema, *position, sample)?.is_break() {
                    continue 'schemas;
                }
            }
        }
        reader.completion()?;
    }
    Ok(())
}

fn admit_sample(
    sample: crate::telemetry::TelemetrySample, count: &mut usize, scan_limit: Option<usize>,
    selection: &crate::evidence_runtime::TelemetrySelection,
) -> Result<Option<crate::telemetry::TelemetrySample>> {
    *count = count.checked_add(1).ok_or_else(|| invalid("telemetry sample count overflows"))?;
    if scan_limit.is_some_and(|limit| *count > limit) {
        return Err(invalid("telemetry scan exceeds the analysis sample budget"));
    }
    let follows = selection.after_stream_cursors.is_empty() || {
        let stream = crate::telemetry_runtime::telemetry_stream_id(sample.scope.as_ref(), &sample.stream);
        selection.after_stream_cursors.get(&stream).is_none_or(|after| sample.sequence > *after)
    };
    let selected = follows
        && selection.scope.matches(sample.scope.as_ref())
        && (selection.sources.is_empty() || selection.sources.contains(&sample.source))
        && selection.signals.iter().all(|signal| sample.values.contains_key(signal));
    Ok(selected.then_some(sample))
}

pub(crate) async fn plot_query(
    at: &harmos::storage::StorageCursor, selection: &crate::evidence_runtime::TelemetrySelection, codec: &telemetry::TypedCodec,
    window: Option<(&str, f64)>, local: &crate::storage::sqlite::State,
) -> Result<Option<Query>> {
    let mut query = telemetry_query(at, codec, selection, None, 0, 0)?;
    if let Some((_, seconds)) = window {
        let terms = telemetry_terms(selection, None)
            .map(|(field, value)| {
                (
                    field,
                    match value {
                        TelemetryTerm::Text(value) => rusqlite::types::Value::Text(value.to_owned()),
                        TelemetryTerm::Unsigned(value) => rusqlite::types::Value::Integer(value as i64),
                    },
                )
            })
            .collect();
        let Some(end) = local.latest_telemetry_time(at.clone(), codec.schema().clone(), terms, selection.clone(), codec.clone()).await?
        else {
            return Ok(None);
        };
        let end: time::OffsetDateTime = end.into();
        let duration = crate::plot::plot_window_duration(seconds).ok_or_else(|| invalid("plot window is invalid"))?;
        let start = end.checked_sub(duration).ok_or_else(|| invalid("plot window timestamp overflows"))?;
        if let Source::Stream(source) = &mut query.source {
            source.source_start = source.source_start.max(Some(start.into()));
            close_window(source);
        }
    }
    Ok(Some(query))
}

pub(crate) fn append_retained<T>(
    left: harmos::storage::ReadBatch<Vec<T>>, right: harmos::storage::ReadBatch<Vec<T>>,
) -> crate::storage::Result<harmos::storage::ReadBatch<Vec<T>>> {
    let length = left.len().checked_add(right.len()).ok_or(crate::storage::Error::Invalid("telemetry collection length overflows"))?;
    if length <= left.capacity() {
        return left.try_map_with(right, |mut left, right| {
            left.extend(right);
            Ok(left)
        });
    }
    let capacity = length.checked_next_power_of_two().ok_or(crate::storage::Error::Invalid("telemetry collection capacity overflows"))?;
    left.try_map_with(right, |left, right| {
        let mut target = Vec::new();
        target.try_reserve_exact(capacity).map_err(|_| crate::storage::Error::Invalid("telemetry collection allocation failed"))?;
        target.extend(left);
        target.extend(right);
        Ok(target)
    })
}

async fn telemetry_predecessors(
    at: &harmos::storage::StorageCursor, codec: &telemetry::TypedCodec, selection: &crate::evidence_runtime::TelemetrySelection,
    start: std::time::SystemTime, options: ReadOptions<'_>, local: &crate::storage::sqlite::State,
    mut visit: impl FnMut(u64, crate::telemetry::TelemetrySample) -> Result<()>,
) -> Result<()> {
    let ReadOptions { visible, sequence, .. } = options;
    let terms = telemetry_terms(selection, visible)
        .map(|(field, value)| {
            (
                field,
                match value {
                    TelemetryTerm::Text(value) => rusqlite::types::Value::Text(value.to_owned()),
                    TelemetryTerm::Unsigned(value) => rusqlite::types::Value::Integer(value as i64),
                },
            )
        })
        .collect::<Vec<_>>();
    let mut after = String::new();
    loop {
        let (cursor, rows) = local
            .telemetry_predecessor_page(at.clone(), codec.schema().clone(), selection.clone(), terms.clone(), start, after.clone())
            .await?;
        if cursor == after {
            break;
        }
        after = cursor;
        for row in rows {
            let position = row.record.position.sequence;
            visit(position, sequence.sample(codec.decode_stored(&row)?, position))?;
        }
        tokio::task::yield_now().await;
    }
    Ok(())
}

fn telemetry_query(
    at: &harmos::storage::StorageCursor, codec: &telemetry::TypedCodec, selection: &crate::evidence_runtime::TelemetrySelection,
    visible: Option<&dokime_domain::model::execution::RecordScope>, after: u64, extra_transforms: usize,
) -> Result<Query> {
    use harmos::storage::query::transform::{
        Transform,
        filter::{Clause, Compare, Condition, Filter, Term},
    };
    let count = telemetry_terms(selection, visible).count();
    let transforms = usize::from(count != 0).checked_add(extra_transforms).ok_or_else(query_memory)?;
    let mut terms = query_slots(count)?;
    for (field, value) in telemetry_terms(selection, visible) {
        let value = match value {
            TelemetryTerm::Text(value) => harmos::storage::query::scalar::ScalarValue::Text(query_text(value)?),
            TelemetryTerm::Unsigned(value) => harmos::storage::query::scalar::ScalarValue::Unsigned(value),
        };
        terms.push(Term {
            field: query_path(field)?,
            condition: Condition::Compare { op: Compare::Eq, value },
            negate: false,
        });
    }
    let mut transforms = query_slots(transforms)?;
    if count != 0 {
        let mut clauses = query_slots(1)?;
        clauses.push(Clause { terms });
        transforms.push(Transform::Filter(Filter { clauses }));
    }
    let time = |timestamp: &dokime_domain::model::time::Timestamp| -> Result<std::time::SystemTime> {
        Ok(timestamp.to_datetime().map_err(crate::storage::Error::from)?.into())
    };
    let mut query = Query {
        at: at.clone(),
        source: Source::Stream(StreamSelection {
            schema: codec.schema().clone(),
            scope: selection
                .scope
                .test_execution_id
                .as_ref()
                .map(|id| Identifier::new(id.as_str()))
                .transpose()
                .map_err(crate::storage::Error::from)?,
            item: None,
            stream: None,
            source_start: selection.start.as_ref().map(time).transpose()?,
            source_end: selection.end.as_ref().map(time).transpose()?,
            after,
        }),
        transforms,
        slice: RowSlice::default(),
    };
    if let Source::Stream(source) = &mut query.source {
        close_window(source);
    }
    Ok(query)
}

// A Source Window Needs Both Ends; An Open Side Spans The Representable Instants.
fn close_window(source: &mut StreamSelection) {
    let epoch = std::time::SystemTime::UNIX_EPOCH;
    let span = std::time::Duration::from_nanos(i64::MAX as u64);
    if source.source_start.is_some() || source.source_end.is_some() {
        source.source_start.get_or_insert(epoch - span);
        source.source_end.get_or_insert(epoch + span);
    }
}

enum TelemetryTerm<'a> {
    Text(&'a str),
    Unsigned(u64),
}

fn telemetry_terms<'a>(
    selection: &'a crate::evidence_runtime::TelemetrySelection, visible: Option<&'a dokime_domain::model::execution::RecordScope>,
) -> impl Iterator<Item = (&'static str, TelemetryTerm<'a>)> {
    let selected = &selection.scope;
    let fields = [
        ("payload.metadata.scope.test_execution_id", selected.test_execution_id.as_ref().map(|id| id.as_str())),
        ("payload.metadata.scope.test_cycle_id", selected.test_cycle_id.as_ref().map(|id| id.as_str())),
        ("payload.metadata.scope.test_case_id", selected.test_case_id.as_ref().map(|id| id.as_str())),
        ("payload.metadata.scope.template_id", selected.template_id.as_ref().map(|id| id.as_str())),
        ("payload.metadata.scope.session_id", visible.and_then(|s| s.session_id.as_ref()).map(|id| id.as_str())),
        ("payload.metadata.scope.test_cycle_id", visible.and_then(|s| s.test_cycle_id.as_ref()).map(|id| id.as_str())),
        ("payload.metadata.scope.test_case_id", visible.and_then(|s| s.test_case_id.as_ref()).map(|id| id.as_str())),
        ("payload.metadata.scope.test_execution_id", visible.and_then(|s| s.test_execution_id.as_ref()).map(|id| id.as_str())),
        ("payload.metadata.scope.step_id", visible.and_then(|s| s.step_id.as_ref()).map(|id| id.as_str())),
        ("payload.metadata.scope.criterion_id", visible.and_then(|s| s.criterion_id.as_ref()).map(|id| id.as_str())),
        ("payload.metadata.scope.template_id", visible.and_then(|s| s.template_id.as_ref()).map(|id| id.as_str())),
        ("payload.metadata.scope.suite_id", visible.and_then(|s| s.suite_id.as_ref()).map(|id| id.as_str())),
    ];
    use dokime_domain::model::execution::EvidenceOrigin;
    let (phase, attempt) = match visible.map(|scope| &scope.origin) {
        Some(EvidenceOrigin::Initialization) => (Some(1), None),
        Some(EvidenceOrigin::Cleanup) => (Some(3), None),
        Some(EvidenceOrigin::Step { attempt_id }) => (Some(2), Some(attempt_id.as_str())),
        _ => (None, None),
    };
    fields
        .into_iter()
        .filter_map(|(field, value)| value.map(|value| (field, TelemetryTerm::Text(value))))
        .chain(phase.map(|value| ("payload.metadata.scope.origin.phase", TelemetryTerm::Unsigned(value))))
        .chain(attempt.map(|value| ("payload.metadata.scope.origin.attempt_id", TelemetryTerm::Text(value))))
}

fn query_memory() -> Error {
    harmos::Error::ResourceExhausted { resource: "telemetry query construction" }.into()
}
fn query_slots<T>(count: usize) -> Result<Vec<T>> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(count).map_err(|_| query_memory())?;
    Ok(rows)
}
fn query_text(value: &str) -> Result<String> {
    let mut result = String::new();
    result.try_reserve_exact(value.len()).map_err(|_| query_memory())?;
    result.push_str(value);
    Ok(result)
}
fn query_path(field: &str) -> Result<harmos::storage::query::transform::FieldPath> {
    let mut fields = query_slots(field.split('.').count())?;
    for part in field.split('.') {
        fields.push(query_text(part)?);
    }
    Ok(harmos::storage::query::transform::FieldPath { fields })
}

/* === Private Functions === */

// Retains Each Admission Once; Scope References Borrow Its Payload And Values.
// The Current Durable Envelope Still Materializes Each Scope At The Storage Boundary.
pub(crate) fn stage_rows(
    epoch: &dokime_domain::model::identity::SessionId, samples: &[crate::domains::telemetry::PreparedSample],
    codecs: &[Arc<telemetry::TypedCodec>], payloads: &crate::domains::telemetry::Payloads,
) -> Result<Vec<(usize, ScopedTelemetry)>> {
    let mut rows = crate::domains::telemetry::telemetry_slots(samples.len())?;
    let epoch = Arc::new(epoch.clone());
    let mut shared = std::collections::BTreeMap::new();
    for (index, row) in samples.iter().enumerate() {
        let codec = codecs.get(row.contract).ok_or_else(|| invalid("telemetry codec index is missing"))?;
        let decoded = matches!(payloads, crate::domains::telemetry::Payloads::Decoded(decoder) if Arc::ptr_eq(decoder, codec));
        if row.original() && !decoded {
            let payload =
                row.sample.storage_payload.as_deref().ok_or_else(|| invalid("typed telemetry sample has no original row payload"))?;
            codec.check_payload(payload)?;
        }
        let identity = Arc::as_ptr(&row.sample);
        let shared = shared.entry(identity).or_insert_with(|| {
            Arc::new(SharedTelemetry {
                epoch: Arc::clone(&epoch),
                sample: Arc::clone(&row.sample),
                codec: Arc::clone(codec),
            })
        });
        rows.push((
            index,
            ScopedTelemetry {
                row: Arc::clone(shared),
                scope: row.scope().cloned(),
                ordinal: row.ordinal,
            },
        ));
    }
    Ok(rows)
}

fn prepare_staged(
    epoch: &dokime_domain::model::identity::SessionId, sample: &crate::telemetry::TelemetrySample,
    scope: Option<&dokime_domain::model::execution::RecordScope>, ordinal: u64, codec: &telemetry::TypedCodec,
) -> Result<PreparedTelemetry> {
    let payload = codec.encode_staged(epoch, ordinal, sample, scope)?;
    let scope = scope.and_then(|scope| scope.test_execution_id.as_ref()).map(|id| id.as_str());
    let identifier = |value: &str| Identifier::new(value).map_err(crate::storage::Error::from);
    let at = sample.timestamp.to_datetime().map_err(|_| invalid("typed telemetry timestamp is invalid"))?;
    let seconds = u64::try_from(at.unix_timestamp()).map_err(|_| invalid("typed telemetry predates 1970"))?;
    Ok(PreparedTelemetry {
        schema: codec.schema().clone(),
        stream: identifier(sample.stream.as_str())?,
        item: identifier(sample.sample_id.as_str())?,
        source: identifier(epoch.as_str())?,
        scope: scope.map(identifier).transpose()?,
        ordinal,
        source_time: std::time::UNIX_EPOCH
            .checked_add(std::time::Duration::new(seconds, at.nanosecond()))
            .ok_or_else(|| invalid("typed telemetry timestamp overflows"))?,
        payload,
    })
}

async fn closed(closure: Option<&crate::services::telemetry::Sampling>, scope: Option<&str>) {
    match (closure, scope) {
        (Some(sampling), Some(scope)) => sampling.wait_measurement_closed(scope).await,
        _ => std::future::pending().await,
    }
}

pub(super) fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}

/* === Tests === */

#[cfg(test)]
mod admission_tests {
    use super::*;
    use harmos::{Application, Runtime, storage::StorageBackends};
    use std::{num::NonZeroUsize, sync::Arc};

    struct TestApp;
    impl Application for TestApp {
        type Lanes = crate::app::Lanes;
        type State = ();
        type Transaction = harmos::state::Never;
        type Record = harmos::state::Never;
        type Streams = ();
    }

    #[derive(Default, serde::Deserialize)]
    struct StorageConfig {
        #[serde(skip)]
        storage: Option<crate::storage::Manager>,
    }

    impl harmos::Lifecycle for TestApp {
        type Config = StorageConfig;
        type Error = harmos::Error;
        type Context = harmos::Startup<Self>;

        async fn prepare(config: Self::Config, setup: &mut harmos::Setup<Self>) -> harmos::Result<Self> {
            setup.settings().storage(config.storage.as_ref().expect("storage fixture is configured"));
            Ok(Self)
        }
        async fn initialize(&mut self, _: &Self::Context) -> harmos::Result<()> {
            Ok(())
        }
        async fn stop(&mut self, _: &Self::Context) -> harmos::Result<()> {
            Ok(())
        }
        async fn cleanup(&mut self) -> harmos::Result<()> {
            Ok(())
        }
    }

    #[harmos::message]
    struct Reading {
        value: String,
    }

    #[test]
    fn staged_payload_memory_is_independent_of_lease_scope_count() {
        use crate::domains::telemetry::{Payloads, PreparedSample};
        use dokime_domain::model::{
            execution::RecordScope,
            identity::{SessionId, TestExecutionId},
        };

        let contract = crate::telemetry::TelemetryContract::declared("fixture.shared", "Shared").typed::<Reading>();
        let codec = Arc::new(telemetry::codec(&contract, &Default::default()).unwrap());
        let epoch = SessionId::from_seed("shared.epoch");
        let sample = Arc::new(
            crate::telemetry::TelemetrySample::new(
                crate::model::identity::new_id(),
                contract.stream.clone(),
                7,
                dokime_domain::model::time::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH),
                "fixture.shared",
            )
            .typed(&Reading { value: "x".repeat(128 * 1024) }),
        );
        let payload = sample.storage_payload.as_ref().unwrap();
        let mut retained = None;

        for scopes in [0, 1, 8, 64] {
            let mut references = vec![PreparedSample {
                sample: Arc::clone(&sample),
                lease: None,
                contract: 0,
                ordinal: 1,
            }];
            for scope in 0..scopes {
                references.push(PreparedSample {
                    sample: Arc::clone(&sample),
                    lease: Some(RecordScope {
                        test_execution_id: Some(TestExecutionId::from_seed(format!("scope.{scope}"))),
                        ..Default::default()
                    }),
                    contract: 0,
                    ordinal: scope as u64 + 2,
                });
            }
            let staged = stage_rows(&epoch, &references, &[Arc::clone(&codec)], &Payloads::Unchecked).unwrap();
            assert_eq!(staged.len(), scopes + 1);
            assert!(staged.iter().all(|(_, scoped)| Arc::ptr_eq(&scoped.row, &staged[0].1.row)));
            assert!(staged.iter().all(|(_, scoped)| Arc::ptr_eq(&scoped.row.sample, &sample)));
            let unique = staged.iter().map(|(_, scoped)| Arc::as_ptr(&scoped.row)).collect::<std::collections::BTreeSet<_>>();
            assert_eq!(unique.len(), 1);
            let retained_payload = unique.len() * payload.capacity();
            assert_eq!(*retained.get_or_insert(retained_payload), retained_payload);

            // Scope Materialization Preserves Identity And The Scoped Ordinal Without Revalidating Payloads.
            for ((_, scoped), expected) in staged.into_iter().zip(&references) {
                let input = scoped.into_input().unwrap();
                let entry = codec.decode(&input.record.payload, usize::MAX).unwrap();
                assert_eq!(entry.sample.scope.as_ref(), expected.scope());
                assert_eq!(entry.ordinal, expected.ordinal);
                assert_eq!(entry.sample.storage_payload.as_deref(), Some(payload.as_slice()));
            }
        }
    }

    #[tokio::test]
    async fn seven_day_window_scan_work_is_history_independent() {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(crate::storage::Backend::primary(root.path().to_owned()).await.unwrap());
        let local = backend.local_state();
        let state = Arc::new(harmos::storage::state::file::FileBackend::new(
            root.path().to_owned(),
            harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
            crate::config::storage_state_defaults(),
        ));
        let storage = Manager::new(StorageBackends { primary: backend, state }, crate::config::storage_defaults()).unwrap();
        let runtime = Runtime::<TestApp>::launch((), StorageConfig { storage: Some(storage.clone()) }).await.unwrap();
        let contract = crate::telemetry::TelemetryContract::declared("fixture.longrun", "Long Run").typed::<Reading>();
        let settings = harmos::config::SchemaSettings::default();
        let codec = telemetry::codec(&contract, &settings).unwrap();
        storage.write_schema(codec.schema().clone(), Arc::clone(codec.graph())).await.unwrap();
        let epoch = dokime_domain::model::identity::SessionId::from_seed("longrun.epoch");
        let selection = crate::evidence_runtime::TelemetrySelection {
            streams: vec![contract.stream.clone().into()],
            ..Default::default()
        };
        let mut work = None;

        for day in 0..7_u64 {
            for start in (0..1440_u64).step_by(120) {
                let mut batch = Vec::new();
                for offset in 0..120_u64 {
                    let minute = day * 1440 + start + offset;
                    let timestamp = time::OffsetDateTime::UNIX_EPOCH + time::Duration::minutes(minute as i64);
                    let sample = crate::telemetry::TelemetrySample::new(
                        crate::model::identity::new_id(),
                        contract.stream.clone(),
                        minute,
                        dokime_domain::model::time::Timestamp::from_datetime(timestamp),
                        "fixture.longrun",
                    )
                    .typed(&Reading { value: minute.to_string() });
                    batch.push(prepare_staged(&epoch, &sample, None, minute, &codec).unwrap().into_input());
                }
                storage.write_stream(batch).await.unwrap();
            }
            let at = storage.flush().await.unwrap();
            assert_eq!(storage.status().pending_bytes, 0);
            let before = local.read_counters();
            let mut rows = Vec::new();
            let options = ReadOptions {
                window: Some(("fixture.longrun", 300.0)),
                ..Default::default()
            };
            visit_saved(&storage, &at, &selection, &settings, options, &local, |sample| {
                rows.push(sample);
                Ok(())
            })
            .await
            .unwrap();
            let after = local.read_counters();
            let scanned = after.data_scan_rows - before.data_scan_rows;
            assert_eq!(rows.len(), 6);
            assert_eq!(after.telemetry_anchor_rows - before.telemetry_anchor_rows, 1);
            assert!(scanned <= 16, "scanned {scanned} rows on day {day}");
            if let Some(previous) = work {
                assert_eq!(scanned, previous);
            }
            work = Some(scanned);
        }
        runtime.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn seven_day_follow_wakes_scan_only_new_rows() {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(crate::storage::Backend::primary(root.path().to_owned()).await.unwrap());
        let local = backend.local_state();
        let state = Arc::new(harmos::storage::state::file::FileBackend::new(
            root.path().to_owned(),
            harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
            crate::config::storage_state_defaults(),
        ));
        let storage = Manager::new(StorageBackends { primary: backend, state }, crate::config::storage_defaults()).unwrap();
        let runtime = Runtime::<TestApp>::launch((), StorageConfig { storage: Some(storage.clone()) }).await.unwrap();
        let contract = crate::telemetry::TelemetryContract::declared("fixture.follow", "Follow").typed::<Reading>();
        let settings = harmos::config::SchemaSettings::default();
        let codec = telemetry::codec(&contract, &settings).unwrap();
        storage.write_schema(codec.schema().clone(), Arc::clone(codec.graph())).await.unwrap();
        let epoch = dokime_domain::model::identity::SessionId::from_seed("follow.epoch");
        let streams = vec![contract.stream.clone().into()];
        let paged = crate::evidence_runtime::TelemetrySelection {
            streams: streams.clone(),
            limit: Some(100),
            ..Default::default()
        };
        let whole = crate::evidence_runtime::TelemetrySelection {
            streams,
            limit: Some(2_000),
            ..Default::default()
        };
        let (mut settled, mut work) = (0, None);

        // Each Daily Wake Follows From The Settled Position: Its Scan Covers That Day's Rows, Not The History Before.
        for day in 0..7_u64 {
            for start in (0..1440_u64).step_by(120) {
                let mut batch = Vec::new();
                for offset in 0..120_u64 {
                    let minute = day * 1440 + start + offset;
                    let timestamp = time::OffsetDateTime::UNIX_EPOCH + time::Duration::minutes(minute as i64);
                    let sample = crate::telemetry::TelemetrySample::new(
                        crate::model::identity::new_id(),
                        contract.stream.clone(),
                        minute,
                        dokime_domain::model::time::Timestamp::from_datetime(timestamp),
                        "fixture.follow",
                    )
                    .typed(&Reading { value: minute.to_string() });
                    batch.push(prepare_staged(&epoch, &sample, None, minute, &codec).unwrap().into_input());
                }
                storage.write_stream(batch).await.unwrap();
            }
            let at = storage.flush().await.unwrap();

            // A Limited Page Stops At The Limit And Settles Its Last Position.
            let options = ReadOptions { after: settled, ..Default::default() };
            let page = collect_page(&storage, &at, &paged, &settings, options, &local).await.unwrap();
            assert!(page.limited);
            assert_eq!(page.samples.len(), 100);
            assert_eq!(page.samples.first().map(|sample| sample.sequence), Some(day * 1440));
            let last = *page.positions.last().unwrap();
            assert!(page.positions.windows(2).all(|pair| pair[0] < pair[1]));

            let before = local.read_counters();
            let options = ReadOptions { after: last, ..Default::default() };
            let rest = collect_page(&storage, &at, &whole, &settings, options, &local).await.unwrap();
            let scanned = local.read_counters().data_scan_rows - before.data_scan_rows;
            assert!(!rest.limited);
            assert_eq!(rest.samples.len(), 1340);
            assert_eq!(rest.samples.first().map(|sample| sample.sequence), Some(day * 1440 + 100));
            assert!(scanned <= 1440, "scanned {scanned} rows on day {day}");
            if let Some(previous) = work {
                assert_eq!(scanned, previous);
            }
            work = Some(scanned);
            settled = at.sequence;
        }
        runtime.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn later_refusal_preserves_the_accepted_prefix_without_replay() {
        let root = tempfile::tempdir().unwrap();
        let backend = Arc::new(crate::storage::Backend::primary(root.path().to_owned()).await.unwrap());
        let mut settings = crate::config::storage_defaults();
        settings.request.max_pending_write_bytes = NonZeroUsize::new(128 * 1024).unwrap();
        let state = Arc::new(harmos::storage::state::file::FileBackend::new(
            root.path().to_owned(),
            harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
            crate::config::storage_state_defaults(),
        ));
        let storage = Manager::new(StorageBackends { primary: backend, state }, settings).unwrap();
        let runtime = Runtime::<TestApp>::launch((), StorageConfig { storage: Some(storage.clone()) }).await.unwrap();
        let contract = crate::telemetry::TelemetryContract::declared("partial.fixture", "Partial").typed::<Reading>();
        let codec = telemetry::codec(&contract, &Default::default()).unwrap();
        storage.write_schema(codec.schema().clone(), Arc::clone(codec.graph())).await.unwrap();
        let execution = dokime_domain::model::identity::TestExecutionId::from_seed("partial.scope");
        let mut rows = crate::domains::telemetry::telemetry_slots(2).unwrap();
        let epoch = dokime_domain::model::identity::SessionId::from_seed("partial.epoch");
        for (index, value) in ["first".to_owned(), "x".repeat(256 * 1024)].into_iter().enumerate() {
            let row = Reading { value };
            let mut sample = crate::telemetry::TelemetrySample::new(
                crate::model::identity::new_id(),
                contract.stream.clone(),
                index as u64,
                dokime_domain::model::time::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH),
                "partial.fixture",
            )
            .typed(&row);
            sample.scope = Some(dokime_domain::model::execution::RecordScope {
                test_execution_id: Some(execution.clone()),
                ..Default::default()
            });
            rows.push(crate::domains::telemetry::PreparedSample {
                sample: Arc::new(sample),
                lease: None,
                contract: 0,
                ordinal: index as u64,
            });
        }
        let rows = stage_rows(&epoch, &rows, &[Arc::new(codec)], &crate::domains::telemetry::Payloads::Unchecked).unwrap();
        let before = storage.cursor_accepted();
        let admission = TelemetryAdmission {
            storage: storage.clone(),
            rows,
            _owners: Vec::new(),
        };
        let result = admission.commit(Some(&crate::services::telemetry::Sampling::new())).await.unwrap();
        assert_eq!(&*result.accepted, &[true, false]);
        assert!(result.failure.is_some());
        assert_eq!(storage.cursor_accepted().sequence, before.sequence + 1);
        drop(result);
        storage.flush().await.unwrap();
        runtime.shutdown().await.unwrap();
    }
}
