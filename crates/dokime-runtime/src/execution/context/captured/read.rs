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
use harmos::{
    Message,
    storage::{FileReference, ObjectRevision},
};

// Local Imports
use super::{native::source::Source, source::CapturedSource};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::result::TestExecution;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn read_execution(
    session: &crate::storage::Manager, execution: &ObjectRevision<TestExecution>,
) -> Result<harmos::storage::ReadBatch<CapturedSource>> {
    let seal = execution.value.saved.as_ref().ok_or_else(|| invalid("execution has no captured source dependencies"))?;
    if !execution.position.covers(&seal.dependencies_saved)? || execution.position.sequence == seal.dependencies_saved.sequence {
        return Err(invalid("seal revision does not follow saved dependencies"));
    }
    let source = &seal.source;
    let captured = read_file(session, &source.context).await?;
    captured.try_map(|mut captured| {
        if captured.owner_node_id != source.owner_node_id
            || captured.execution_id != source.execution.execution_id
            || captured.evidence_through != source.evidence_through
        {
            return Err(invalid("captured file differs from sealed source"));
        }
        captured.validate_header()?;
        if let Some(anchor) = &captured.execution
            && !source.execution.position.covers(&anchor.position)?
        {
            return Err(invalid("captured anchor follows its published revision"));
        }
        let mut lifecycle = execution.value.clone();
        lifecycle.saved = None;
        if captured.lifecycle != lifecycle {
            return Err(invalid("captured lifecycle differs from sealed execution"));
        }
        captured.context.validate(&lifecycle)?;
        captured.outputs = source.outputs.clone();
        Ok(captured)
    })
}

pub(crate) async fn read_file(
    session: &crate::storage::Manager, file: &FileReference,
) -> Result<harmos::storage::ReadBatch<CapturedSource>> {
    let native = read_native::<Source>(session, file, super::native::source::MEDIA_TYPE).await?;
    native.try_map(Source::into_source)
}

pub(super) async fn read_native<T: Message + harmos::MemorySize>(
    session: &crate::storage::Manager, file: &FileReference, media_type: &str,
) -> Result<harmos::storage::ReadBatch<T>> {
    if file.metadata.media_type != media_type {
        return Err(invalid("sealed capture has a different native media type"));
    }

    let mut reader = session
        .read_file(harmos::storage::FileRead {
            source: harmos::storage::FileSource::Stored(file.clone()),
            range: harmos::storage::ByteRange { offset: 0, limit: None },
        })
        .await?;
    let length = usize::try_from(file.length).map_err(|_| invalid("captured file length exceeds native address space"))?;
    let mut bytes = harmos::storage::ReadBatch::new(()).try_map(|_| {
        let mut bytes = Vec::<u8>::new();
        bytes.try_reserve_exact(length).map_err(|_| invalid("captured file assembly allocation failed"))?;
        Ok::<_, Error>(bytes)
    })?;
    while let Some(batch) = reader.next().await? {
        if batch.offset() != u64::try_from(bytes.len()).map_err(|_| invalid("captured file offset overflows"))? {
            return Err(invalid("captured file pages are not contiguous"));
        }
        if bytes.len().checked_add(batch.bytes().len()).is_none_or(|end| end > length) {
            return Err(invalid("captured file pages exceed the immutable length"));
        }
        let page = batch.try_map(|batch| Ok::<_, Error>(batch.bytes))?;
        bytes = crate::runtime::storage::telemetry::append_retained(bytes, page)?;
    }
    if reader.completion()?.bytes != file.length || bytes.len() != length {
        return Err(invalid("captured file completion differs from its immutable length"));
    }
    bytes.try_map(|bytes| {
        T::decode(&bytes).map_err(|source| {
            harmos::Error::Backend {
                operation: "decode native captured report",
                kind: harmos::BackendFailure::Corrupt,
                source: Box::new(source),
            }
            .into()
        })
    })
}

/* === Private Functions === */

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

/* === Tests === */
