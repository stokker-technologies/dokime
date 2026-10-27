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
        RecordInput, SchemaKey, StreamInput,
        operations::Query,
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

pub(crate) struct TelemetryAdmission {
    storage: Manager,
    rows: Vec<(usize, PreparedTelemetry)>,
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

impl TelemetryAdmission {
    pub(crate) async fn commit(mut self, closure: Option<&crate::services::telemetry::Sampling>) -> Result<CommittedTelemetry> {
        self.rows.sort_unstable_by(|left, right| left.1.scope.cmp(&right.1.scope).then(left.0.cmp(&right.0)));
        let mut accepted = crate::domains::telemetry::telemetry_slots(self.rows.len())?;
        accepted.resize(self.rows.len(), false);
        let mut rows = self.rows.into_iter();
        let mut failure = None;
        'groups: while let Some((first_index, first)) = rows.next() {
            let count = 1 + rows.as_slice().iter().take_while(|(_, row)| row.scope == first.scope).count();
            let scope = first.scope.as_ref().map(ToString::to_string);
            let mut indices = crate::domains::telemetry::telemetry_slots(count)?;
            let mut writes = crate::domains::telemetry::telemetry_slots(count)?;
            indices.push(first_index);
            writes.push(first.into_input());
            for _ in 1..count {
                let (index, row) = rows.next().ok_or_else(|| invalid("telemetry scope group is incomplete"))?;
                indices.push(index);
                writes.push(row.into_input());
            }
            let mut writes = writes.into_iter();
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

    pub(crate) async fn prepare_telemetry(
        &self, samples: &[crate::domains::telemetry::PreparedSample], contracts: &[crate::telemetry::TelemetryContract],
    ) -> Result<TelemetryAdmission> {
        use crate::domains::telemetry::telemetry_slots as slots;
        let mut ids = std::collections::BTreeSet::new();
        for row in samples {
            let Some(id) = row.sample.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()) else {
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
            let Some(scope) = &row.sample.scope else {
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
        let mut rows = slots(samples.len())?;
        for (index, row) in samples.iter().enumerate() {
            let codec = codecs.get(row.contract).ok_or_else(|| invalid("telemetry codec index is missing"))?;
            let prepared = prepare_staged(&self.context_epoch, &row.sample, row.ordinal, codec)?;
            rows.push((index, prepared));
        }
        Ok(TelemetryAdmission { storage: self.storage.clone(), rows, _owners: owners })
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
    while let Some(output) = catalog.next().await? {
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
    settings: &harmos::config::SchemaSettings, options: ReadOptions<'_>,
) -> Result<harmos::storage::ReadBatch<Vec<crate::telemetry::TelemetrySample>>> {
    let ReadOptions { scan_limit, window, visible, sequence } = options;
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
            let mut query = telemetry_query(at, codec, selection, visible, 1)?;
            telemetry_aggregate(&mut query, &[], "_storage.source_time", "last")?;
            let mut reader = session.query(query).await?;
            while let Some(output) = reader.next().await? {
                let QueryArrow::Batch(batch) = output else {
                    continue;
                };
                let values = batch
                    .as_record_batch()
                    .column_by_name("last")
                    .and_then(|column| column.as_any().downcast_ref::<arrow_array::TimestampNanosecondArray>())
                    .ok_or_else(|| invalid("telemetry maximum timestamp is absent"))?;
                use arrow_array::Array;
                for row in 0..values.len() {
                    if !values.is_null(row) {
                        end = Some(end.map_or(values.value(row), |prior: i64| prior.max(values.value(row))));
                    }
                }
            }
            reader.completion()?;
        }
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
    let mut samples =
        harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, crate::storage::Error>(Vec::<crate::telemetry::TelemetrySample>::new()))?;
    for codec in codecs.iter() {
        if !selection.schema_versions.is_empty() && !selection.schema_versions.contains(&codec.schema().version.get()) {
            continue;
        }
        let secondary = window.is_some_and(|(anchor, _)| codec.schema().id.as_str().strip_prefix("dokime.telemetry.") != Some(anchor));
        if let Some(start) = start.filter(|_| secondary) {
            let predecessors = telemetry_predecessors(session, at, codec, selection, visible, start, sequence).await?;
            let length = samples.len().checked_add(predecessors.len()).ok_or_else(|| invalid("telemetry sample count overflows"))?;
            if scan_limit.is_some_and(|limit| length > limit) {
                return Err(invalid("telemetry scan exceeds the analysis sample budget"));
            }
            samples = append_retained(samples, predecessors)?;
        }
        let mut query = telemetry_query(at, codec, selection, visible, 0)?;
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
            while let Some(rows) = reader.next().await? {
                let page = crate::storage::projection::page(rows, |row| {
                    let entry = codec.decode_stored(&row)?;
                    Ok(sequence.sample(entry, row.record.position.sequence))
                })?;
                let length = samples.len().checked_add(page.len()).ok_or_else(|| invalid("telemetry sample count overflows"))?;
                if scan_limit.is_some_and(|limit| length > limit) {
                    return Err(invalid("telemetry scan exceeds the analysis sample budget"));
                }
                samples = append_retained(samples, page)?;
            }
            continue;
        }
        let mut reader = session.query(query).await?;
        while let Some(output) = reader.next().await? {
            let QueryArrow::Batch(batch) = output else {
                continue;
            };
            let entries = telemetry::decode_positioned(codec, batch)?;
            let page = crate::storage::projection::page(entries, |(position, entry)| Ok(sequence.sample(entry, position)))?;
            let length = samples.len().checked_add(page.len()).ok_or_else(|| invalid("telemetry sample count overflows"))?;
            if scan_limit.is_some_and(|limit| length > limit) {
                return Err(invalid("telemetry scan exceeds the analysis sample budget"));
            }
            samples = append_retained(samples, page)?;
        }
        reader.completion()?;
    }
    Ok(samples.try_map(|mut samples| {
        samples.sort_unstable_by_key(|sample| sample.sequence);
        samples.retain(|sample| {
            let follows = selection.after_stream_cursors.is_empty() || {
                let stream = crate::telemetry_runtime::telemetry_stream_id(sample.scope.as_ref(), &sample.stream);
                selection.after_stream_cursors.get(&stream).is_none_or(|after| sample.sequence > *after)
            };
            follows
                && selection.scope.matches(sample.scope.as_ref())
                && (selection.sources.is_empty() || selection.sources.contains(&sample.source))
                && selection.signals.iter().all(|signal| sample.values.contains_key(signal))
        });
        if let Some(limit) = selection.limit {
            if selection.latest && samples.len() > limit {
                samples.drain(..samples.len() - limit);
            }
            samples.truncate(limit);
        }
        Ok::<_, crate::storage::Error>(samples)
    })?)
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
    session: &crate::storage::Manager, at: &harmos::storage::StorageCursor, codec: &telemetry::TypedCodec,
    selection: &crate::evidence_runtime::TelemetrySelection, visible: Option<&dokime_domain::model::execution::RecordScope>,
    start: std::time::SystemTime, sequence: Sequence,
) -> Result<harmos::storage::ReadBatch<Vec<crate::telemetry::TelemetrySample>>> {
    let mut samples = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(Vec::new()))?;
    let Some(end) = start.checked_sub(std::time::Duration::from_nanos(1)) else { return Ok(samples) };
    let mut query = telemetry_query(at, codec, selection, visible, 1)?;
    if let Source::Stream(source) = &mut query.source {
        source.source_end = Some(source.source_end.map_or(end, |prior| prior.min(end)));
        close_window(source);
        if source.source_start.zip(source.source_end).is_some_and(|(start, end)| start >= end) {
            return Ok(samples);
        }
    }
    // Exact Scope Groups Keep Attempt And Wildcard Correlations Distinct.
    let fields = [
        "payload.metadata.scope.session_id",
        "payload.metadata.scope.test_cycle_id",
        "payload.metadata.scope.test_case_id",
        "payload.metadata.scope.test_execution_id",
        "payload.metadata.scope.step_id",
        "payload.metadata.scope.criterion_id",
        "payload.metadata.scope.template_id",
        "payload.metadata.scope.suite_id",
        "payload.metadata.scope.origin.phase",
        "payload.metadata.scope.origin.attempt_id",
    ];
    telemetry_aggregate(&mut query, &fields, "_storage.source_time", "predecessor_time")?;
    let mut groups = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(Vec::<arrow_array::RecordBatch>::new()))?;
    let mut reader = session.query(query).await?;
    while let Some(output) = reader.next().await? {
        let QueryArrow::Batch(batch) = output else { continue };
        let page = batch.try_map(|batch| {
            let mut groups = query_slots(1)?;
            groups.push(batch);
            Ok::<_, Error>(groups)
        })?;
        groups = append_retained(groups, page)?;
    }
    reader.completion()?;
    // Retain Only Group Metadata And Release Its Query Slot Before Tie Selection.
    drop(reader);
    let mut positions = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(Vec::<u64>::new()))?;
    for group in groups.iter() {
        use arrow_array::Array;
        let times = group
            .column_by_name("predecessor_time")
            .and_then(|column| column.as_any().downcast_ref::<arrow_array::TimestampNanosecondArray>())
            .ok_or_else(|| invalid("telemetry predecessor timestamp is absent"))?;
        for row in 0..group.num_rows() {
            if times.is_null(row) {
                continue;
            }
            let time = time::OffsetDateTime::from_unix_timestamp_nanos(i128::from(times.value(row)))
                .map_err(|_| invalid("telemetry predecessor timestamp is invalid"))?;
            let mut query = telemetry_query(at, codec, selection, visible, 2)?;
            if let Source::Stream(source) = &mut query.source {
                // Half-Open Window Selecting Exactly The Predecessor Instant.
                let start: std::time::SystemTime = time.into();
                source.source_start = Some(start);
                // The Window Spans One Clock Tick: SystemTime Is Coarser Than A Nanosecond On Windows.
                let tick = [1, 100].into_iter().map(std::time::Duration::from_nanos).find(|tick| start + *tick > start);
                source.source_end = Some(start + tick.ok_or_else(|| invalid("telemetry predecessor window is empty"))?);
            }
            telemetry_predecessor_scope(&mut query, group, row, &fields)?;
            telemetry_aggregate(&mut query, &[], "_storage.position", "predecessor")?;
            let page = telemetry_predecessor_positions(session, query).await?;
            positions = append_retained(positions, page)?;
        }
    }
    drop(groups);
    for position in positions.iter().copied() {
        let mut query = telemetry_query(at, codec, selection, visible, 1)?;
        if let Source::Stream(source) = &mut query.source {
            source.after = position.checked_sub(1).ok_or_else(query_memory)?;
        }
        query.slice.limit = Some(1);
        telemetry_position(&mut query, position)?;
        let mut reader = session.query(query).await?;
        while let Some(output) = reader.next().await? {
            let QueryArrow::Batch(batch) = output else { continue };
            let entries = telemetry::decode_positioned(codec, batch)?;
            let page = crate::storage::projection::page(entries, |(position, entry)| Ok(sequence.sample(entry, position)))?;
            samples = append_retained(samples, page)?;
        }
        reader.completion()?;
    }
    Ok(samples)
}

