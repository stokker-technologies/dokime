/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Native Lifecycle Images and Revision Watches

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use arrow_array::StructArray;
use harmos_operations::storage::{ListRows, ListRowsRequest, ListRowsResponse, RowOutput, WatchRows, WatchRowsRequest, WatchRowsResponse};
use harmos_query::arrow::ArrowBatch;
use harmos_protocol::model::{PageRequest, ReadCut};
use harmos_runtime::operations::{NativeInvocation, NativeOutput, OperationManager};
use harmos_runtime::storage::{ObjectDefinition, ObjectKey, StoredObject, writes::convert};
use harmos_runtime::storage::query::{ObjectSelection, ObjectVersions, Source};

// Local Imports
use super::io;
use crate::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn list<T: StoredObject>(
    operations: &OperationManager, key: Option<ObjectKey>, versions: ObjectVersions, at: Option<ReadCut>, page: PageRequest,
) -> Result<ListRowsResponse> {
    let source = source::<T>(key, versions)?;
    let request = ListRowsRequest { source: Some(source), merge: None, at, transforms: Vec::new(), page: Some(page) };
    io::call::<ListRows>(operations, request).await
}

/// Register From the Opening List Cut Before Draining Its Remaining Pages.
/// Consumers Resume Only at DurableProgress/Resync, Never From Per-Row Positions.
pub fn watch<T: StoredObject>(
    operations: &OperationManager, key: Option<ObjectKey>, at: ReadCut, output: NativeOutput<WatchRowsResponse>,
) -> Result<NativeInvocation<WatchRows>> {
    let source = source::<T>(key, ObjectVersions::Revisions)?;
    operations.submit_native::<WatchRows>(WatchRowsRequest { source: Some(source), at: Some(at) }, output)
        .map_err(|source| Error::Native { action: "Watch Lifecycle: List a Fresh Image and Reopen Its Issued Cut", source })
}

/// Decode One Bounded List Page With the Library's Canonical Column Codec.
pub fn decode<T: StoredObject>(page: &ListRowsResponse, bytes: NonZeroUsize) -> Result<Vec<T>> {
    let graph = ObjectDefinition::of::<T>().graph(&Default::default())
        .map_err(|source| Error::Invalid(format!("Decode Lifecycle Page: Restore Its Frozen Graph: {source}")))?;
    let mut values = Vec::new();
    for row in &page.rows {
        let RowOutput::Batch(data) = row else { continue };
        let batch = ArrowBatch::from_ipc(data.clone()).map_err(|source| Error::Invalid(format!("Decode Lifecycle Page: {source}")))?;
        let payload = batch.as_record_batch().column_by_name("payload")
            .and_then(|column| column.as_any().downcast_ref::<StructArray>())
            .ok_or_else(|| Error::Precondition("Decode Lifecycle Page: Restore Its Typed Payload Columns".into()))?;
        let payloads = convert::encode_payloads(&graph, payload, bytes)
            .map_err(|source| Error::Invalid(format!("Decode Lifecycle Page: {source}")))?;
        for payload in payloads {
            let value = T::decode(&payload).map_err(|source| Error::Invalid(format!("Decode Lifecycle Page: {source}")))?;
            values.push(value);
        }
    }
    Ok(values)
}

/* === Private Functions === */

fn source<T: StoredObject>(key: Option<ObjectKey>, versions: ObjectVersions) -> Result<Source> {
    let schema = ObjectDefinition::of::<T>().schema().map_err(|source| Error::Invalid(format!("Read Lifecycle Rows: Restore Their Schema: {source}")))?;
    Ok(Source::Objects(ObjectSelection { schema: Some(schema), key, versions }))
}

/* === Tests === */
