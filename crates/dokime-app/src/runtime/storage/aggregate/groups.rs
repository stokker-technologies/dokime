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
use std::{cmp::Ordering, collections::BTreeMap, num::NonZeroUsize};

// Third Party Imports
use harmos::storage::reduce::{Accumulator, Agg, ScalarValue, Scratch};
use harmos::{Aggregate, MemorySize};

// Local Imports
use super::fields::{self, Row};
use crate::runtime::query::aggregate::{
    AGGREGATE_ROW_CEILING, AggregateOperator, AggregateOutcome, AggregateQuery,
};
use crate::{Error, Result};
use dokime::core::Value;

/* === Constants === */
const MAX_COLUMNS: usize = 32;
const MAX_FILTERS: usize = 64;
const MAX_FILTER_VALUES: usize = 1024;
const MAX_NAME_BYTES: usize = 256;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Groups {
    entries: BTreeMap<Key, Vec<Accumulator>>,
    omitted: bool,
    // Value slots the order statistics have reserved across every page so far.
    retained: usize,
}
struct Key(Vec<Value>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Groups {
    pub(super) fn update(
        current: Option<&Self>,
        query: &AggregateQuery,
        rows: &[Row<'_>],
        allowance: NonZeroUsize,
    ) -> Result<Self> {
        validate(query)?;
        let bound = update_memory(current, query, rows)?;
        admitted(bound, allowance)?;
        let mut selected = match current {
            Some(current) => current.copy()?,
            None => Self {
                entries: BTreeMap::new(),
                omitted: false,
                retained: 0,
            },
        };
        // Median And Percentiles Retain Their Values; The Allowance Bounds How Many.
        let budget = (allowance.get() / std::mem::size_of::<ScalarValue>())
            .saturating_sub(selected.retained);
        let mut scratch = Scratch::new(budget);
        if query.group_by.is_empty() && selected.entries.is_empty() {
            selected.entries.insert(Key(Vec::new()), reductions(query)?);
        }
        for (ordinal, row) in rows.iter().enumerate() {
            if row.source() != query.source {
                return Err(invalid("aggregate row source differs from its query"));
            }
            if !matches(row, query, allowance)? {
                continue;
            }
            let mut key = slots(query.group_by.len())?;
            for column in &query.group_by {
                key.push(row.field(column)?.project(allowance)?);
            }
            let key = Key(key);
            if !selected.entries.contains_key(&key) {
                selected.entries.insert(key.copy()?, reductions(query)?);
            }
            let reductions = selected
                .entries
                .get_mut(&key)
                .expect("original group admitted");
            for (reduction, column) in reductions.iter_mut().zip(&query.aggregates) {
                let value = if column.column.is_empty() {
                    ScalarValue::Unsigned(1)
                } else {
                    scalar(
                        &row.field(&column.column)?.project(allowance)?,
                        column.aggregate,
                    )?
                };
                reduction.push(ordinal as u64, &value, &mut scratch)?;
            }
            // Retain The First Limit+1 Groups, Including Exact Aggregates For Each.
            // An Evicted Larger Key Can Never Become A Final Smaller Key.
            if selected.entries.len() > effective_limit(query.limit) + 1 {
                selected.entries.pop_last();
                selected.omitted = true;
            }
        }
        selected.retained = selected
            .retained
            .saturating_add(budget - scratch.remaining());
        admitted(selected.memory_size(), allowance)?;
        Ok(selected)
    }

    pub(super) fn outcome(
        &self,
        query: &AggregateQuery,
        allowance: NonZeroUsize,
    ) -> Result<AggregateOutcome> {
        validate(query)?;
        let count = self.entries.len().min(effective_limit(query.limit));
        let columns = query.group_by.len() + query.aggregates.len();
        let mut bytes = std::mem::size_of::<AggregateOutcome>()
            .saturating_add(columns.saturating_mul(std::mem::size_of::<String>()))
            .saturating_add(count.saturating_mul(std::mem::size_of::<Vec<Value>>()));
        for name in query
            .group_by
            .iter()
            .chain(query.aggregates.iter().map(|column| &column.alias))
        {
            bytes = bytes.saturating_add(name.len());
        }
        for (key, reductions) in self.entries.iter().take(count) {
            bytes = bytes.saturating_add(columns.saturating_mul(std::mem::size_of::<Value>()));
            for value in &key.0 {
                bytes = bytes.saturating_add(
                    fields::value_memory(value, 0)?.saturating_sub(std::mem::size_of::<Value>()),
                );
            }
            for reduction in reductions {
                bytes = bytes.saturating_add(reduction.text_bytes());
            }
        }
        // Finishing An Order Statistic Copies Its Retained Values Once.
        bytes = bytes.saturating_add(
            self.retained
                .saturating_mul(std::mem::size_of::<ScalarValue>()),
        );
        admitted(bytes, allowance)?;
        let mut names = slots(columns)?;
        for name in query
            .group_by
            .iter()
            .chain(query.aggregates.iter().map(|column| &column.alias))
        {
            names.push(fields::copy_text(name)?);
        }
        let mut rows = slots(count)?;
        for (key, reductions) in self.entries.iter().take(count) {
            let mut row = slots(columns)?;
            for value in &key.0 {
                row.push(fields::copy_value(value, 0)?);
            }
            for reduction in reductions {
                row.push(value(reduction.clone().finish(&mut || Ok(()))?)?);
            }
            rows.push(row);
        }
        Ok(AggregateOutcome {
            columns: names,
            rows,
            complete: !self.omitted && self.entries.len() <= effective_limit(query.limit),
        })
    }

    fn copy(&self) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for (key, values) in &self.entries {
            let mut copied = slots(values.len())?;
            copied.extend(values.iter().cloned());
            entries.insert(key.copy()?, copied);
        }
        Ok(Self {
            entries,
            omitted: self.omitted,
            retained: self.retained,
        })
    }
}
impl MemorySize for Groups {
    fn memory_size(&self) -> usize {
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(harmos::btree_map_memory::<Key, Vec<Accumulator>>(
                self.entries.len(),
            ))
            .saturating_add(
                self.retained
                    .saturating_mul(std::mem::size_of::<ScalarValue>()),
            );
        for (key, values) in &self.entries {
            bytes = bytes
                .saturating_add(key.memory_size().saturating_sub(std::mem::size_of::<Key>()))
                .saturating_add(accumulators_memory(values));
        }
        bytes
    }
}
impl MemorySize for Key {
    fn memory_size(&self) -> usize {
        self.0.memory_size()
    }
}
impl Key {
    fn copy(&self) -> Result<Self> {
        let mut values = slots(self.0.len())?;
        for value in &self.0 {
            values.push(fields::copy_value(value, 0)?);
        }
        Ok(Self(values))
    }
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_arrays(&self.0, &other.0)
    }
}

