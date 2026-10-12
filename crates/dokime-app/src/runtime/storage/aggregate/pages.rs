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
    MemorySize,
    storage::{ObjectRevision, StoredRecord},
};

// Local Imports
use super::{fields::Row, groups::Groups};
use crate::runtime::query::aggregate::{AggregateQuery, AggregateSource};
use crate::{Error, Result};
use dokime::execution::TestCycle;
use dokime::storage::{
    objects::{case::Case, cycle::Cycle, execution::Execution},
    records::RecordRow,
};
use dokime::{definition::TestCase, execution::TestExecution, records::RecordEntry};

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */
pub(super) trait Projection {
    type Domain: MemorySize;
    fn memory(&self) -> Result<usize>;
    fn restore(&self, allowance: NonZeroUsize) -> Result<Self::Domain>;
    fn count(value: &Self::Domain, source: AggregateSource) -> Result<usize>;
    fn rows<'a>(value: &'a Self::Domain, source: AggregateSource, rows: &mut Vec<Row<'a>>);
}

/* === Implementations === */
impl Projection for ObjectRevision<Case> {
    type Domain = TestCase;
    fn memory(&self) -> Result<usize> {
        self.version.check_position(&self.position)?;
        Ok(self
            .value
            .memory_size()
            .saturating_add(self.value.projection_memory()?))
    }
    fn restore(&self, _: NonZeroUsize) -> Result<TestCase> {
        Ok(self.value.clone().try_into()?)
    }
    fn count(_: &TestCase, source: AggregateSource) -> Result<usize> {
        if source != AggregateSource::Cases {
            return Err(invalid("case page has another aggregate source"));
        }
        Ok(1)
    }
    fn rows<'a>(value: &'a TestCase, _: AggregateSource, rows: &mut Vec<Row<'a>>) {
        rows.push(Row::Case(value));
    }
}
impl Projection for ObjectRevision<Cycle> {
    type Domain = TestCycle;
    fn memory(&self) -> Result<usize> {
        self.version.check_position(&self.position)?;
        Ok(self
            .value
            .memory_size()
            .saturating_add(self.value.projection_memory()?))
    }
    fn restore(&self, _: NonZeroUsize) -> Result<TestCycle> {
        Ok(self.value.restore()?)
    }
    fn count(_: &TestCycle, source: AggregateSource) -> Result<usize> {
        if source != AggregateSource::Cycles {
            return Err(invalid("cycle page has another aggregate source"));
        }
        Ok(1)
    }
    fn rows<'a>(value: &'a TestCycle, _: AggregateSource, rows: &mut Vec<Row<'a>>) {
        rows.push(Row::Cycle(value));
    }
}
impl Projection for ObjectRevision<Execution> {
    type Domain = TestExecution;
    fn memory(&self) -> Result<usize> {
        self.version.check_position(&self.position)?;
        Ok(self
            .value
            .memory_size()
            .saturating_add(self.value.projection_memory()?))
    }
    fn restore(&self, allowance: NonZeroUsize) -> Result<TestExecution> {
        Ok(self.value.clone().into_execution(allowance)?)
    }
    fn count(value: &TestExecution, source: AggregateSource) -> Result<usize> {
        match source {
            AggregateSource::Executions => Ok(1),
            AggregateSource::Steps => Ok(value.steps.len()),
            _ => Err(invalid("execution page has another aggregate source")),
        }
    }
    fn rows<'a>(value: &'a TestExecution, source: AggregateSource, rows: &mut Vec<Row<'a>>) {
        if source == AggregateSource::Executions {
            rows.push(Row::Execution(value));
            return;
        }
        for (ordinal, step) in value.steps.iter().enumerate() {
            rows.push(Row::Step {
                execution: value,
                step,
                ordinal,
            });
        }
    }
}
impl Projection for StoredRecord<RecordRow> {
    type Domain = RecordEntry;
    fn memory(&self) -> Result<usize> {
        validate_epoch(self.scope.as_ref().map(|scope| scope.as_str()))?;
        Ok(self
            .payload
            .memory_size()
            .saturating_add(self.payload.projection_memory()?))
    }
    fn restore(&self, allowance: NonZeroUsize) -> Result<RecordEntry> {
        let value = self.payload.clone().into_entry(allowance)?;
        if self.item.as_str() != value.record_id.as_str() {
            return Err(invalid(
                "native record item differs from original capture identity",
            ));
        }
        Ok(value)
    }
    fn count(_: &RecordEntry, source: AggregateSource) -> Result<usize> {
        if source != AggregateSource::Records {
            return Err(invalid("record page has another aggregate source"));
        }
        Ok(1)
    }
    fn rows<'a>(value: &'a RecordEntry, _: AggregateSource, rows: &mut Vec<Row<'a>>) {
        rows.push(Row::Record(value));
    }
}

