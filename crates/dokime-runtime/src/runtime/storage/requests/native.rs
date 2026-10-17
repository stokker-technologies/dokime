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
use std::sync::Arc;

// Third Party Imports
use harmos::storage::{
    ObjectRead, ObjectRevision, ObjectVersion, StoredObject,
    query::{
        ObjectVersions,
        scalar::ScalarValue,
        transform::{
            FieldPath,
            filter::{Clause, Compare, Condition, Filter, Term},
        },
    },
};

// Local Imports
use super::{Pending, RunReceipt, error, receipt_error};
use crate::runtime::Result;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::NodeId;
use crate::storage::{
    Manager,
    requests::{Command, Request},
};

/* === Constants === */
const MAX_COMMANDS: usize = 4097;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// The latest saved receipt of one request with its latest command receipts.
pub(super) async fn read(storage: &Manager, id: &CommandId) -> Result<Option<RunReceipt>> {
    let Some(parent) = storage
        .read_object::<Request>(id.as_str(), ObjectVersion::Latest)
        .await
        .map_err(receipt_error)?
    else {
        return Ok(None);
    };
    let mut receipt = parent.value.into_receipt()?;
    if receipt.summary.request_id != *id {
        return Err(error("request key differs from payload identity"));
    }
    let commands = latest::<Command>(
        storage,
        filter(vec![equal("request_id", ScalarValue::Text(id.to_string()))]),
    )
    .await?;
    if commands.len() > MAX_COMMANDS {
        return Err(error("command receipt history exceeded capacity"));
    }
    for command in commands {
        let command = command.value.into_receipt(id)?;
        receipt.commands.insert(command.command_id.clone(), command);
    }
    Ok(Some(receipt))
}

/// This node's nonterminal saved requests, ascending by identity.
pub(super) async fn pending(storage: &Manager, node: &NodeId) -> Result<Vec<CommandId>> {
    let requests = latest::<Request>(
        storage,
        filter(vec![
            equal("node_id", ScalarValue::Text(node.to_string())),
            equal("terminal", ScalarValue::Bool(false)),
        ]),
    )
    .await?;
    let mut ids = requests
        .into_iter()
        .map(|request| {
            CommandId::from_uuid(request.value.key())
                .map_err(|source| receipt_error(crate::storage::Error::from(source)))
        })
        .collect::<Result<Vec<_>>>()?;
    ids.sort();
    Ok(ids)
}

pub(super) async fn save(storage: &Manager, pending: &mut Pending<Arc<RunReceipt>>) -> Result<()> {
    if pending.position.is_none() {
        if pending.receipt.commands.len() > MAX_COMMANDS {
            return Err(error("command receipt history exceeded capacity"));
        }
        let count = pending
            .changed_commands()
            .count()
            .checked_add(1)
            .and_then(std::num::NonZeroUsize::new)
            .ok_or_else(|| error("receipt object group size overflow"))?;
        if storage.status().failure.is_some() {
            storage.flush().await.map_err(receipt_error)?;
        }
        let mut objects = Vec::new();
        for (id, command) in pending.changed_commands() {
            if id != &command.command_id || pending.receipt.commands.get(id) != Some(command) {
                return Err(error("changed command differs from its parent receipt"));
            }
            objects.push(Command::new(&pending.receipt.summary.request_id, command)?);
        }
        // Publish The Parent After Its Command Dependencies
        // A physical prefix may survive a crash before the entire queue
        // group is saved. Keep the older recoverable parent until every
        // changed command precedes the new parent, especially terminal.
        let parent = Request::new(&pending.receipt)?;
        let receipts = storage
            .write_object_group(storage_group_bytes(), count, |group| {
                for command in &objects {
                    group.write(command)?;
                }
                group.write(&parent)
            })
            .await
            .map_err(receipt_error)?;
        pending.position = Some(
            receipts
                .last()
                .expect("nonempty admitted group")
                .position
                .clone(),
        );
    }

    // Save Command Admission And Terminal Publication Before External Effects
    if !pending.receipt.summary.phase.terminal() && pending.changed_commands().next().is_none() {
        return Ok(());
    }
    let position = pending
        .position
        .as_ref()
        .expect("admitted receipt position");
    let saved = storage.flush_until(position).await.map_err(receipt_error)?;
    if !saved.covers(position).map_err(receipt_error)? {
        return Err(error("save did not cover pending receipt"));
    }
    Ok(())
}

/* === Private Functions === */
// One Receipt Group Is Bounded By Its Command History Cap.
fn storage_group_bytes() -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap()
}

async fn latest<T>(storage: &Manager, filter: Filter) -> Result<Vec<ObjectRevision<T>>>
where
    T: StoredObject,
    crate::storage::objects::Objects: harmos::catalog::Contains<T>,
{
    Ok(storage
        .read_objects::<T>(ObjectRead {
            at: storage.cursor_saved(),
            key: None,
            versions: ObjectVersions::Latest,
            filters: vec![filter],
        })
        .await?
        .collect()
        .await?)
}

fn filter(terms: Vec<Term>) -> Filter {
    Filter {
        clauses: vec![Clause { terms }],
    }
}

fn equal(field: &str, value: ScalarValue) -> Term {
    Term {
        field: FieldPath::from(format!("payload.{field}").as_str()),
        condition: Condition::Compare {
            op: Compare::Eq,
            value,
        },
        negate: false,
    }
}

/* === Tests === */
