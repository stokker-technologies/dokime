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
    MemorySize, Protobuf,
    storage::{FileReference, ObjectRevision, QueryValue, SharedQueryValue, StorageView},
};

// Local Imports
use super::{native::source::Source, source::CapturedSource};
use crate::{Error, Result};
use dokime::execution::TestExecution;

/* === Constants === */
const CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn read_execution(
    view: &StorageView,
    execution: &SharedQueryValue<ObjectRevision<TestExecution>>,
) -> Result<QueryValue<CapturedSource>> {
    let seal = execution
        .value
        .saved
        .as_ref()
        .ok_or_else(|| invalid("execution has no captured source dependencies"))?;
    if !execution.position.covers(&seal.dependencies_saved)?
        || execution.position.sequence == seal.dependencies_saved.sequence
    {
        return Err(invalid("seal revision does not follow saved dependencies"));
    }
    let source = &seal.source;
    let maximum = view.decoded_bytes()?;
    let captured = read_file(view, &source.context).await?;
    let computation = view.begin_computation()?;
    let validation_memory = captured
        .memory_size()
        .checked_add(source.outputs.memory_size())
        .and_then(|bytes| bytes.checked_add(execution.value.memory_size()))
        .and_then(|bytes| bytes.checked_add(captured.validation_memory()))
        .ok_or_else(|| invalid("captured owner validation memory overflows"))?;
    let validation_memory = allowance(validation_memory, maximum, "captured owner validation")?;
    computation.check()?;
    drop(computation);
    Ok(
        view.project_owned(captured, validation_memory, |captured, _| {
            if captured.owner_node_id != source.owner_node_id
                || captured.execution_id != source.execution.execution_id
                || captured.evidence_through != source.evidence_through
            {
                return Err(corrupt(invalid("captured file differs from sealed source")));
            }
            captured.validate_header().map_err(corrupt)?;
            if let Some(anchor) = &captured.execution
                && !source
                    .execution
                    .position
                    .covers(&anchor.position)
                    .map_err(Error::from)
                    .map_err(corrupt)?
            {
                return Err(corrupt(invalid(
                    "captured anchor follows its published revision",
                )));
            }
            let mut lifecycle = execution.value.clone();
            lifecycle.saved = None;
            if captured.lifecycle != lifecycle {
                return Err(corrupt(invalid(
                    "captured lifecycle differs from sealed execution",
                )));
            }
            captured.context.validate(&lifecycle).map_err(corrupt)?;
            let mut captured = captured;
            captured.outputs = source.outputs.clone();
            Ok(captured)
        })?,
    )
}

pub(crate) async fn read_file(
    view: &StorageView,
    file: &FileReference,
) -> Result<QueryValue<CapturedSource>> {
    let native = read_native::<Source>(view, file, super::native::source::MEDIA_TYPE).await?;
    let maximum = view.decoded_bytes()?;
    let computation = view.begin_computation()?;
    let projection_memory = allowance(
        native.projection_memory()?,
        maximum,
        "captured report projection",
    )?;
    computation.check()?;
    drop(computation);
    Ok(
        view.project_owned(native, projection_memory, |native, limit| {
            native.into_source(limit).map_err(corrupt)
        })?,
    )
}

/* === Private Functions === */
pub(super) async fn read_native<T: Protobuf + MemorySize>(
    view: &StorageView,
    file: &FileReference,
    media_type: &str,
) -> Result<QueryValue<T>> {
    let computation = view.begin_computation()?;
    if file.metadata.media_type != media_type {
        return Err(invalid("sealed capture has a different native media type"));
    }
    let maximum = view.decoded_bytes()?;
    let length = usize::try_from(file.length)
        .map_err(|_| invalid("captured file length exceeds native address space"))?;
    let buffer_memory = length
        .checked_add(std::mem::size_of::<Vec<u8>>())
        .and_then(NonZeroUsize::new)
        .filter(|bytes| length != 0 && *bytes <= maximum)
        .ok_or_else(|| invalid("captured file exceeds original decoded allowance"))?;
    computation.check()?;
    drop(computation);
    let mut reader = view.read_file(file).await?;
    let chunk_size = NonZeroUsize::new(CHUNK_BYTES.min(length)).unwrap();
    let first = reader
        .next_chunk(chunk_size)
        .await?
        .ok_or_else(|| invalid("captured file has no immutable bytes"))?
        .into_query_value();
    let mut bytes = view.project_owned(first, buffer_memory, |first, _| {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(length)
            .map_err(|_| exhausted("captured file allocation"))?;
        if bytes.capacity() > length || first.len() > length {
            return Err(corrupt(invalid(
                "captured file allocation exceeds preclaim",
            )));
        }
        bytes.extend_from_slice(&first);
        Ok(bytes)
    })?;
    while let Some(chunk) = reader.next_chunk(chunk_size).await? {
        let chunk = chunk.into_query_value();
        bytes = view.project_owned(bytes, buffer_memory, |mut bytes, _| {
            let next = bytes
                .len()
                .checked_add(chunk.len())
                .filter(|next| *next <= length && *next <= bytes.capacity())
                .ok_or_else(|| corrupt(invalid("captured file exceeds frozen length")))?;
            bytes.extend_from_slice(&chunk);
            debug_assert_eq!(bytes.len(), next);
            Ok(bytes)
        })?;
    }
    // The Reader Verified Final Length And Digest Before Releasing Its Last Chunk
    let computation = view.begin_computation()?;
    if bytes.len() != length {
        return Err(invalid("captured file differs from frozen length"));
    }
    let decode_memory = allowance(
        T::decoding_memory(&bytes).map_err(codec_error)?,
        maximum,
        "captured file decoding",
    )?;
    computation.check()?;
    drop(computation);
    let native = view.project_owned(bytes, decode_memory, |bytes, _| {
        T::decode(&bytes).map_err(codec_error)
    })?;
    Ok(native)
}

fn allowance(bytes: usize, maximum: NonZeroUsize, resource: &'static str) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|bytes| bytes.get() != usize::MAX && *bytes <= maximum)
        .ok_or_else(|| {
            tracing::warn!(
                resource,
                bytes,
                maximum = maximum.get(),
                "captured report exceeds projection budget"
            );
            exhausted(resource).into()
        })
}

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

fn exhausted(resource: &'static str) -> harmos::Error {
    harmos::Error::ResourceExhausted { resource }
}

fn corrupt(source: Error) -> harmos::Error {
    if let Error::Storage(dokime::storage::Error::Runtime(source)) = source {
        return source;
    }
    harmos::Error::Backend {
        operation: "decode native captured report",
        kind: harmos::BackendFailure::Corrupt,
        source: Box::new(source),
    }
}

fn codec_error(source: harmos::MessageError) -> harmos::Error {
    harmos::Error::Backend {
        operation: "decode native captured report",
        kind: harmos::BackendFailure::Corrupt,
        source: Box::new(source),
    }
}

/* === Tests === */
