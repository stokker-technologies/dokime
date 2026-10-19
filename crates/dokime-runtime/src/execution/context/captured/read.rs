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
use crate::storage::Manager;
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

pub(crate) async fn read_execution(storage: &Manager, execution: &ObjectRevision<TestExecution>) -> Result<CapturedSource> {
    let seal = execution.value.saved.as_ref().ok_or_else(|| invalid("execution has no captured source dependencies"))?;
    if !execution.position.covers(&seal.dependencies_saved)? || execution.position.sequence == seal.dependencies_saved.sequence {
        return Err(invalid("seal revision does not follow saved dependencies"));
    }
    let source = &seal.source;
    let mut captured = read_file(storage, &source.context).await?;
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
}

pub(crate) async fn read_file(storage: &Manager, file: &FileReference) -> Result<CapturedSource> {
    read_native::<Source>(storage, file, super::native::source::MEDIA_TYPE).await?.into_source()
}

pub(super) async fn read_native<T: Message>(storage: &Manager, file: &FileReference, media_type: &str) -> Result<T> {
    if file.metadata.media_type != media_type {
        return Err(invalid("sealed capture has a different native media type"));
    }
    let bytes = crate::storage::read_file_bytes(storage, file).await?;
    T::decode(&bytes).map_err(|source| {
        harmos::Error::Backend {
            operation: "decode native captured report",
            kind: harmos::BackendFailure::Corrupt,
            source: Box::new(source),
        }
        .into()
    })
}

/* === Private Functions === */

fn invalid(reason: &str) -> Error {
    Error::InvalidReportInputs(reason.into())
}

/* === Tests === */