async fn telemetry_predecessor_positions(session: &crate::storage::Manager, query: Query) -> Result<harmos::storage::ReadBatch<Vec<u64>>> {
    let mut positions = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(Vec::<u64>::new()))?;
    let mut reader = session.query(query).await?;
    while let Some(output) = reader.next().await? {
        let QueryArrow::Batch(batch) = output else { continue };
        let count = batch.as_record_batch().num_rows();
        let page = batch.try_map(|batch| {
            use arrow_array::Array;
            let values = batch
                .column_by_name("predecessor")
                .and_then(|column| column.as_any().downcast_ref::<arrow_array::UInt64Array>())
                .ok_or_else(|| invalid("telemetry predecessor position is absent"))?;
            let mut positions = query_slots(count)?;
            for index in 0..values.len() {
                if !values.is_null(index) {
                    positions.push(values.value(index));
                }
            }
            Ok::<_, Error>(positions)
        })?;
        positions = append_retained(positions, page)?;
    }
    reader.completion()?;
    Ok(positions)
}

fn telemetry_predecessor_scope(query: &mut Query, group: &arrow_array::RecordBatch, row: usize, fields: &[&str]) -> Result<()> {
    use harmos::storage::query::transform::{
        FieldPath, FieldPathExecution, Transform,
        filter::{Clause, Compare, Condition, Filter, Term},
    };
    let mut terms = query_slots(fields.len())?;
    for field in fields {
        // Aggregate Group Names Are Literal Columns, Unlike The Source's Nested Paths.
        let mut literal = query_slots(1)?;
        literal.push(query_text(field)?);
        let value = FieldPath { fields: literal }.value(group, row)?;
        let condition = match value {
            harmos::storage::query::scalar::ScalarValue::Missing | harmos::storage::query::scalar::ScalarValue::Null => Condition::IsNull,
            value => Condition::Compare { op: Compare::Eq, value },
        };
        terms.push(Term { field: query_path(field)?, condition, negate: false });
    }
    let mut clauses = query_slots(1)?;
    clauses.push(Clause { terms });
    if query.transforms.len() == query.transforms.capacity() {
        return Err(query_memory());
    }
    query.transforms.push(Transform::Filter(Filter { clauses }));
    Ok(())
}

