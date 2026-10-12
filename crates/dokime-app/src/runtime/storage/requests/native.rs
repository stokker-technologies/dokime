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
use harmos::storage::{
    ObjectComparison, ObjectNullOrder, ObjectQuery, ObjectScalar, ObjectVersion, QueryOptions,
    QueryValue, ReadLimits, ReadOrder, StorageProductionPreparation, StoredObject,
};
use harmos::{Identifier, MemorySize};

// Local Imports
use super::{Active, Pending, ReceiptLoan, ReceiptStore, RunReceipt, error, receipt_error};
use crate::Result;
use dokime::core::{CommandId, NodeId};
use dokime::execution::request::StepCommandReceipt;
use dokime::storage::{
    Manager,
    requests::{Command, Request},
};

/* === Constants === */
const PAGE_ROWS: usize = 64;
const MAX_COMMANDS: usize = 4097;
const MAX_RECEIPT_BYTES: usize = 4 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Receipts {
    storage: Manager,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Receipts {
    pub(super) fn new(storage: Manager) -> Self {
        Self { storage }
    }
}

impl ReceiptStore for Receipts {
    type Receipt = ReceiptLoan;
    type Keys = QueryValue<Vec<CommandId>>;
    type Command = QueryValue<StepCommandReceipt>;
    type Registry = StorageProductionPreparation;

    fn retain(&self, receipt: RunReceipt) -> Result<ReceiptLoan> {
        let bytes = allowance(receipt.memory_size())?;
        self.storage
            .produce(bytes, |_| Ok(receipt))
            .and_then(|value| value.try_into_value())
            .and_then(|value| value.try_into_shared())
            .map_err(receipt_error)
    }

    fn reserve_entry(&self, id: &CommandId) -> Result<Self::Registry> {
        // One Maximum Node Per Entry Plus Root Covers Actual Map Growth.
        // Value Buffers Have Their Own Loans; This Preclaim Owns Inline Slots
        // And This Entry's Key Until Removal, Including Pending Retry State.
        let nodes = harmos::btree_map_memory::<CommandId, Active<Self>>(1);
        let key = id
            .memory_size()
            .saturating_sub(std::mem::size_of::<CommandId>());
        let position = self
            .storage
            .status()
            .accepted
            .memory_size()
            .saturating_sub(std::mem::size_of::<harmos::storage::StorageCursor>());
        let bytes = nodes
            .checked_add(key)
            .and_then(|bytes| bytes.checked_add(position))
            .ok_or_else(|| error("receipt registry size overflow"))?;
        self.storage
            .prepare_production(allowance(bytes)?)
            .map_err(receipt_error)
    }

    fn revise(
        &self,
        receipt: &ReceiptLoan,
        growth: usize,
        change: impl FnOnce(&mut RunReceipt) -> Result<()>,
    ) -> Result<ReceiptLoan> {
        let bytes = receipt
            .memory_size()
            .checked_add(growth)
            .ok_or_else(|| error("receipt update size overflow"))?;
        receipt
            .try_project(allowance(bytes)?, |original, _allowance| {
                let mut receipt = original.clone();
                change(&mut receipt).map_err(|source| {
                    harmos::Error::backend(
                        "update request receipt",
                        harmos::BackendFailure::Permanent,
                        source,
                    )
                })?;
                if receipt.commands.len() > MAX_COMMANDS {
                    return Err(invalid("command receipt history exceeded capacity"));
                }
                Ok(receipt)
            })
            .and_then(|value| value.try_into_shared())
            .map_err(receipt_error)
    }

    fn command(&self, receipt: &ReceiptLoan, id: &CommandId) -> Result<Self::Command> {
        let command = receipt
            .commands
            .get(id)
            .ok_or_else(|| error("command receipt is missing"))?;
        receipt
            .try_project(allowance(command.memory_size())?, |receipt, _allowance| {
                Ok(receipt.commands[id].clone())
            })
            .map_err(receipt_error)
    }

    async fn read(&self, id: &CommandId) -> Result<Option<ReceiptLoan>> {
        read(&self.storage, id)
            .await?
            .map(|value| value.try_into_shared().map_err(receipt_error))
            .transpose()
    }

    async fn pending(
        &self,
        node: &NodeId,
        after: Option<&CommandId>,
    ) -> Result<Option<Self::Keys>> {
        pending(&self.storage, node, after).await
    }

    async fn save(&self, pending: &mut Pending<ReceiptLoan>) -> Result<()> {
        save(&self.storage, pending).await
    }
}

/* === Public Functions === */
pub(super) async fn read(
    storage: &Manager,
    id: &CommandId,
) -> Result<Option<QueryValue<RunReceipt>>> {
    let limit = NonZeroUsize::new(MAX_RECEIPT_BYTES).unwrap();
    let mut query = ObjectQuery::<Request>::new();
    query.item = Some(id.as_str());
    query.options = options(1);
    query.options.page.as_mut().unwrap().bytes = NonZeroUsize::new(64 * 1024).unwrap();
    let selection = storage.query(query).await.map_err(receipt_error)?;
    let view = selection
        .view()
        .ok_or_else(|| error("parent query has no saved view"))?
        .clone();
    drop(selection);
    let Some(parent) = storage
        .read_object_in::<Request>(&view, id.as_str(), ObjectVersion::Latest, limit)
        .await
        .map_err(receipt_error)?
    else {
        return Ok(None);
    };
    let mut receipt = view
        .project_owned(parent, limit, |parent, allowance| {
            let bytes = parent
                .value
                .projection_memory()
                .map_err(codec_error)?
                .checked_add(parent.value.memory_size());
            if bytes.is_none_or(|bytes| bytes > allowance.get()) {
                return Err(harmos::Error::ResourceExhausted {
                    resource: "request parent projection memory",
                });
            }
            let receipt = parent.value.into_receipt(allowance).map_err(codec_error)?;
            if receipt.summary.request_id != *id {
                return Err(invalid("request key differs from payload identity"));
            }
            Ok(receipt)
        })
        .map_err(receipt_error)?;

    // Latest Typed Commands Share The Original Parent's Saved View
    let mut query = ObjectQuery::<Command>::new()
        .project(Vec::new())
        .map_err(receipt_error)?;
    query.options = options(PAGE_ROWS);
    query.options.page.as_mut().unwrap().bytes = NonZeroUsize::new(64 * 1024).unwrap();
    query
        .filter(
            field("request_id"),
            ObjectComparison::Equal,
            ObjectScalar::Text(id.to_string()),
        )
        .map_err(receipt_error)?;
    query
        .order_by(
            field("command_id"),
            ReadOrder::Ascending,
            ObjectNullOrder::Last,
        )
        .map_err(receipt_error)?;
    let mut commands = storage.query_in(&view, query).map_err(receipt_error)?;
    while let Some(rows) = commands.next_page().await.map_err(receipt_error)? {
        for row in rows.iter() {
            let command = storage
                .read_object_in::<Command>(
                    &view,
                    row.item.as_str(),
                    ObjectVersion::Exact(row.version.clone()),
                    NonZeroUsize::new(MAX_RECEIPT_BYTES).unwrap(),
                )
                .await
                .map_err(receipt_error)?
                .ok_or_else(|| error("selected command receipt is missing from its saved view"))?;
            receipt = view
                .project_owned(
                    receipt,
                    NonZeroUsize::new(MAX_RECEIPT_BYTES).unwrap(),
                    |mut combined, allowance| {
                        let count = combined
                            .commands
                            .len()
                            .checked_add(1)
                            .filter(|count| *count <= MAX_COMMANDS)
                            .ok_or_else(|| invalid("command receipt history exceeded capacity"))?;
                        let growth =
                            harmos::btree_map_memory::<CommandId, StepCommandReceipt>(count)
                                .saturating_sub(harmos::btree_map_memory::<
                                    CommandId,
                                    StepCommandReceipt,
                                >(count - 1));
                        let bytes = combined
                            .memory_size()
                            .checked_add(growth)
                            .and_then(|bytes| bytes.checked_add(command.value.memory_size()))
                            .and_then(|bytes| bytes.checked_add(command.value.projection_memory()))
                            .and_then(|bytes| bytes.checked_add(command.value.key().len()));
                        if bytes.is_none_or(|bytes| bytes > allowance.get()) {
                            return Err(harmos::Error::ResourceExhausted {
                                resource: "command receipt projection memory",
                            });
                        }
                        let command = command
                            .value
                            .clone()
                            .into_receipt(id, allowance)
                            .map_err(codec_error)?;
                        if combined
                            .commands
                            .last_key_value()
                            .is_some_and(|(previous, _)| previous >= &command.command_id)
                        {
                            return Err(invalid("command receipt page did not advance"));
                        }
                        combined
                            .commands
                            .insert(command.command_id.clone(), command);
                        Ok(combined)
                    },
                )
                .map_err(receipt_error)?;
        }
    }
    Ok(Some(receipt))
}

pub(super) async fn pending(
    storage: &Manager,
    node: &NodeId,
    after: Option<&CommandId>,
) -> Result<Option<QueryValue<Vec<CommandId>>>> {
    let mut query = ObjectQuery::<Request>::new()
        .project(vec![field("request_id")])
        .map_err(receipt_error)?;
    query.options = options(PAGE_ROWS);
    query.options.page.as_mut().unwrap().bytes = NonZeroUsize::new(64 * 1024).unwrap();
    query
        .filter(
            field("node_id"),
            ObjectComparison::Equal,
            ObjectScalar::Text(node.to_string()),
        )
        .map_err(receipt_error)?;
    query
        .filter(
            field("terminal"),
            ObjectComparison::Equal,
            ObjectScalar::Bool(false),
        )
        .map_err(receipt_error)?;
    if let Some(after) = after {
        query
            .filter(
                field("request_id"),
                ObjectComparison::Greater,
                ObjectScalar::Text(after.to_string()),
            )
            .map_err(receipt_error)?;
    }
    query
        .order_by(
            field("request_id"),
            ReadOrder::Ascending,
            ObjectNullOrder::Last,
        )
        .map_err(receipt_error)?;
    let mut reader = storage.query(query).await.map_err(receipt_error)?;
    let Some(page) = reader.next_page().await.map_err(receipt_error)? else {
        return Ok(None);
    };
    let page = page.try_project(
        NonZeroUsize::new(MAX_RECEIPT_BYTES).unwrap(),
        |rows, allowance| {
            let slots = rows.len().checked_mul(std::mem::size_of::<CommandId>())
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<CommandId>>()))
                .ok_or_else(|| invalid("request key page size overflow"))?;
            let bytes = rows.iter().try_fold(slots, |bytes, row| {
                bytes.checked_add(row.item.as_str().len())
                    .ok_or_else(|| invalid("request key page size overflow"))
            })?;
            if bytes > allowance.get() {
                return Err(harmos::Error::ResourceExhausted {
                    resource: "request key projection memory",
                });
            }
            let mut ids = Vec::new();
            ids.try_reserve_exact(rows.len()).map_err(|_| harmos::Error::ResourceExhausted {
                resource: "request key page slots",
            })?;
            let unused = ids.capacity().saturating_sub(rows.len())
                .checked_mul(std::mem::size_of::<CommandId>())
                .and_then(|unused| bytes.checked_add(unused));
            if unused.is_none_or(|bytes| bytes > allowance.get()) {
                return Err(harmos::Error::ResourceExhausted {
                    resource: "request key page capacity",
                });
            }
            for row in rows {
                if !matches!(row.values.as_slice(), [ObjectScalar::Text(id)] if id == row.item.as_str()) {
                    return Err(invalid("request key differs from projected identity"));
                }
                let id = CommandId::from_uuid(row.item.as_str())
                    .map_err(|source| codec_error(source.into()))?;
                let previous = ids.last().or(after);
                if previous.is_some_and(|previous| previous >= &id) {
                    return Err(invalid("request recovery page did not advance"));
                }
                ids.push(id);
            }
            Ok(ids)
        },
    ).map_err(receipt_error)?;
    Ok(Some(page))
}

