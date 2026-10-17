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
use std::{cmp::Ordering, collections::BTreeMap};

// Third Party Imports
use harmos::{
    Aggregate,
    storage::query::{
        Accumulator, Scratch,
        scalar::{Agg, ScalarValue},
    },
};

// Local Imports
use super::fields::{self, Row};
use crate::runtime::query::aggregate::{AggregateOperator, AggregateOutcome, AggregateQuery};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::values::Value;

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
}
struct Key(Vec<Value>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Groups {
    /// Fold rows into at most `limit + 1` groups in key order.
    pub(super) fn fold(query: &AggregateQuery, rows: &[Row<'_>]) -> Result<Self> {
        validate(query)?;
        let mut selected = Self { entries: BTreeMap::new(), omitted: false };
        if query.group_by.is_empty() {
            selected.entries.insert(Key(Vec::new()), reductions(query));
        }
        let mut scratch = Scratch::new(rows.len().saturating_mul(query.aggregates.len()));
        for (ordinal, row) in rows.iter().enumerate() {
            if row.source() != query.source {
                return Err(invalid("aggregate row source differs from its query"));
            }
            if !matches(row, query)? {
                continue;
            }
            let mut key = Vec::with_capacity(query.group_by.len());
            for column in &query.group_by {
                key.push(row.field(column)?.project()?);
            }
            let reductions = selected.entries.entry(Key(key)).or_insert_with(|| reductions(query));
            for (reduction, column) in reductions.iter_mut().zip(&query.aggregates) {
                let value = if column.column.is_empty() {
                    ScalarValue::Unsigned(1)
                } else {
                    scalar(&row.field(&column.column)?.project()?, column.aggregate)?
                };
                reduction.push(ordinal as u64, &value, &mut scratch).map_err(reduction_error)?;
            }
            // Retain The First Limit+1 Groups; An Evicted Larger Key Never Returns.
            if query.limit != 0 && selected.entries.len() > query.limit + 1 {
                selected.entries.pop_last();
                selected.omitted = true;
            }
        }
        Ok(selected)
    }

    pub(super) fn outcome(self, query: &AggregateQuery) -> Result<AggregateOutcome> {
        let count = match query.limit {
            0 => self.entries.len(),
            limit => self.entries.len().min(limit),
        };
        let complete = !self.omitted && self.entries.len() <= count;
        let columns = query.group_by.iter().cloned().chain(query.aggregates.iter().map(|column| column.alias.clone())).collect();
        let mut rows = Vec::with_capacity(count);
        for (key, reductions) in self.entries.into_iter().take(count) {
            let mut row = key.0;
            for reduction in reductions {
                row.push(value(reduction.finish(&mut || Ok(())).map_err(reduction_error)?)?);
            }
            rows.push(row);
        }
        Ok(AggregateOutcome { columns, rows, complete })
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
        return Err(invalid("aggregate request exceeds its column or filter bound"));
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
            || query.aggregates[..index].iter().any(|prior| prior.alias == column.alias)
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
fn reductions(query: &AggregateQuery) -> Vec<Accumulator> {
    query.aggregates.iter().map(|column| Accumulator::new(Agg::Of(column.aggregate))).collect()
}

// The Scalar Fold Reads Scalars; A Nested Value Has No Order Or Number.
// `Count` Only Asks Whether A Value Is Present, So Any Non-Null Value Counts.
fn scalar(value: &Value, aggregate: Aggregate) -> Result<ScalarValue> {
    Ok(match value {
        Value::Null => ScalarValue::Null,
        _ if aggregate == Aggregate::Count => ScalarValue::Unsigned(1),
        Value::Bool(value) => ScalarValue::Bool(*value),
        Value::Integer(value) => ScalarValue::Signed(*value),
        Value::Unsigned(value) => ScalarValue::Unsigned(*value),
        Value::Number(value) if value.is_finite() => ScalarValue::Float(*value),
        Value::Number(_) => return Err(invalid("aggregate numbers must be finite")),
        Value::Text(value) => ScalarValue::Text(value.clone()),
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
        ScalarValue::Unsigned(value) => i64::try_from(value).map(Value::Integer).unwrap_or(Value::Unsigned(value)),
        ScalarValue::Float(value) => Value::Number(value),
        ScalarValue::Text(value) => Value::Text(value),
        ScalarValue::Datetime(value) => Value::Datetime(
            time::OffsetDateTime::from_unix_timestamp_nanos(value).map_err(|_| invalid("aggregate datetime is out of range"))?,
        ),
        ScalarValue::Exact(value) => {
            let magnitude = value.numerator as f64 / value.denominator.get() as f64;
            Value::Number(if value.negative { -magnitude } else { magnitude })
        }
    })
}

fn matches(row: &Row<'_>, query: &AggregateQuery) -> Result<bool> {
    for filter in &query.filters {
        let value = row.field(&filter.column)?.project()?;
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
                if !matches!(expected, Value::Null) && scalar_compare(&value, expected)? == Ordering::Equal {
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
        (Value::Bool(_), Value::Bool(_)) | (Value::Text(_), Value::Text(_)) | (Value::Datetime(_), Value::Datetime(_)) => true,
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
        (Integer(left), Number(right)) => compare_float_integer(*right, i128::from(*left)).reverse(),
        (Unsigned(left), Number(right)) => compare_float_integer(*right, i128::from(*left)).reverse(),
        (Number(left), Number(right)) => left.partial_cmp(right).expect("finite native numbers preflighted"),
        (Bool(left), Bool(right)) => left.cmp(right),
        (Text(left), Text(right)) => left.cmp(right),
        (Datetime(left), Datetime(right)) => left.cmp(right),
        (Array(left), Array(right)) => compare_arrays(left, right),
        (Object(left), Object(right)) => {
            for ((left_key, left_value), (right_key, right_value)) in left.iter().zip(right) {
                let order = left_key.cmp(right_key).then_with(|| compare(left_value, right_value));
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
    whole.cmp(&integer).then_with(|| value.partial_cmp(&(whole as f64)).expect("finite native number"))
}
fn numeric(value: &Value) -> bool {
    matches!(value, Value::Integer(_) | Value::Unsigned(_) | Value::Number(_))
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
        return Err(Error::validation("aggregate_query_invalid", message, Vec::new()));
    }
    Ok(())
}
fn invalid(message: &'static str) -> Error {
    Error::validation("aggregate_query_invalid", message, Vec::new())
}

fn reduction_error(error: harmos::Error) -> Error {
    match error {
        harmos::Error::InvalidInput { reason, .. } => invalid(reason),
        error => error.into(),
    }
}

/* === Tests === */