/* === Public Functions === */
pub(super) fn project<T: Projection>(
    entries: &[T],
    current: Option<&Groups>,
    query: &AggregateQuery,
    allowance: NonZeroUsize,
) -> Result<Groups> {
    let mut domain_bytes = std::mem::size_of::<Vec<T::Domain>>();
    for entry in entries {
        domain_bytes = domain_bytes.saturating_add(entry.memory()?);
    }
    admitted(domain_bytes, allowance)?;
    let mut domains = slots(entries.len())?;
    for entry in entries {
        domains.push(entry.restore(allowance)?);
    }
    let retained = domains.memory_size();
    if retained > domain_bytes {
        return Err(exhausted());
    }

    // Count Borrowed Row Slots From Actual Restored Fields Before Allocating Them.
    let mut count = 0usize;
    for value in &domains {
        count = count
            .checked_add(T::count(value, query.source)?)
            .ok_or_else(|| invalid("native aggregate row count overflows"))?;
    }
    // Native Copy And Conversion Scratch Have Settled; Only Actual Domains Remain.
    let scratch = retained
        .saturating_add(std::mem::size_of::<Vec<Row<'_>>>())
        .saturating_add(count.saturating_mul(std::mem::size_of::<Row<'_>>()));
    admitted(scratch, allowance)?;
    let mut rows = slots(count)?;
    for value in &domains {
        T::rows(value, query.source, &mut rows);
    }
    let remaining = allowance
        .get()
        .checked_sub(scratch)
        .and_then(NonZeroUsize::new)
        .ok_or_else(exhausted)?;
    Groups::update(current, query, &rows, remaining)
}

pub(super) fn decode_record(bytes: &[u8], allowance: NonZeroUsize) -> harmos::Result<RecordRow> {
    let native = RecordRow::decoding_memory(bytes).map_err(|source| {
        harmos::Error::backend(
            "decode native aggregate record",
            harmos::BackendFailure::Corrupt,
            source,
        )
    })?;
    let bound = native
        .saturating_add(std::mem::size_of::<Vec<u8>>())
        .saturating_add(bytes.len());
    admitted(bound, allowance).map_err(projection_error)?;
    RecordRow::from_bytes(bytes, bytes.len()).map_err(|source| projection_error(source.into()))
}

pub(super) fn projection_error(source: Error) -> harmos::Error {
    match source {
        Error::Harmos(source) | Error::Storage(dokime::storage::Error::Runtime(source)) => source,
        source => harmos::Error::backend(
            "project native aggregate page",
            harmos::BackendFailure::Corrupt,
            source,
        ),
    }
}

/* === Private Functions === */
fn validate_epoch(epoch: Option<&str>) -> Result<()> {
    let epoch = epoch.ok_or_else(|| invalid("native capture record has no process epoch"))?;
    let uuid = uuid::Uuid::parse_str(epoch)
        .map_err(|_| invalid("native capture process epoch is invalid"))?;
    let mut canonical = [0u8; 36];
    if uuid.hyphenated().encode_lower(&mut canonical) != epoch {
        return Err(invalid("native capture process epoch is not canonical"));
    }
    Ok(())
}
fn slots<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values.try_reserve_exact(count).map_err(|_| exhausted())?;
    if values.capacity() > count {
        return Err(exhausted());
    }
    Ok(values)
}
fn admitted(bytes: usize, allowance: NonZeroUsize) -> Result<()> {
    if bytes == usize::MAX || bytes > allowance.get() {
        return Err(exhausted());
    }
    Ok(())
}
fn exhausted() -> Error {
    harmos::Error::ResourceExhausted {
        resource: "native aggregate page projection",
    }
    .into()
}
fn invalid(message: &'static str) -> Error {
    Error::validation("aggregate_query_invalid", message, Vec::new())
}

/* === Tests === */