pub(super) async fn save(storage: &Manager, pending: &mut Pending<ReceiptLoan>) -> Result<()> {
    if pending.position.is_none() {
        if pending.receipt.commands.len() > MAX_COMMANDS {
            return Err(error("command receipt history exceeded capacity"));
        }
        let count = pending
            .changed_commands()
            .count()
            .checked_add(1)
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| error("receipt object group size overflow"))?;
        if storage.status().failure.is_some() {
            storage.flush().await.map_err(receipt_error)?;
        }
        let parent_bytes = parent_construction(&pending.receipt)?;
        let receipts = storage
            .write_object_group(
                NonZeroUsize::new(MAX_RECEIPT_BYTES).unwrap(),
                count,
                |group| {
                    // Native Builder Scratch Uses The Original Production Ledger
                    for (id, command) in pending.changed_commands() {
                        if id != &command.command_id
                            || pending.receipt.commands.get(id) != Some(command)
                        {
                            return Err(invalid("changed command differs from its parent receipt"));
                        }
                        let bytes =
                            command_construction(&pending.receipt.summary.request_id, command)
                                .map_err(|source| {
                                    harmos::Error::backend(
                                        "prepare command receipt",
                                        harmos::BackendFailure::Permanent,
                                        source,
                                    )
                                })?;
                        let native = storage.produce(bytes, |_| {
                            Command::new(&pending.receipt.summary.request_id, command)
                                .map_err(codec_error)
                        })?;
                        group.write(native.value())?;
                    }

                    // Publish The Parent After Its Command Dependencies
                    // A physical prefix may survive a crash before the entire queue
                    // group is saved. Keep the older recoverable parent until every
                    // changed command precedes the new parent, especially terminal.
                    let parent = storage.produce(parent_bytes, |_| {
                        Request::new(&pending.receipt).map_err(codec_error)
                    })?;
                    group.write(parent.value())?;
                    Ok(())
                },
            )
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
fn options(rows: usize) -> QueryOptions {
    QueryOptions {
        page: Some(ReadLimits {
            rows: NonZeroUsize::new(rows).expect("nonempty page"),
            bytes: NonZeroUsize::new(MAX_RECEIPT_BYTES).unwrap(),
        }),
        decoded_bytes: Some(NonZeroUsize::new(MAX_RECEIPT_BYTES).unwrap()),
        ..QueryOptions::default()
    }
}

fn field(name: &'static str) -> Identifier {
    Identifier::new(name).expect("native receipt field")
}

fn allowance(bytes: usize) -> Result<NonZeroUsize> {
    if bytes > MAX_RECEIPT_BYTES {
        return Err(error("receipt projection exceeds memory allowance"));
    }
    NonZeroUsize::new(bytes).ok_or_else(|| error("empty receipt projection allowance"))
}

fn parent_construction(receipt: &RunReceipt) -> Result<NonZeroUsize> {
    allowance(Request::encoding_memory(receipt)?)
}

fn command_construction(id: &CommandId, command: &StepCommandReceipt) -> Result<NonZeroUsize> {
    allowance(Command::encoding_memory(id, command)?)
}

fn invalid(reason: &'static str) -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "request receipt",
        reason,
    }
}

fn codec_error(source: dokime::storage::Error) -> harmos::Error {
    harmos::Error::backend(
        "decode request receipt",
        harmos::BackendFailure::Permanent,
        source,
    )
}

/* === Tests === */
