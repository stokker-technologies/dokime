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
use std::{num::NonZeroUsize, sync::Arc};
// Third Party Imports
use harmos::storage::{QueryLimits, ReadKind, ReadSelection, RecordFilter, StorageSchema};
use rusqlite::Connection;
// Local Imports
use super::super::{Error, memory, query};
use super::{Entry, SavedData, scan};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Prepared<'a> {
    filter: &'a RecordFilter,
    projection: scan::projection::Projection<'a>,
    schema: StorageSchema,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<'a> Prepared<'a> {
    pub(super) fn new(
        connection: &Connection,
        memory: &Arc<memory::Budget>,
        cut: &SavedData,
        selection: &'a ReadSelection,
        maximum: NonZeroUsize,
    ) -> Result<Option<Self>, Error> {
        let Some(filter) = selection.record_filter.as_ref() else {
            return Ok(None);
        };
        if selection.kind != ReadKind::Records {
            return Err(harmos::Error::InvalidInput {
                field: "record filter",
                reason: "only original saved records support payload filtering",
            }
            .into());
        }
        filter.check()?;
        scan::projection::admission(selection, &filter.columns)?;
        let schema = scan::schema(connection, memory, cut, selection, maximum, None)?;
        let bytes = scan::projection::construction_memory(&schema, &filter.columns)?;
        let projection = scan::projection::Projection::new(&schema, &filter.columns, bytes)?;
        Ok(Some(Self {
            filter,
            projection,
            schema,
        }))
    }

    pub(super) fn matches(
        &self,
        entry: &Entry,
        selection: &ReadSelection,
        cut: &SavedData,
        memory: &Arc<memory::Budget>,
        maximum: NonZeroUsize,
    ) -> Result<bool, Error> {
        query::checkpoint()?;
        let Entry::Record(record) = entry else {
            return Err(Error::Corrupt(
                "record filter received a different entry kind",
            ));
        };
        scan::check_record(
            record,
            None,
            record.position.sequence,
            &self.schema,
            selection,
            cut,
        )?;
        let bytes = self.projection.output_memory(record, None)?;
        let allowance = NonZeroUsize::new(bytes)
            .ok_or(Error::Data("record filter projection allowance is empty"))?;
        let values = self
            .projection
            .project(record, None, memory, allowance, maximum)?;
        if harmos::MemorySize::memory_size(&values) > bytes {
            return Err(Error::Corrupt("record filter projection exceeded preclaim"));
        }
        let selected = self.filter.matches(&values)?;
        drop(values);
        Ok(selected)
    }
}

/* === Public Functions === */
pub(super) fn admission(selection: &ReadSelection, limits: QueryLimits) -> harmos::Result<()> {
    let Some(filter) = selection.record_filter.as_ref() else {
        return Ok(());
    };
    if selection.kind != ReadKind::Records {
        return Err(harmos::Error::InvalidInput {
            field: "record filter",
            reason: "only original saved records support payload filtering",
        });
    }
    filter.check()?;
    if harmos::MemorySize::memory_size(filter) > limits.memory_bytes.get() {
        return Err(harmos::Error::ResourceExhausted {
            resource: "record filter planning memory",
        });
    }
    scan::projection::admission(selection, &filter.columns)
}

/* === Private Functions === */

/* === Tests === */