/* === Public Functions === */
pub(super) fn validate(query: &AggregateQuery) -> Result<()> {
    if query.group_by.len() > MAX_COLUMNS
        || query.aggregates.is_empty()
        || query.aggregates.len() > MAX_COLUMNS
        || query.filters.len() > MAX_FILTERS
    {
        return Err(invalid(
            "aggregate request exceeds its column or filter bound",
        ));
    }
    for (index, name) in query.group_by.iter().enumerate() {
        column(query, name)?;
        if query.group_by[..index].contains(name) {
            return Err(invalid("group column is repeated"));
        }
    }
    for (index, column) in query.aggregates.iter().enumerate() {
        if column.alias.is_empty()
            || column.alias.len() > MAX_NAME_BYTES
            || query.group_by.contains(&column.alias)
            || query.aggregates[..index]
                .iter()
                .any(|prior| prior.alias == column.alias)
        {
            return Err(invalid("aggregate alias is invalid or repeated"));
        }
        if !column.aggregate.answerable() {
            return Err(invalid("aggregate percentile rank exceeds 100"));
        }
        if column.column.is_empty() {
            if column.aggregate != Aggregate::Count {
                return Err(invalid("aggregate requires a native column"));
            }
        } else {
            self::column(query, &column.column)?;
        }
    }
    for filter in &query.filters {
        column(query, &filter.column)?;
        let count = filter.values.len();
        match filter.op {
            AggregateOperator::IsNull if count != 0 => {
                return Err(invalid("is_null requires no values"));
            }
            AggregateOperator::In if count == 0 || count > MAX_FILTER_VALUES => {
                return Err(invalid("in filter exceeds its value bound"));
            }
            AggregateOperator::IsNull | AggregateOperator::In => {}
            _ if count != 1 => return Err(invalid("scalar filter requires one value")),
            _ => {}
        }
        for value in &filter.values {
            if matches!(value, Value::Array(_) | Value::Object(_)) {
                return Err(invalid("filter values must be scalars"));
            }
            if matches!(value, Value::Number(value) if !value.is_finite()) {
                return Err(invalid("filter number must be finite"));
            }
        }
    }
    Ok(())
}