fn telemetry_position(query: &mut Query, position: u64) -> Result<()> {
    use harmos::storage::query::transform::{
        Transform,
        filter::{Clause, Compare, Condition, Filter, Term},
    };
    let mut terms = query_slots(1)?;
    terms.push(Term {
        field: query_path("_storage.position")?,
        condition: Condition::Compare { op: Compare::Eq, value: harmos::storage::query::scalar::ScalarValue::Unsigned(position) },
        negate: false,
    });
    let mut clauses = query_slots(1)?;
    clauses.push(Clause { terms });
    if query.transforms.len() == query.transforms.capacity() {
        return Err(query_memory());
    }
    query.transforms.push(Transform::Filter(Filter { clauses }));
    Ok(())
}

fn telemetry_query(
    at: &harmos::storage::StorageCursor, codec: &telemetry::TypedCodec, selection: &crate::evidence_runtime::TelemetrySelection,
    visible: Option<&dokime_domain::model::execution::RecordScope>, extra_transforms: usize,
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
        terms.push(Term { field: query_path(field)?, condition: Condition::Compare { op: Compare::Eq, value }, negate: false });
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
            after: 0,
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

fn telemetry_aggregate(query: &mut Query, fields: &[&str], measure: &str, name: &str) -> Result<()> {
    use harmos::storage::query::transform::{
        Transform,
        aggregate::{Aggregate, Measure},
        project::Projection,
    };
    let mut groups = query_slots(fields.len())?;
    for field in fields {
        groups.push(Projection { field: query_path(field)?, name: query_text(field)? });
    }
    let mut measures = query_slots(1)?;
    measures.push(Measure { field: query_path(measure)?, statistic: harmos::Aggregate::Max, name: query_text(name)? });
    if query.transforms.len() == query.transforms.capacity() {
        return Err(query_memory());
    }
    query.transforms.push(Transform::Aggregate(Aggregate { groups, measures }));
    Ok(())
}

/* === Private Functions === */

fn prepare_staged(
    epoch: &dokime_domain::model::identity::SessionId, sample: &crate::telemetry::TelemetrySample, ordinal: u64,
    codec: &telemetry::TypedCodec,
) -> Result<PreparedTelemetry> {
    let payload = codec.encode_staged(epoch, ordinal, sample)?;
    let scope = sample.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()).map(|id| id.as_str());
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
        let backend = Arc::new(crate::storage::Backend::primary(root.path().to_owned()).await.unwrap());
        let mut settings = crate::config::storage_defaults();
        settings.request.max_pending_write_bytes = NonZeroUsize::new(128 * 1024).unwrap();
        let state = Arc::new(harmos::storage::state::file::FileBackend::new(
            root.path().to_owned(),
            harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
            crate::config::storage_state_defaults(),
        ));
        let storage = Manager::new(StorageBackends { primary: backend, state }, settings).unwrap();
        let runtime = Runtime::<TestApp>::builder((), ()).runtime_with_storage(&storage).await.unwrap();
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
            sample.scope =
                Some(dokime_domain::model::execution::RecordScope { test_execution_id: Some(execution.clone()), ..Default::default() });
            let row = prepare_staged(&epoch, &sample, index as u64, &codec).unwrap();
            rows.push((index, row));
        }
        let before = storage.cursor_accepted();
        let admission = TelemetryAdmission { storage: storage.clone(), rows, _owners: Vec::new() };
        let result = admission.commit(Some(&crate::services::telemetry::Sampling::new())).await.unwrap();
        assert_eq!(&*result.accepted, &[true, false]);
        assert!(result.failure.is_some());
        assert_eq!(storage.cursor_accepted().sequence, before.sequence + 1);
        drop(result);
        storage.flush().await.unwrap();
        runtime.stop().await.unwrap();
    }
}
