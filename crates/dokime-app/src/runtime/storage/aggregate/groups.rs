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
use harmos::MemorySize;

// Local Imports
use super::fields::{self, Row};
use crate::runtime::query::aggregate::{
    AGGREGATE_ROW_CEILING, AggregateFunction, AggregateOperator, AggregateOutcome, AggregateQuery,
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
    entries: BTreeMap<Key, Vec<Reduction>>,
    omitted: bool,
}
struct Key(Vec<Value>);
struct Mean {
    count: u64,
    value: f64,
}

/* === Enums === */
enum Reduction {
    Count(u64),
    Sum(Sum),
    Mean(Mean),
    Min(Option<Value>),
    Max(Option<Value>),
}
enum Sum {
    Empty,
    Integer(i128),
    Number { value: f64, compensation: f64 },
}

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
            },
        };
        if query.group_by.is_empty() && selected.entries.is_empty() {
            selected.entries.insert(Key(Vec::new()), reductions(query)?);
        }
        for row in rows {
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
                    Value::Unsigned(1)
                } else {
                    row.field(&column.column)?.project(allowance)?
                };
                reduction.add(value)?;
            }
            // Retain The First Limit+1 Groups, Including Exact Aggregates For Each.
            // An Evicted Larger Key Can Never Become A Final Smaller Key.
            if selected.entries.len() > effective_limit(query.limit) + 1 {
                selected.entries.pop_last();
                selected.omitted = true;
            }
        }
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
                bytes = bytes.saturating_add(
                    reduction
                        .memory_size()
                        .saturating_sub(std::mem::size_of::<Reduction>()),
                );
            }
        }
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
                row.push(reduction.value()?);
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
            for value in values {
                copied.push(value.copy()?);
            }
            entries.insert(key.copy()?, copied);
        }
        Ok(Self {
            entries,
            omitted: self.omitted,
        })
    }
}
impl MemorySize for Groups {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>().saturating_add(
            self.entries
                .memory_size()
                .saturating_sub(std::mem::size_of_val(&self.entries)),
        )
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
impl MemorySize for Reduction {
    fn memory_size(&self) -> usize {
        let heap = match self {
            Self::Min(Some(value)) | Self::Max(Some(value)) => value
                .memory_size()
                .saturating_sub(std::mem::size_of::<Value>()),
            _ => 0,
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}
impl Reduction {
    fn copy(&self) -> Result<Self> {
        Ok(match self {
            Self::Count(count) => Self::Count(*count),
            Self::Sum(value) => Self::Sum(value.copy()),
            Self::Mean(value) => Self::Mean(Mean {
                count: value.count,
                value: value.value,
            }),
            Self::Min(value) => Self::Min(
                value
                    .as_ref()
                    .map(|value| fields::copy_value(value, 0))
                    .transpose()?,
            ),
            Self::Max(value) => Self::Max(
                value
                    .as_ref()
                    .map(|value| fields::copy_value(value, 0))
                    .transpose()?,
            ),
        })
    }
    fn add(&mut self, value: Value) -> Result<()> {
        if matches!(value, Value::Null) {
            return Ok(());
        }
        match self {
            Self::Count(count) => {
                *count = count
                    .checked_add(1)
                    .ok_or_else(|| invalid("aggregate count overflows"))?
            }
            Self::Sum(sum) => sum.add(&value)?,
            Self::Mean(mean) => mean.add(number(&value)?)?,
            Self::Min(current) => {
                if current
                    .as_ref()
                    .is_none_or(|current| compare(&value, current) == Ordering::Less)
                {
                    *current = Some(value);
                }
            }
            Self::Max(current) => {
                if current
                    .as_ref()
                    .is_none_or(|current| compare(&value, current) == Ordering::Greater)
                {
                    *current = Some(value);
                }
            }
        }
        Ok(())
    }
    fn value(&self) -> Result<Value> {
        Ok(match self {
            Self::Count(count) => i64::try_from(*count)
                .map(Value::Integer)
                .unwrap_or(Value::Unsigned(*count)),
            Self::Sum(value) => value.value()?,
            Self::Mean(mean) if mean.count == 0 => Value::Null,
            Self::Mean(mean) => Value::Number(mean.value),
            Self::Min(value) | Self::Max(value) => value
                .as_ref()
                .map(|value| fields::copy_value(value, 0))
                .transpose()?
                .unwrap_or(Value::Null),
        })
    }
}
impl Sum {
    fn copy(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Integer(value) => Self::Integer(*value),
            Self::Number {
                value,
                compensation,
            } => Self::Number {
                value: *value,
                compensation: *compensation,
            },
        }
    }
    fn add(&mut self, value: &Value) -> Result<()> {
        if let Value::Integer(value) = value {
            return self.add_integer(i128::from(*value));
        }
        if let Value::Unsigned(value) = value {
            return self.add_integer(i128::from(*value));
        }
        let incoming = number(value)?;
        let (sum, compensation) = match self {
            Self::Empty => (0.0, 0.0),
            Self::Integer(value) => (*value as f64, 0.0),
            Self::Number {
                value,
                compensation,
            } => (*value, *compensation),
        };
        let next = sum + incoming;
        let correction = if sum.abs() >= incoming.abs() {
            (sum - next) + incoming
        } else {
            (incoming - next) + sum
        };
        let compensation = compensation + correction;
        if !next.is_finite() || !compensation.is_finite() {
            return Err(invalid("aggregate sum exceeds finite range"));
        }
        *self = Self::Number {
            value: next,
            compensation,
        };
        Ok(())
    }
    fn add_integer(&mut self, incoming: i128) -> Result<()> {
        match self {
            Self::Empty => {
                *self = Self::Integer(incoming);
                Ok(())
            }
            Self::Integer(value) => {
                *value = value
                    .checked_add(incoming)
                    .ok_or_else(|| invalid("aggregate integer sum overflows"))?;
                Ok(())
            }
            Self::Number { .. } => self.add(&Value::Number(incoming as f64)),
        }
    }
    fn value(&self) -> Result<Value> {
        match self {
            Self::Empty => Ok(Value::Null),
            Self::Integer(value) => i64::try_from(*value)
                .map(Value::Integer)
                .or_else(|_| u64::try_from(*value).map(Value::Unsigned))
                .map_err(|_| invalid("aggregate sum exceeds native integer range")),
            Self::Number {
                value,
                compensation,
            } => {
                let value = value + compensation;
                if !value.is_finite() {
                    return Err(invalid("aggregate sum exceeds finite range"));
                }
                Ok(Value::Number(value))
            }
        }
    }
}
impl Mean {
    fn add(&mut self, value: f64) -> Result<()> {
        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| invalid("aggregate count overflows"))?;
        if self.count == 1 {
            self.value = value;
            return Ok(());
        }
        let count = self.count as f64;
        // Same-Sign Differences Cannot Overflow; Opposite-Sign Weighted Terms Cannot Add Out Of Range.
        self.value = if self.value.is_sign_negative() == value.is_sign_negative() {
            self.value + (value - self.value) / count
        } else {
            self.value * ((count - 1.0) / count) + value / count
        };
        if !self.value.is_finite() {
            return Err(invalid("aggregate mean exceeds finite range"));
        }
        Ok(())
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
        if column.column.is_empty() {
            if column.function != AggregateFunction::Count {
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
        .saturating_add(harmos::btree_map_memory::<Key, Vec<Reduction>>(count));
    if let Some(current) = current {
        for (key, values) in &current.entries {
            bytes =
                bytes.saturating_add(key.memory_size().saturating_sub(std::mem::size_of::<Key>()));
            bytes = bytes.saturating_add(
                values
                    .memory_size()
                    .saturating_sub(std::mem::size_of_val(values)),
            );
        }
    }
    if query.group_by.is_empty() && current.is_none() {
        bytes = bytes.saturating_add(
            query
                .aggregates
                .len()
                .saturating_mul(std::mem::size_of::<Reduction>()),
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
                    .saturating_mul(std::mem::size_of::<Reduction>()),
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
fn reductions(query: &AggregateQuery) -> Result<Vec<Reduction>> {
    let mut values = slots(query.aggregates.len())?;
    for column in &query.aggregates {
        values.push(match column.function {
            AggregateFunction::Count => Reduction::Count(0),
            AggregateFunction::Sum => Reduction::Sum(Sum::Empty),
            AggregateFunction::Mean => Reduction::Mean(Mean {
                count: 0,
                value: 0.0,
            }),
            AggregateFunction::Min => Reduction::Min(None),
            AggregateFunction::Max => Reduction::Max(None),
        });
    }
    Ok(values)
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
fn number(value: &Value) -> Result<f64> {
    match value {
        Value::Integer(value) => Ok(*value as f64),
        Value::Unsigned(value) => Ok(*value as f64),
        Value::Number(value) if value.is_finite() => Ok(*value),
        _ => Err(invalid("sum and mean require finite numeric values")),
    }
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
