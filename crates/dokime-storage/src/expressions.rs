/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Projected Expression Evaluation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

// Third Party Imports
use arrow_array::{Array, ArrayRef, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray, StructArray, UInt64Array};
use arrow_schema::SchemaRef;
use arrow_schema::{DataType, Field, Schema};
use arrow_select::filter::filter_record_batch;
use harmos_domain::Identifier;
use harmos_domain::model::storage::query::{Source as QuerySource, StreamSelection, TimeMerge, TimeMergeMode};
use harmos_domain::model::storage::{PayloadEncoding, ReadLimits, SchemaKey, StorageCursor, StoreHistory};
use harmos_query::TableReader;
use harmos_query::batches::{BatchPage, BatchReader, QueryFuture};
use harmos_query::merge::TimeMergeReader;
use harmos_query::{
    Accumulator, Table, TableWriter,
    transform::{
        FieldPath, FieldPathExecution, Transform,
        sort::{Sort, SortKey},
    },
};
use prost::Message;
use prost_reflect::DescriptorPool;
use rusqlite::params;
use std::num::NonZeroU32;
use std::task::{Context, Poll, Waker};

// Local Imports
use crate::{
    BatchSelection, Error, ExecutionCut, NumericWindow, Result, Sample, Store,
    scalar::{self, Agg, ScalarValue},
    telemetry::codec::Codec,
};
use dokime_protocol::host::evidence::RecordsAppendRequest;
use dokime_protocol::model::Aggregate;
use dokime_protocol::model::DataExpression;
use dokime_protocol::model::FieldReference;
use dokime_protocol::model::FilterOperator;
use dokime_protocol::model::PlotFilter;
use dokime_protocol::model::RecordContracts;

use dokime_protocol::model::data_expression;
use dokime_protocol::model::origin;
use dokime_protocol::model::transform_expression;
use harmos_protocol::model::Declarations;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Source = (bool, String);

/* === Structs === */

#[derive(Clone)]
pub(crate) struct Projection {
    pub cut: ExecutionCut,
    pub stream: String,
    pub fields: BTreeMap<String, DataExpression>,
    pub filters: Vec<PlotFilter>,
    pub grouping: Option<DataExpression>,
    pub group: Option<ScalarValue>,
}

pub(crate) struct Evaluation<'a> {
    store: &'a Store,
    cut: &'a ExecutionCut,
    source: Source,
    attached: BTreeMap<Source, String>,
    intervals: BTreeMap<(String, String), String>,
    aggregates: RefCell<BTreeMap<Vec<u8>, ScalarValue>>,
}

