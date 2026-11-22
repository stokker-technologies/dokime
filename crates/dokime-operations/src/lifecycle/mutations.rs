/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Lifecycle Compound Mutation Admission

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_operations::storage::{CompoundOutcome, Mutate, MutateRequest, MutationId, MutationItem, StorageCursor};
use harmos_protocol::model::Receipt;
use harmos_runtime::operations::OperationManager;
use harmos_runtime::storage::RecordInput;
use harmos_runtime::{Identified, Identifier, MemorySize, Message, schema::Described};

// Local Imports
use super::io;
use crate::{Error, Result};
use dokime_storage::persisted::bindings;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn mutate(operations: &OperationManager, request: MutateRequest, bytes: NonZeroUsize) -> Result<Receipt> {
    if request.memory_size() > bytes.get() {
        return Err(Error::Bound("Commit Lifecycle: Reduce the Mutation Within Its Configured Byte Bound".into()));
    }
    let response = io::call::<Mutate>(operations, request).await?;
    if matches!(response.outcome, CompoundOutcome::Conflict { .. }) {
        return Err(Error::Conflict("Commit Lifecycle: Read the Current Object Revisions and Use a New Mutation Identity".into()));
    }
    let receipt = response.receipt.ok_or_else(|| Error::Precondition("Commit Lifecycle: Return Its Durable Receipt".into()))?;
    if !receipt.completed || receipt.durable_at.is_none() {
        return Err(Error::Precondition("Commit Lifecycle: Complete the Accepted Mutation Before Advancing Its Owner".into()));
    }
    Ok(receipt)
}

pub(super) fn event<T: Identified + Described + Message>(
    mutation: &MutationId, kind: &str, scope: Identifier, value: &T, bytes: NonZeroUsize,
) -> Result<MutationItem> {
    let (schema, _) = bindings::schema::<T>()?;
    let payload = value.encode();
    if payload.len() > bytes.get() {
        return Err(Error::Bound("Append Lifecycle Event: Keep Its Payload Within the Message Bound".into()));
    }
    let item = Identifier::new(format!("{}/{kind}", mutation.key))
        .map_err(|source| Error::Invalid(format!("Append Lifecycle Event: Select a Valid Mutation Identity: {source}")))?;
    Ok(MutationItem::Append(RecordInput {
        schema,
        scope: Some(scope),
        item,
        source: None,
        source_time: None,
        payload,
    }))
}

pub(super) fn bound_preview_deletions(items: &[MutationItem], bytes: NonZeroUsize) -> Result<()> {
    // Reserve the Other Seal Items and Their Response Positions as Well as the Keys.
    let deletion_bytes = items.iter().map(MemorySize::memory_size).sum::<usize>();
    let receipt_bytes = items.len().saturating_add(4).saturating_mul(size_of::<StorageCursor>());
    if deletion_bytes.saturating_add(receipt_bytes) > bytes.get() {
        return Err(Error::Bound("Admit Execution: Reduce Its Derived Object Inventory to Fit One Seal Mutation".into()));
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
