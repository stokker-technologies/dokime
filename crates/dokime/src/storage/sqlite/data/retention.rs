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
use harmos::{catalog::Projection, storage::*};
use rusqlite::params;
// Local Imports
use super::super::{Error, State};
use super::{position, saved, sequence};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl State {
    pub(crate) async fn data_prune<C: ObjectCatalog>(
        &self,
        through: StorageCursor,
        after: u64,
        oldest_pin_generation: Option<u64>,
        maximum: NonZeroUsize,
    ) -> Result<(bool, bool), Error> {
        // The Backend Validates Its Immutable Catalog At Initialization.
        // Validated catalog identities are at most 1024 bytes; each descriptor
        // callback's temporary identity coexists with the retained policy vector.
        let allocation = C::definition_count()
            .checked_mul(std::mem::size_of::<(String, i64)>() + 1024)
            .and_then(|bytes| bytes.checked_add(1024 + 128 * (std::mem::size_of::<String>() + 16)))
            .ok_or(Error::Data("retention policy allocation size overflow"))?;
        let reservation = self.reserve_memory(allocation)?;
        let mut policies = Vec::with_capacity(C::definition_count());
        C::definitions().try_for_each(|definition| {
            let schema = definition.schema()?;
            let keep = i64::try_from(definition.retention().versions().get())
                .map_err(|_| Error::Data("retention count exceeds SQLite range"))?;
            policies.push((schema.id.to_string(), keep));
            Ok::<(), Error>(())
        })?;
        let maximum = maximum.get().min(64);
        self.call(move |connection| Ok((|| {
            let _reservation = reservation;
            let transaction = connection.transaction()?;
            super::bootstrap::check_writes(&transaction)?;
            let mut head = saved(&transaction)?;
            head.history.check(&through.history)?;
            if through.sequence > head.sequence { return Err(Error::Data("retention exceeds saved frontier")); }
            let mut retired = Vec::with_capacity(maximum);
            for (schema, keep) in &policies {
                if retired.len() == maximum { break; }
                // Only New Object Versions Can Change Logical Retention Eligibility.
                // File And Evidence Saves Still Reclaim Rows Released By Old Pins.
                let changed: bool = transaction.query_row(
                    "SELECT EXISTS(SELECT 1 FROM storage_data_entries
                     INDEXED BY storage_data_live_object_position WHERE kind='object'
                     AND retired_at IS NULL AND schema_id=?1 AND position>?2 AND position<=?3)",
                    params![schema,position(after),position(through.sequence)],
                    |row| row.get(0),
                )?;
                if !changed { continue; }
                // Changed Keys Supply Bounded Candidate Lookups In Key And Position Order.
                // The Newer-Key Index Stops At The Policy's Keep Count.
                let mut statement = transaction.prepare(
                    "SELECT entries.position FROM storage_data_entries AS entries
                     INDEXED BY storage_data_live_object_key WHERE entries.kind='object'
                     AND entries.schema_id=?1 AND entries.position<=?2 AND entries.retired_at IS NULL
                     AND entries.item IN (SELECT changed.item FROM storage_data_entries AS changed
                         INDEXED BY storage_data_live_object_position WHERE changed.kind='object'
                         AND changed.retired_at IS NULL AND changed.schema_id=?1
                         AND changed.position>?5 AND changed.position<=?2)
                     AND (SELECT COUNT(*) FROM (SELECT 1 FROM storage_data_entries AS newer
                         INDEXED BY storage_data_live_object_key
                         WHERE newer.kind='object' AND newer.schema_id=entries.schema_id
                         AND newer.item=entries.item AND newer.position>entries.position
                         AND newer.position<=?2 AND newer.retired_at IS NULL LIMIT ?3)) >= ?3
                     ORDER BY entries.item,entries.position LIMIT ?4")?;
                let mut rows = statement.query(params![schema,position(through.sequence),keep,(maximum-retired.len()) as i64,position(after)])?;
                while let Some(row)=rows.next()? { retired.push(row.get::<_,String>(0)?); }
            }
            if !retired.is_empty() {
                head.retention_generation = head.retention_generation.checked_add(1).ok_or(Error::Data("retention generation exhausted"))?;
                for value in &retired {
                    head.lost_objects_through = head.lost_objects_through.max(sequence(value)?);
                    transaction.execute("UPDATE storage_data_entries SET retired_at=?1 WHERE position=?2 AND retired_at IS NULL", params![position(head.retention_generation),value])?;
                }
                transaction.execute("UPDATE storage_data_head SET lost_objects_through=?1,retention_generation=?2 WHERE singleton=1", params![position(head.lost_objects_through),position(head.retention_generation)])?;
            }
            let removable = {
                let mut statement = transaction.prepare("SELECT position FROM storage_data_entries INDEXED BY storage_data_retired_object_position WHERE kind='object' AND retired_at IS NOT NULL AND retired_at<=?1 ORDER BY position LIMIT ?2")?;
                let mut rows = statement.query(params![position(oldest_pin_generation.unwrap_or(u64::MAX)),maximum as i64])?;
                let mut positions = Vec::with_capacity(maximum);
                while let Some(row)=rows.next()? { positions.push(row.get::<_,String>(0)?); }
                positions
            };
            let reclaimed = removable.len();
            for value in removable {
                let changed = transaction.execute(
                    "UPDATE storage_data_inventory SET reclaimed=1 WHERE position=?1 AND kind='object' AND reclaimed=0
                     AND EXISTS(SELECT 1 FROM storage_data_entries AS entries
                         WHERE entries.position=storage_data_inventory.position
                         AND entries.kind=storage_data_inventory.kind
                         AND entries.digest=storage_data_inventory.digest AND entries.retired_at IS NOT NULL)",
                    [&value],
                )?;
                if changed != 1 { return Err(Error::Corrupt("object reclamation lacks admission inventory")); }
                transaction.execute("DELETE FROM storage_object_scalars WHERE position=?1",[&value])?;
                transaction.execute("DELETE FROM storage_data_entries WHERE position=?1",[value])?;
            }
            transaction.commit()?;
            // Continue After Either Logical Or Physical Removal Progress.
            // Pins Can Retain Bytes Without Requiring A Busy Maintenance Loop.
            Ok((retired.len() < maximum, retired.is_empty() && reclaimed == 0))
        })())).await?
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