struct RetainedTable {
    table: Table,
    reader: TableReader,
    at: StorageCursor,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub(crate) fn constant_expression(&self, cut: &ExecutionCut, expression: &DataExpression) -> Result<ScalarValue> {
        let evaluation = Evaluation {
            store: self,
            cut,
            source: (false, String::new()),
            attached: BTreeMap::new(),
            intervals: BTreeMap::new(),
            aggregates: RefCell::new(BTreeMap::new()),
        };
        let batch = RecordBatch::try_from_iter(vec![("constant", Arc::new(UInt64Array::from(vec![0])) as ArrayRef)])?;
        evaluation.value(expression, &batch, 0, 0)
    }
    pub(crate) fn evaluated_batches(
        &self, window: NumericWindow<'_>, fields: &[String], mut read: impl FnMut(RecordBatch) -> Result<bool>,
    ) -> Result<()> {
        let projection = self.projection.lock().map_err(|_| Error::Data("restart after the plot projection lock failed"))?.clone();
        let Some(plan) = projection.filter(|p| p.cut.execution.identifier == window.execution && p.stream == window.stream) else {
            return self.read_batches(BatchSelection::from(window), fields, read);
        };
        if plan.filters.is_empty() && plan.group.is_none() && !fields.iter().any(|field| plan.fields.contains_key(field)) {
            return self.read_batches(BatchSelection::from(window), fields, read);
        }
        let expressions = fields
            .iter()
            .map(|field| plan.fields.get(field).cloned().unwrap_or_else(|| reference(window.stream, field, false)))
            .collect::<Vec<_>>();
        let mut inputs = expressions.clone();
        inputs.extend(plan.filters.iter().filter_map(|f| f.expression.clone()));
        inputs.extend(plan.grouping.clone());
        self.evaluate(&plan.cut, (false, window.stream.into()), BatchSelection::from(window), &inputs, |evaluation, batch| {
            let mut columns = Vec::new();
            for (field, expression) in fields.iter().zip(&expressions) {
                let values = (0..batch.num_rows()).map(|row| evaluation.value(expression, &batch, row, 0)).collect::<Result<Vec<_>>>()?;
                columns.push((Arc::new(Field::new(field, values_type(&values), true)), values_array(&values)?));
            }
            let mut keep = Vec::with_capacity(batch.num_rows());
            for row in 0..batch.num_rows() {
                let mut selected = true;
                for filter in &plan.filters {
                    selected &= evaluation.filter(filter, &batch, row)?;
                }
                if let (Some(grouping), Some(group)) = (&plan.grouping, &plan.group) {
                    selected &=
                        evaluation.value(grouping, &batch, row, 0)?.compare(group).map_err(|e| Error::Invalid(e.to_string()))?.is_eq();
                }
                keep.push(Some(selected));
            }
            let values =
                if columns.is_empty() { StructArray::new_empty_fields(batch.num_rows(), None) } else { StructArray::from(columns) };
            let output = RecordBatch::try_from_iter(vec![("_storage", batch.column(0).clone()), ("values", Arc::new(values) as ArrayRef)])?;
            let output = filter_record_batch(&output, &BooleanArray::from(keep))?;
            if output.num_rows() == 0 { Ok(true) } else { read(output) }
        })
    }

