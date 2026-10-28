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

// Third Party Imports
use harmos::storage::{ObjectRead, ObjectVersion, ReadBatch, StoredObject, query::ObjectVersions};

// Local Imports
use super::{Pending, Receipt, RunReceipt, error, receipt_error};
use crate::runtime::Result;
use crate::storage::{
    Manager,
    requests::{Command, Request},
};
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::NodeId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn read(storage: &Manager, session: &crate::storage::Manager, id: &CommandId) -> Result<Option<ReadBatch<RunReceipt>>> {
    let Some(parent) = session.read_object::<Request>(id.as_str(), ObjectVersion::Latest).await.map_err(receipt_error)? else {
        return Ok(None);
    };
    let mut receipt = parent.try_map(|parent| -> Result<_> { Ok(parent.value.into_receipt()?) })?;
    if receipt.summary.request_id != *id {
        return Err(error("request key differs from payload identity"));
    }
    let mut commands = session
        .read_objects::<Command>(ObjectRead {
            at: storage.cursor_saved(),
            key: None,
            versions: ObjectVersions::Latest,
        })
        .await?;
    while let Some(page) = commands.next().await? {
        let count = page.iter().filter(|row| row.value.request_id() == id.as_str()).count();
        if count == 0 {
            continue;
        }
        receipt = receipt.try_map_with(page, |mut receipt, commands| -> Result<_> {
            for row in commands {
                if row.value.request_id() != id.as_str() {
                    continue;
                }
                let command = row.value.into_receipt(id)?;
                if receipt.commands.insert(command.command_id.clone(), command).is_some() {
                    return Err(error("duplicate stored command identity"));
                }
            }
            Ok(receipt)
        })?;
    }
    Ok(Some(receipt))
}

pub(super) async fn pending(storage: &Manager, session: &crate::storage::Manager, node: &NodeId) -> Result<ReadBatch<Vec<CommandId>>> {
    let seed = harmos::storage::ReadBatch::new(());
    let mut ids = seed.try_map(|_| -> Result<_> { Ok(Vec::<CommandId>::new()) })?;
    let mut requests = session
        .read_objects::<Request>(ObjectRead {
            at: storage.cursor_saved(),
            key: None,
            versions: ObjectVersions::Latest,
        })
        .await?;
    while let Some(page) = requests.next().await? {
        let count = page.iter().filter(|row| row.value.pending_on(node.as_str())).count();
        if count == 0 {
            continue;
        }
        let total = ids.len().checked_add(count).ok_or_else(|| error("pending receipt count overflows"))?;
        let capacity = if total > ids.capacity() {
            total.max(ids.capacity().checked_mul(2).ok_or_else(|| error("pending receipt capacity overflows"))?)
        } else {
            ids.capacity()
        };
        ids = ids.try_map_with(page, |mut ids, page| -> Result<_> {
            if capacity > ids.capacity() {
                ids.try_reserve_exact(capacity - ids.len()).map_err(|_| error("pending receipt allocation failed"))?;
            }
            for row in page {
                if row.value.pending_on(node.as_str()) {
                    ids.push(CommandId::from_uuid(row.value.key()).map_err(crate::storage::Error::from)?);
                }
            }
            Ok(ids)
        })?;
    }
    ids.try_map(|mut ids| -> Result<_> {
        ids.sort_unstable();
        Ok(ids)
    })
}

pub(super) async fn save(storage: &Manager, pending: &mut Pending<Receipt>) -> Result<()> {
    if pending.position.is_none() {
        let count = pending
            .changed_commands()
            .count()
            .checked_add(1)
            .and_then(std::num::NonZeroUsize::new)
            .ok_or_else(|| error("receipt object group size overflow"))?;
        if storage.status().failure.is_some() {
            storage.flush().await.map_err(receipt_error)?;
        }
        for (id, command) in pending.changed_commands() {
            if id != &command.command_id || pending.receipt.commands.get(id) != Some(command) {
                return Err(error("changed command differs from its parent receipt"));
            }
        }
        let mut objects = Vec::new();
        objects.try_reserve_exact(count.get() - 1).map_err(|_| error("receipt object allocation failed"))?;
        for (_, command) in pending.changed_commands() {
            objects.push(Command::new(&pending.receipt.summary.request_id, command)?);
        }
        // Publish The Parent After Its Command Dependencies
        // A physical prefix may survive a crash before the entire queue
        // group is saved. Keep the older recoverable parent until every
        // changed command precedes the new parent, especially terminal.
        let parent = Request::new(&pending.receipt)?;
        let receipts = storage
            .write_object_group(storage.settings().request.max_pending_write_bytes, count, |group| {
                for command in &objects {
                    group.write(command)?;
                }
                group.write(&parent)
            })
            .await
            .map_err(receipt_error)?;
        pending.position = Some(receipts.into_iter().last().expect("nonempty admitted group").position);
    }

    // Save Command Admission And Terminal Publication Before External Effects
    if !pending.receipt.summary.phase.terminal() && pending.changed_commands().next().is_none() {
        return Ok(());
    }
    let position = pending.position.as_ref().expect("admitted receipt position");
    let saved = storage.flush_until(position).await.map_err(receipt_error)?;
    if !saved.covers(position).map_err(receipt_error)? {
        return Err(error("save did not cover pending receipt"));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
