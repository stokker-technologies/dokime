/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Object Encoding and Revision Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{fmt::Display, num::NonZeroUsize};

// Third Party Imports
use harmos_operations::storage::{GetObject, GetObjectRequest, ObjectValue, ObjectVersionToken};
use harmos_protocol::model::{Position, ReadCut};
use harmos_runtime::operations::{NativeOperation, OperationManager};
use harmos_runtime::storage::{ObjectDefinition, StorageCursor, StoredObject};
use prost::Message;

// Local Imports
use crate::{Error, Result};
use dokime_storage::persisted;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Version<T> {
    pub value: T,
    pub revision: ObjectVersionToken,
    pub at: ReadCut,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn read<T: StoredObject>(
    operations: &OperationManager, key: &str, at: Option<ReadCut>, bytes: NonZeroUsize,
) -> Result<Option<Version<T>>> {
    let schema = ObjectDefinition::of::<T>().schema().map_err(|source| invalid("Read Lifecycle", source))?;
    let response = call::<GetObject>(operations, GetObjectRequest { schema: Some(schema), item: key.into(), at }).await?;
    let Some(value) = response.value else { return Ok(None) };
    let value = persisted::catalog().decode::<T>(&value, bytes).map_err(|source| invalid("Read Lifecycle", source))?;
    if response.revision.len() > bytes.get() {
        return Err(Error::Bound("Read Lifecycle: Keep the Native Revision Within the Message Bound".into()));
    }
    let revision = Position::decode(response.revision.as_slice())?;
    if revision.sequence == 0 || revision.encode_to_vec() != response.revision {
        return Err(Error::Precondition("Read Lifecycle: Return Its Original Nonzero Object Revision".into()));
    }
    let revision: StorageCursor = revision.try_into().map_err(|source| invalid("Read Lifecycle", source))?;
    let revision = ObjectVersionToken::new(revision);
    let at = response.resolved_at.ok_or_else(|| Error::Precondition("Read Lifecycle: Return Its Issued Cut".into()))?;
    Ok(Some(Version { value, revision, at }))
}

pub(super) fn encode<T: StoredObject>(value: &T, bytes: NonZeroUsize) -> Result<ObjectValue> {
    persisted::catalog().encode(value, bytes).map_err(|source| invalid("Encode Lifecycle", source))
}

pub(super) async fn call<O: NativeOperation<Endpoints = ()>>(operations: &OperationManager, request: O::Request) -> Result<O::Response> {
    operations.call_native::<O>(request).await.map_err(|source| Error::Native { action: O::ID, source })
}

/* === Private Functions === */

fn invalid(action: &str, source: impl Display) -> Error {
    Error::Invalid(format!("{action}: Restore Its Registered Schema and Codec: {source}"))
}

/* === Tests === */