    pub(crate) fn evaluate(
        &self, cut: &ExecutionCut, source: Source, selection: BatchSelection<'_>, expressions: &[DataExpression],
        mut read: impl FnMut(&Evaluation<'_>, RecordBatch) -> Result<bool>,
    ) -> Result<()> {
        let mut sources = BTreeMap::<Source, BTreeSet<String>>::new();
        sources.entry(source.clone()).or_default();
        for expression in expressions {
            collect(expression, &mut sources, 0)?;
        }
        let fields = sources.remove(&source).unwrap_or_default().into_iter().collect::<Vec<_>>();
        let mut evaluation = Evaluation {
            store: self,
            cut,
            source: source.clone(),
            attached: BTreeMap::new(),
            intervals: BTreeMap::new(),
            aggregates: RefCell::new(BTreeMap::new()),
        };
        let mut intervals = BTreeSet::new();
        for expression in expressions {
            collect_intervals(expression, &mut intervals);
        }
        if sources.is_empty() && intervals.is_empty() {
            return self.source_batches(cut, &source, selection, &fields, |batch| read(&evaluation, batch));
        }
        let mut base = self.source_table(cut, &source, selection, &fields)?;
        for (other, fields) in sources {
            let right = self.source_table(
                cut,
                &other,
                BatchSelection {
                    after: 0,
                    start_ns: i64::MIN,
                    ..selection
                },
                &fields.into_iter().collect::<Vec<_>>(),
            )?;
            let name = format!("_joined{}", evaluation.attached.len());
            base = align(self, cut, &base, &right, &name)?;
            evaluation.attached.insert(other, name);
        }
        for (current, previous) in intervals {
            let current_table = self.source_table(
                cut,
                &(true, current.clone()),
                BatchSelection {
                    after: 0,
                    start_ns: i64::MIN,
                    ..selection
                },
                &[],
            )?;
            let previous_table = self.source_table(
                cut,
                &(true, previous.clone()),
                BatchSelection {
                    after: 0,
                    start_ns: i64::MIN,
                    ..selection
                },
                &[],
            )?;
            let paired = align(self, cut, &current_table, &previous_table, "_previous")?;
            let value_fields = arrow_schema::Fields::from(vec![Field::new("elapsed", DataType::Float64, true)]);
            let schema = Arc::new(Schema::new(vec![
                current_table.schema().field(0).clone(),
                Field::new("values", DataType::Struct(value_fields.clone()), false),
            ]));
            let mut output = TableWriter::new(schema.clone())?;
            let mut reader = paired.reader()?;
            while let Some(batch) = reader.next(&mut || Ok(()))? {
                let current_times = (FieldPath {
                    fields: vec!["_storage".into(), "source_ns".into()],
                })
                .array(&batch)?;
                let current_times = current_times.as_any().downcast_ref::<Int64Array>().unwrap();
                let previous_times = FieldPath {
                    fields: vec!["_previous".into(), "_storage".into(), "source_ns".into()],
                };
                let values = (0..batch.num_rows())
                    .map(|row| {
                        Ok(match previous_times.value(&batch, row)? {
                            ScalarValue::Signed(previous) => Some((current_times.value(row) as i128 - previous as i128) as f64 / 1e9),
                            _ => None,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let values = StructArray::new(value_fields.clone(), vec![Arc::new(Float64Array::from(values))], None);
                output.push(&RecordBatch::try_new(schema.clone(), vec![batch.column(0).clone(), Arc::new(values)])?, &mut || Ok(()))?;
            }
            let name = format!("_interval{}", evaluation.intervals.len());
            base = align(self, cut, &base, &output.finish()?, &name)?;
            evaluation.intervals.insert((current, previous), name);
        }
        let mut reader = base.reader()?;
        while let Some(batch) = reader.next(&mut || Ok(()))? {
            self.check_read()?;
            if !read(&evaluation, batch)? {
                break;
            }
        }
        Ok(())
    }

    fn source_table(&self, cut: &ExecutionCut, source: &Source, selection: BatchSelection<'_>, fields: &[String]) -> Result<Table> {
        let mut writer = None;
        self.source_batches(cut, source, selection, fields, |batch| {
            if writer.is_none() {
                writer = Some(TableWriter::new(batch.schema())?);
            }
            writer.as_mut().unwrap().push(&batch, &mut || Ok(()))?;
            Ok(true)
        })?;
        let table = match writer {
            Some(writer) => writer.finish()?,
            None => {
                let codec = self.source_codec(cut, source)?;
                TableWriter::new(codec.batch(&[])?.schema())?.finish()?
            }
        };
        let keys = ["source_ns", "position"]
            .iter()
            .map(|name| SortKey {
                field: FieldPath {
                    fields: vec!["_storage".into(), (*name).into()],
                },
                descending: false,
                nulls_first: false,
            })
            .collect();
        Ok(table.transform(&Transform::Sort(Sort { keys }), &mut || Ok(()))?)
    }
    pub(crate) fn source_codec(&self, cut: &ExecutionCut, source: &Source) -> Result<Codec> {
        if !source.0 {
            let connection = self.lock()?;
            let codec = self.telemetry.lock().map_err(|_| Error::Data("restart after the stream descriptor lock failed"))?.codec(
                &connection,
                &cut.execution.identifier,
                &source.1,
            )?;
            return Ok(codec.as_ref().clone());
        }
        let catalog = Declarations::decode(cut.execution.catalog.as_slice())?;
        let message = if source.0 {
            let contracts = catalog
                .sections
                .iter()
                .filter(|s| s.r#type == "dokime.v1.RecordContracts")
                .map(|s| RecordContracts::decode(s.payload.as_slice()))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            contracts
                .into_iter()
                .flat_map(|c| c.records)
                .find(|c| c.metadata.as_ref().is_some_and(|m| m.identifier == source.1))
                .map(|c| c.message)
        } else {
            catalog.streams.iter().find(|s| s.id == source.1).map(|s| s.row.clone())
        }
        .ok_or_else(|| Error::Invalid(format!("Query Evidence: Declare Source {}", source.1)))?;
        let pool = DescriptorPool::from_file_descriptor_set(catalog.types.ok_or(Error::Data("restore frozen descriptors"))?)
            .map_err(|_| Error::Data("restore valid frozen descriptors"))?;
        Codec::new(pool.get_message_by_name(&message).ok_or(Error::Data("restore the source row descriptor"))?)
    }
    fn source_batches(
        &self, cut: &ExecutionCut, source: &Source, selection: BatchSelection<'_>, fields: &[String],
        mut read: impl FnMut(RecordBatch) -> Result<bool>,
    ) -> Result<()> {
        let BatchSelection {
            after,
            through,
            start_ns: start,
            end_ns: end,
            ..
        } = selection;
        if !source.0 {
            return self.read_batches(
                BatchSelection {
                    execution: &cut.execution.identifier,
                    stream: &source.1,
                    after,
                    through,
                    start_ns: start,
                    end_ns: end,
                },
                fields,
                read,
            );
        }
        let codec = self.source_codec(cut, source)?;
        let mut cursor = after;
        loop {
            let bytes = {
                let connection = self.lock()?;
                let mut query=connection.prepare("SELECT payload FROM evidence WHERE execution=?1 AND position>?2 AND position<=?3 AND kind=?4 ORDER BY position LIMIT ?5")?;
                query
                    .query_map(
                        params![
                            cut.execution.identifier,
                            cursor as i64,
                            through.min(i64::MAX as u64) as i64,
                            format!("record:{}", source.1),
                            self.storage_settings()?.flush_rows.get() as i64
                        ],
                        |row| row.get::<_, Vec<u8>>(0),
                    )?
                    .collect::<std::result::Result<Vec<_>, _>>()?
            };
            if bytes.is_empty() {
                break;
            }
            let mut rows = Vec::new();
            for bytes in bytes {
                let evidence = crate::Evidence::decode(bytes.as_slice())?;
                cursor = evidence.position;
                if evidence.source_ns < start || evidence.source_ns > end {
                    continue;
                }
                let record = RecordsAppendRequest::decode(evidence.payload.as_slice())?;
                rows.push(Sample {
                    position: evidence.position,
                    source_ns: evidence.source_ns,
                    receipt_ns: evidence.receipt_ns,
                    sequence: 0,
                    stream: source.1.clone(),
                    row: record.payload,
                });
            }
            if !rows.is_empty() && !read(codec.batch(&rows)?)? {
                break;
            }
        }
        Ok(())
    }
}
impl Evaluation<'_> {
    pub(crate) fn value(&self, expression: &DataExpression, batch: &RecordBatch, row: usize, depth: usize) -> Result<ScalarValue> {
        if depth > 32 {
            return Err(Error::Data("simplify expression nesting"));
        }
        use dokime_protocol::model::data_expression::Kind;
        Ok(match expression.kind.as_ref() {
            Some(Kind::Stream(reference)) => self.field(reference, false, batch, row)?,
            Some(Kind::Record(reference)) => self.field(reference, true, batch, row)?,
            Some(Kind::Literal(value)) => scalar::from_wire(value)?,
            Some(Kind::Parameter(key) | Kind::Dimension(key)) => {
                let Some(origin::Kind::PluginRun(run)) = self.cut.execution.origin.as_ref().and_then(|o| o.kind.as_ref()) else {
                    return Err(Error::Data("select parameters from a PluginRun origin"));
                };
                scalar::from_wire(
                    run.test_case.as_ref().and_then(|c| c.parameters.get(key)).ok_or(Error::Data("declare the selected parameter"))?,
                )?
            }
            Some(Kind::Transform(transform)) => {
                let input = transform.input.as_deref().ok_or(Error::Data("supply the transform input"))?;
                match transform.kind.as_ref() {
                    Some(transform_expression::Kind::Detail(field)) => {
                        let mut input = input.clone();
                        let reference = match input.kind.as_mut() {
                            Some(Kind::Stream(r) | Kind::Record(r)) => r,
                            _ => return Err(Error::Data("select a message field for detail")),
                        };
                        reference.field = if reference.field.is_empty() { field.clone() } else { format!("{}.{}", reference.field, field) };
                        self.value(&input, batch, row, depth + 1)?
                    }
                    Some(transform_expression::Kind::Scale(factor)) => ScalarValue::Float(
                        scalar::number(&self.value(input, batch, row, depth + 1)?).ok_or(Error::Data("scale a numeric value"))? * factor,
                    ),
                    Some(transform_expression::Kind::Offset(offset)) => ScalarValue::Float(
                        scalar::number(&self.value(input, batch, row, depth + 1)?).ok_or(Error::Data("offset a numeric value"))? + offset,
                    ),
                    Some(transform_expression::Kind::PerCase(true)) => self.value(input, batch, row, depth + 1)?,
                    Some(transform_expression::Kind::TimeBetween(previous)) => {
                        let Some(Kind::Record(current)) = input.kind.as_ref() else {
                            return Err(Error::Data("select record occurrences for time between"));
                        };
                        let name = self
                            .intervals
                            .get(&(current.identifier.clone(), previous.clone()))
                            .ok_or(Error::Data("bind the record interval"))?;
                        (FieldPath {
                            fields: vec![name.clone(), "values".into(), "elapsed".into()],
                        })
                        .value(batch, row)?
                    }
                    _ => return Err(Error::Data("select a declared transform")),
                }
            }
            Some(Kind::Aggregate(aggregate)) => {
                let input = aggregate.input.as_deref().ok_or(Error::Data("supply the aggregate input"))?;
                let key = expression.encode_to_vec();
                if let Some(value) = self.aggregates.borrow().get(&key) {
                    return Ok(value.clone());
                }
                let value = self.store.reduce_expression(
                    self.cut,
                    input,
                    &[],
                    Aggregate::try_from(aggregate.kind).map_err(|_| Error::Data("select a declared reducer"))?,
                    aggregate.percentile,
                )?;
                self.aggregates.borrow_mut().insert(key, value.clone());
                value
            }
            _ => return Err(Error::Data("select an expression")),
        })
    }
    fn field(&self, reference: &FieldReference, record: bool, batch: &RecordBatch, row: usize) -> Result<ScalarValue> {
        let source = (record, reference.identifier.clone());
        let field = if reference.field.is_empty() {
            let codec = self.store.source_codec(self.cut, &source)?;
            let fields = codec.descriptor.fields().collect::<Vec<_>>();
            if fields.len() != 1 {
                return Err(Error::Data("name a scalar field in this row"));
            }
            fields[0].name().to_owned()
        } else {
            reference.field.clone()
        };
        let mut path = if source == self.source {
            Vec::new()
        } else {
            vec![self.attached.get(&source).ok_or(Error::Data("bind the expression source"))?.clone()]
        };
        path.push("values".into());
        path.extend(field.split('.').map(str::to_owned));
        Ok(FieldPath { fields: path }.value(batch, row)?)
    }
    pub(crate) fn filter(&self, filter: &PlotFilter, batch: &RecordBatch, row: usize) -> Result<bool> {
        let expression = filter.expression.as_ref().ok_or(Error::Data("supply the filter expression"))?;
        let left = self.value(expression, batch, row, 0)?;
        let operator = FilterOperator::try_from(filter.operator).map_err(|_| Error::Data("select a comparison"))?;
        if operator == FilterOperator::Exists {
            return Ok(!matches!(left, ScalarValue::Missing | ScalarValue::Null));
        }
        if matches!(left, ScalarValue::Missing | ScalarValue::Null) {
            return Ok(false);
        }
        let right = scalar::from_wire(filter.value.as_ref().ok_or(Error::Data("supply the comparison value"))?)?;
        if operator == FilterOperator::Contains {
            return Ok(scalar::text(&left).contains(&scalar::text(&right)));
        }
        let order = left.compare(&right).map_err(|e| Error::Invalid(e.to_string()))?;
        Ok(match operator {
            FilterOperator::Eq => order.is_eq(),
            FilterOperator::Ne => !order.is_eq(),
            FilterOperator::Gt => order.is_gt(),
            FilterOperator::Ge => !order.is_lt(),
            FilterOperator::Lt => order.is_lt(),
            FilterOperator::Le => !order.is_gt(),
            _ => false,
        })
    }
}
impl<'a> From<NumericWindow<'a>> for BatchSelection<'a> {
    fn from(window: NumericWindow<'a>) -> Self {
        Self {
            execution: window.execution,
            stream: window.stream,
            after: window.after,
            through: window.through,
            start_ns: window.start_ns,
            end_ns: window.end_ns,
        }
    }
}

impl BatchReader for RetainedTable {
    fn schema(&self) -> SchemaRef {
        self.table.schema()
    }
    fn at(&self) -> &StorageCursor {
        &self.at
    }
    fn next(&mut self) -> QueryFuture<'_, BatchPage> {
        Box::pin(async move {
            let batch = self.reader.next(&mut || Ok(()))?;
            let complete = batch.is_none();
            let batch = batch.unwrap_or_else(|| RecordBatch::new_empty(self.table.schema()));
            Ok(BatchPage { batch, complete })
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn align(store: &Store, cut: &ExecutionCut, base: &Table, right: &Table, name: &str) -> Result<Table> {
    let settings = store.storage_settings()?;
    let history = StoreHistory {
        store: Identifier::new(store.identity()).map_err(|error| Error::Invalid(error.to_string()))?,
        epoch: Identifier::new(&cut.execution.identifier).map_err(|error| Error::Invalid(error.to_string()))?,
    };
    let at = StorageCursor { history, sequence: cut.frontier };
    let left = RetainedTable {
        table: base.clone(),
        reader: base.reader()?,
        at: at.clone(),
    };
    let right_input = RetainedTable {
        table: right.clone(),
        reader: right.reader()?,
        at,
    };
    let schema = SchemaKey {
        id: Identifier::new(name).map_err(|error| Error::Invalid(error.to_string()))?,
        version: NonZeroU32::MIN,
        payload_encoding: PayloadEncoding::PostcardV1,
    };
    let plan = TimeMerge {
        source: QuerySource::Stream(StreamSelection {
            schema,
            scope: None,
            item: None,
            stream: None,
            source_start: None,
            source_end: None,
            after: 0,
        }),
        primary_time: FieldPath::from("_storage.source_ns"),
        secondary_time: FieldPath::from("_storage.source_ns"),
        mode: TimeMergeMode::AtOrBefore,
        work: ReadLimits {
            rows: settings.flush_rows,
            bytes: settings.query_bytes,
        },
    };
    let mut merged = TimeMergeReader::new(Box::new(left), Box::new(right_input), plan, settings.flush_rows)?;
    let mut fields = base.schema().fields().to_vec();
    fields.push(Arc::new(Field::new(name, DataType::Struct(right.schema().fields().clone()), true)));
    let output_schema = Arc::new(Schema::new(fields));
    let mut output = TableWriter::new(output_schema.clone())?;
    loop {
        store.check_read()?;
        // Both Retained Table Readers Complete Their File Read Within One Poll.
        let mut next = merged.next();
        let mut context = Context::from_waker(Waker::noop());
        let page = match next.as_mut().poll(&mut context) {
            Poll::Ready(page) => page?,
            Poll::Pending => return Err(Error::Data("Merge Retained Tables: The Local Reader Must Complete Its Read")),
        };
        let left = page.batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("merged primary is not a struct"))?;
        let mut columns = left.columns().to_vec();
        columns.push(page.batch.column(1).clone());
        let joined = RecordBatch::try_new(output_schema.clone(), columns)?;
        output.push(&joined, &mut || Ok(()))?;
        if page.complete {
            break;
        }
    }
    Ok(output.finish()?)
}
fn collect_intervals(expression: &DataExpression, pairs: &mut BTreeSet<(String, String)>) {
    match expression.kind.as_ref() {
        Some(data_expression::Kind::Transform(t)) => {
            if let Some(input) = t.input.as_deref() {
                if let (Some(transform_expression::Kind::TimeBetween(previous)), Some(data_expression::Kind::Record(current))) =
                    (&t.kind, &input.kind)
                {
                    pairs.insert((current.identifier.clone(), previous.clone()));
                }
                collect_intervals(input, pairs);
            }
        }
        Some(data_expression::Kind::Aggregate(a)) => {
            if let Some(input) = a.input.as_deref() {
                collect_intervals(input, pairs);
            }
        }
        _ => (),
    }
}
pub(crate) fn reference(source: &str, field: &str, record: bool) -> DataExpression {
    let reference = FieldReference {
        identifier: source.into(),
        field: field.into(),
    };
    DataExpression {
        kind: Some(if record { data_expression::Kind::Record(reference) } else { data_expression::Kind::Stream(reference) }),
        ..Default::default()
    }
}
fn collect(expression: &DataExpression, sources: &mut BTreeMap<Source, BTreeSet<String>>, depth: usize) -> Result<()> {
    if depth > 32 {
        return Err(Error::Data("simplify expression nesting"));
    }
    use dokime_protocol::model::data_expression::Kind;
    match expression.kind.as_ref() {
        Some(Kind::Stream(r)) => {
            sources.entry((false, r.identifier.clone())).or_default().insert(r.field.clone());
        }
        Some(Kind::Record(r)) => {
            sources.entry((true, r.identifier.clone())).or_default().insert(r.field.clone());
        }
        Some(Kind::Transform(t)) => {
            if let Some(transform_expression::Kind::TimeBetween(previous)) = &t.kind {
                sources.entry((true, previous.clone())).or_default();
            }
            if let Some(input) = t.input.as_deref() {
                collect(input, sources, depth + 1)?;
            }
        }
        Some(Kind::Aggregate(_)) => (),
        _ => (),
    }
    Ok(())
}
pub(crate) fn values_type(values: &[ScalarValue]) -> DataType {
    match values.iter().find(|v| !matches!(v, ScalarValue::Missing | ScalarValue::Null)) {
        Some(ScalarValue::Text(_)) => DataType::Utf8,
        Some(ScalarValue::Bool(_)) => DataType::Boolean,
        Some(ScalarValue::Signed(_)) => DataType::Int64,
        Some(ScalarValue::Unsigned(_)) => DataType::UInt64,
        _ => DataType::Float64,
    }
}
pub(crate) fn values_array(values: &[ScalarValue]) -> Result<ArrayRef> {
    Ok(match values_type(values) {
        DataType::Utf8 => Arc::new(StringArray::from(
            values.iter().map(|v| if let ScalarValue::Text(v) = v { Some(v.as_str()) } else { None }).collect::<Vec<_>>(),
        )),
        DataType::Boolean => Arc::new(BooleanArray::from(
            values.iter().map(|v| if let ScalarValue::Bool(v) = v { Some(*v) } else { None }).collect::<Vec<_>>(),
        )),
        DataType::Int64 => Arc::new(Int64Array::from(
            values.iter().map(|v| if let ScalarValue::Signed(v) = v { Some(*v) } else { None }).collect::<Vec<_>>(),
        )),
        DataType::UInt64 => Arc::new(UInt64Array::from(
            values.iter().map(|v| if let ScalarValue::Unsigned(v) = v { Some(*v) } else { None }).collect::<Vec<_>>(),
        )),
        _ => Arc::new(Float64Array::from(values.iter().map(scalar::number).collect::<Vec<_>>())),
    })
}
pub(crate) fn accumulator(kind: Aggregate, percentile: f64) -> Result<Accumulator> {
    use harmos_storage::Aggregate as EngineAggregate;
    let kind = match kind {
        Aggregate::Count => EngineAggregate::Count,
        Aggregate::Sum => EngineAggregate::Sum,
        Aggregate::Min => EngineAggregate::Min,
        Aggregate::Max => EngineAggregate::Max,
        Aggregate::Mean => EngineAggregate::Mean,
        Aggregate::Stddev => EngineAggregate::Std,
        Aggregate::Rms => EngineAggregate::Rms,
        Aggregate::Median => EngineAggregate::Median,
        Aggregate::Percentile if percentile.is_finite() && (0.0..=100.0).contains(&percentile) && percentile.fract() == 0.0 => {
            EngineAggregate::Percentile(percentile as u8)
        }
        _ => return Err(Error::Data("select a supported aggregate and an integer percentile from 0 to 100")),
    };
    Ok(Accumulator::new(Agg::Of(kind)))
}

/* === Tests === */
