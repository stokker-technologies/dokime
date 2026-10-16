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
use harmos::storage::{ObjectReleaseCursor, ObjectReleasePage, ObjectRetentionRelease};
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
    pub(crate) async fn data_object_releases(
        &self,
        cut: super::SavedData,
        after: Option<ObjectReleaseCursor>,
        limits: ReadLimits,
        execution: Option<std::sync::Arc<super::super::query::Budget>>,
    ) -> Result<ObjectReleasePage, Error> {
        let reservation = self.reserve_memory(
            limits
                .bytes
                .get()
                .checked_mul(4)
                .and_then(|bytes| bytes.checked_add(16 * 1024))
                .ok_or(Error::Data("object release page allocation overflow"))?,
        )?;
        self.call_read(execution, move |connection| Ok((|| {
            let _reservation = reservation;
            let mut statement = connection.prepare(
                "SELECT r.schema_key,r.item,r.through FROM storage_object_releases AS r
                 WHERE r.generation<=?1 AND r.through<=?2
                 AND r.generation=(SELECT MAX(newer.generation) FROM storage_object_releases AS newer
                     WHERE newer.schema_id=r.schema_id AND newer.item=r.item
                     AND newer.generation<=?1 AND newer.through<=?2)
                 AND (r.schema_id,r.schema_version,r.encoding,r.item)>(?3,?4,?5,?6)
                 ORDER BY r.schema_id,r.schema_version,r.encoding,r.item LIMIT ?7")?;
            let mut rows = statement.query(params![position(cut.retention_generation), position(cut.sequence),
                after.as_ref().map_or("", |after| after.schema.id.as_str()),
                after.as_ref().map_or(0, |after| after.schema.version.get()),
                after.as_ref().map_or(0, |after| after.schema.payload_encoding as u8),
                after.as_ref().map_or("", |after| after.item.as_str()),
                limits.rows.get().min(64) as i64 + 1])?;
            let mut entries = Vec::new();
            let mut bytes = 0usize;
            let mut more = false;
            while let Some(row) = rows.next()? {
                super::super::query::checkpoint()?;
                let key: String = row.get(0)?;
                let item: String = row.get(1)?;
                if key.len() > 8192 || item.len() > 1024 {
                    return Err(Error::Corrupt("object release identity exceeds its bound"));
                }
                let release = ObjectRetentionRelease {
                    schema: serde_json::from_str(&key)?,
                    item: ObjectKey::new(&item, NonZeroUsize::new(1024).unwrap())?,
                    through: StorageCursor { history: cut.history.clone(), sequence: sequence(&row.get::<_,String>(2)?)? },
                };
                let length = super::entry::check_size(&release, limits.bytes.get())?;
                if entries.len() == limits.rows.get().min(64) || bytes + length > limits.bytes.get() {
                    if entries.is_empty() { return Err(Error::Data("object release exceeds page byte bound")); }
                    more = true;
                    break;
                }
                bytes += length;
                entries.push(release);
            }
            let next = if more { entries.last().map(ObjectRetentionRelease::cursor) } else { None };
            Ok(ObjectReleasePage { entries, next })
        })())).await?
    }
    pub(crate) async fn data_release_object_history(
        &self,
        schema: SchemaKey,
        item: ObjectKey,
        through: StorageCursor,
    ) -> Result<(), Error> {
        self.call(move |connection| Ok((|| {
            let transaction = connection.transaction()?;
            super::bootstrap::check_writes(&transaction)?;
            let head = saved(&transaction)?;
            head.history.check(&through.history)?;
            if through.sequence > head.sequence {
                return Err(Error::Data("object history release exceeds saved frontier"));
            }
            let previous: Option<String> = transaction.query_row(
                "SELECT MAX(through) FROM storage_object_releases WHERE schema_id=?1 AND item=?2",
                params![schema.id.as_str(), item.as_str()], |row| row.get(0),
            )?;
            if previous.as_deref().is_some_and(|previous| previous >= position(through.sequence).as_str()) {
                return Ok(());
            }
            let generation = head.retention_generation.checked_add(1)
                .ok_or(Error::Data("retention generation exhausted"))?;
            transaction.execute(
                "INSERT INTO storage_object_releases(schema_id,schema_key,schema_version,encoding,item,through,generation) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                params![schema.id.as_str(), serde_json::to_string(&schema)?, schema.version.get(),
                    schema.payload_encoding as u8, item.as_str(), position(through.sequence), position(generation)],
            )?;
            transaction.execute(
                "UPDATE storage_data_head SET retention_generation=?1 WHERE singleton=1",
                [position(generation)],
            )?;
            transaction.commit()?;
            Ok(())
        })())).await?
    }
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
            .checked_mul(std::mem::size_of::<(String, i64, bool)>() + 1024)
            .and_then(|bytes| bytes.checked_add(1024 + 128 * (std::mem::size_of::<String>() + 16)))
            .ok_or(Error::Data("retention policy allocation size overflow"))?;
        let reservation = self.reserve_memory(allocation)?;
        let mut policies = Vec::with_capacity(C::definition_count());
        C::definitions().try_for_each(|definition| {
            let schema = definition.schema()?;
            let (keep, released) = match definition.retention() {
                ObjectRetention::UntilReleased => (1, true),
                policy => (
                    i64::try_from(policy.versions().get())
                        .map_err(|_| Error::Data("retention count exceeds SQLite range"))?,
                    false,
                ),
            };
            policies.push((schema.id.to_string(), keep, released));
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
            for (schema, keep, released) in &policies {
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
                     AND (?6=0 OR EXISTS(SELECT 1 FROM storage_object_releases AS released
                         WHERE released.schema_id=entries.schema_id AND released.item=entries.item
                         AND entries.position<=released.through))
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
                let mut rows = statement.query(params![schema,position(through.sequence),keep,(maximum-retired.len()) as i64,position(after),released])?;
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