/* === Private Functions === */
fn update_memory(
    current: Option<&Groups>,
    query: &AggregateQuery,
    rows: &[Row<'_>],
) -> Result<usize> {
    let count = current
        .map_or(0, |current| current.entries.len())
        .saturating_add(rows.len())
        .saturating_add(1)
        .min(effective_limit(query.limit) + 2);
    let mut bytes = std::mem::size_of::<Groups>()
        .saturating_add(harmos::btree_map_memory::<Key, Vec<Accumulator>>(count));
    if let Some(current) = current {
        bytes = bytes.saturating_add(
            current
                .memory_size()
                .saturating_sub(std::mem::size_of::<Groups>()),
        );
    }
    if query.group_by.is_empty() && current.is_none() {
        bytes = bytes.saturating_add(
            query
                .aggregates
                .len()
                .saturating_mul(std::mem::size_of::<Accumulator>()),
        );
    }
    for row in rows {
        bytes = bytes
            .saturating_add(
                query
                    .group_by
                    .len()
                    .saturating_mul(std::mem::size_of::<Value>()),
            )
            .saturating_add(
                query
                    .aggregates
                    .len()
                    .saturating_mul(std::mem::size_of::<Accumulator>()),
            );
        // The Retained Key And Lookup Key Coexist During Insertion.
        for name in &query.group_by {
            let owned = row
                .field(name)?
                .projection_memory()?
                .saturating_sub(std::mem::size_of::<Value>());
            bytes = bytes.saturating_add(owned).saturating_add(owned);
        }
        bytes = bytes.saturating_add(
            query
                .group_by
                .len()
                .saturating_mul(std::mem::size_of::<Value>()),
        );
        for column in &query.aggregates {
            if !column.column.is_empty() {
                bytes = bytes.saturating_add(row.field(&column.column)?.projection_memory()?);
            }
        }
        for filter in &query.filters {
            bytes = bytes.saturating_add(row.field(&filter.column)?.projection_memory()?);
        }
    }
    if bytes == usize::MAX {
        return Err(invalid("aggregate projection memory overflows"));
    }
    Ok(bytes)
}
fn reductions(query: &AggregateQuery) -> Result<Vec<Accumulator>> {
    let mut values = slots(query.aggregates.len())?;
    for column in &query.aggregates {
        values.push(Accumulator::new(Agg::Of(column.aggregate)));
    }
    Ok(values)
}
fn accumulators_memory(values: &[Accumulator]) -> usize {
    values
        .iter()
        .fold(std::mem::size_of_val(values), |bytes, accumulator| {
            bytes.saturating_add(accumulator.text_bytes())
        })
}
// The Harmos reduction reads scalars; a nested value has no order or number.
// `Count` only asks whether a value is present, so any non-null value counts.
fn scalar(value: &Value, aggregate: Aggregate) -> Result<ScalarValue> {
    Ok(match value {
        Value::Null => ScalarValue::Null,
        _ if aggregate == Aggregate::Count => ScalarValue::Unsigned(1),
        Value::Bool(value) => ScalarValue::Bool(*value),
        Value::Integer(value) => ScalarValue::Signed(*value),
        Value::Unsigned(value) => ScalarValue::Unsigned(*value),
        Value::Number(value) if value.is_finite() => ScalarValue::Float(*value),
        Value::Number(_) => return Err(invalid("aggregate numbers must be finite")),
        Value::Text(value) => ScalarValue::Text(fields::copy_text(value)?),
        Value::Datetime(value) => ScalarValue::Datetime(value.unix_timestamp_nanos()),
        Value::Array(_) | Value::Object(_) => {
            return Err(invalid("aggregate values must be scalars"));
        }
    })
}
fn value(scalar: ScalarValue) -> Result<Value> {
    Ok(match scalar {
        ScalarValue::Missing | ScalarValue::Null => Value::Null,
        ScalarValue::Bool(value) => Value::Bool(value),
        ScalarValue::Signed(value) => Value::Integer(value),
        ScalarValue::Unsigned(value) => i64::try_from(value)
            .map(Value::Integer)
            .unwrap_or(Value::Unsigned(value)),
        ScalarValue::Float(value) => Value::Number(value),
        ScalarValue::Text(value) => Value::Text(value),
        ScalarValue::Datetime(value) => Value::Datetime(
            time::OffsetDateTime::from_unix_timestamp_nanos(value)
                .map_err(|_| invalid("aggregate datetime is out of range"))?,
        ),
        ScalarValue::Exact(value) => {
            let magnitude = value.numerator as f64 / value.denominator.get() as f64;
            Value::Number(if value.negative {
                -magnitude
            } else {
                magnitude
            })
        }
    })
}
fn matches(row: &Row<'_>, query: &AggregateQuery, allowance: NonZeroUsize) -> Result<bool> {
    for filter in &query.filters {
        let value = row.field(&filter.column)?.project(allowance)?;
        if filter.op == AggregateOperator::IsNull {
            if !matches!(value, Value::Null) {
                return Ok(false);
            }
            continue;
        }
        if matches!(value, Value::Null) {
            return Ok(false);
        }
        let matched = if filter.op == AggregateOperator::In {
            let mut found = false;
            for expected in &filter.values {
                if !matches!(expected, Value::Null)
                    && scalar_compare(&value, expected)? == Ordering::Equal
                {
                    found = true;
                    break;
                }
            }
            found
        } else {
            let expected = &filter.values[0];
            if matches!(expected, Value::Null) {
                return Ok(false);
            }
            let order = scalar_compare(&value, expected)?;
            match filter.op {
                AggregateOperator::Eq => order == Ordering::Equal,
                AggregateOperator::Neq => order != Ordering::Equal,
                AggregateOperator::Gt => order == Ordering::Greater,
                AggregateOperator::Gte => order != Ordering::Less,
                AggregateOperator::Lt => order == Ordering::Less,
                AggregateOperator::Lte => order != Ordering::Greater,
                AggregateOperator::In | AggregateOperator::IsNull => {
                    unreachable!("filter validated")
                }
            }
        };
        if !matched {
            return Ok(false);
        }
    }
    Ok(true)
}
fn scalar_compare(left: &Value, right: &Value) -> Result<Ordering> {
    let compatible = match (left, right) {
        (Value::Bool(_), Value::Bool(_))
        | (Value::Text(_), Value::Text(_))
        | (Value::Datetime(_), Value::Datetime(_)) => true,
        _ => numeric(left) && numeric(right),
    };
    if !compatible {
        return Err(invalid("aggregate filter scalar types differ"));
    }
    Ok(compare(left, right))
}
fn compare(left: &Value, right: &Value) -> Ordering {
    use Value::*;
    match (left, right) {
        (Integer(left), Integer(right)) => left.cmp(right),
        (Unsigned(left), Unsigned(right)) => left.cmp(right),
        (Integer(left), Unsigned(right)) => i128::from(*left).cmp(&i128::from(*right)),
        (Unsigned(left), Integer(right)) => i128::from(*left).cmp(&i128::from(*right)),
        (Number(left), Integer(right)) => compare_float_integer(*left, i128::from(*right)),
        (Number(left), Unsigned(right)) => compare_float_integer(*left, i128::from(*right)),
        (Integer(left), Number(right)) => {
            compare_float_integer(*right, i128::from(*left)).reverse()
        }
        (Unsigned(left), Number(right)) => {
            compare_float_integer(*right, i128::from(*left)).reverse()
        }
        (Number(left), Number(right)) => left
            .partial_cmp(right)
            .expect("finite native numbers preflighted"),
        (Bool(left), Bool(right)) => left.cmp(right),
        (Text(left), Text(right)) => left.cmp(right),
        (Datetime(left), Datetime(right)) => left.cmp(right),
        (Array(left), Array(right)) => compare_arrays(left, right),
        (Object(left), Object(right)) => {
            for ((left_key, left_value), (right_key, right_value)) in left.iter().zip(right) {
                let order = left_key
                    .cmp(right_key)
                    .then_with(|| compare(left_value, right_value));
                if order != Ordering::Equal {
                    return order;
                }
            }
            left.len().cmp(&right.len())
        }
        _ => rank(left).cmp(&rank(right)),
    }
}
fn compare_arrays(left: &[Value], right: &[Value]) -> Ordering {
    for (left, right) in left.iter().zip(right) {
        let order = compare(left, right);
        if order != Ordering::Equal {
            return order;
        }
    }
    left.len().cmp(&right.len())
}
fn rank(value: &Value) -> u8 {
    match value {
        Value::Bool(_) => 0,
        Value::Integer(_) | Value::Unsigned(_) | Value::Number(_) => 1,
        Value::Text(_) => 2,
        Value::Datetime(_) => 3,
        Value::Array(_) => 4,
        Value::Object(_) => 5,
        Value::Null => 6,
    }
}
fn compare_float_integer(value: f64, integer: i128) -> Ordering {
    if value < i64::MIN as f64 {
        return Ordering::Less;
    }
    if value >= 18_446_744_073_709_551_616.0 {
        return Ordering::Greater;
    }
    let whole = value.trunc() as i128;
    whole.cmp(&integer).then_with(|| {
        value
            .partial_cmp(&(whole as f64))
            .expect("finite native number")
    })
}
fn numeric(value: &Value) -> bool {
    matches!(
        value,
        Value::Integer(_) | Value::Unsigned(_) | Value::Number(_)
    )
}
fn column(query: &AggregateQuery, name: &str) -> Result<()> {
    if name.len() > MAX_NAME_BYTES {
        return Err(invalid("aggregate column name exceeds its byte bound"));
    }
    if !fields::known(query.source, name) {
        let mut message = format!("unknown column {name}; available:");
        for column in fields::columns(query.source) {
            message.push(' ');
            message.push_str(column);
        }
        return Err(Error::validation(
            "aggregate_query_invalid",
            message,
            Vec::new(),
        ));
    }
    Ok(())
}
fn effective_limit(limit: usize) -> usize {
    if limit == 0 {
        AGGREGATE_ROW_CEILING
    } else {
        limit.min(AGGREGATE_ROW_CEILING)
    }
}
fn slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| invalid("aggregate allocation failed"))?;
    if values.capacity() > count {
        return Err(invalid("aggregate capacity exceeds admission"));
    }
    Ok(values)
}
fn admitted(bytes: usize, allowance: NonZeroUsize) -> Result<()> {
    if bytes == usize::MAX || bytes > allowance.get() {
        return Err(Error::Harmos(harmos::Error::ResourceExhausted {
            resource: "native aggregate projection",
        }));
    }
    Ok(())
}
fn invalid(message: &'static str) -> Error {
    Error::validation("aggregate_query_invalid", message, Vec::new())
}

/* === Tests === */
