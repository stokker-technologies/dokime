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
use std::{
    io::{Read, Seek},
    num::NonZeroUsize,
    sync::Mutex,
};

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{
        QueryValue, SharedQueryValue, StorageProductionComputation, StorageProductionPreparation,
    },
};
use tokio::task::JoinHandle;

// Local Imports
use super::{content_bytes, error};
use crate::{
    Error, Result,
    execution_runtime::outputs::{
        ExecutionFileContent, ExecutionFileContentKind, ExecutionFileStreamState,
    },
};
use dokime::storage::Manager;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Reader {
    source: SharedQueryValue<Source>,
    offset: u64,
    pending: Option<Pending>,
}

struct Source {
    file: Mutex<std::fs::File>,
    metadata: ExecutionFileContent,
}

struct Pending {
    maximum: NonZeroUsize,
    deadline: tokio::time::Instant,
    task: JoinHandle<Result<QueryValue<ExecutionFileContent>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Reader {
    pub(super) fn prepare(storage: &Manager, path: &str) -> Result<StorageProductionPreparation> {
        let source_memory = std::mem::size_of::<Source>()
            .checked_add(path.len())
            .and_then(|bytes| {
                bytes.checked_add(super::media_type_for_path(path).map_or(0, str::len))
            })
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| error("native stream source memory overflows"))?;
        Ok(storage.prepare_read_production(source_memory)?)
    }

    pub(super) fn new(
        preparation: StorageProductionPreparation,
        computation: &StorageProductionComputation,
        file: std::fs::File,
        path: &str,
        observed_size: u64,
        stream_state: ExecutionFileStreamState,
    ) -> Result<Self> {
        computation.check()?;
        let content_type = super::media_type_for_path(path).map(text).transpose()?;
        let metadata = ExecutionFileContent {
            relative_path: text(path)?,
            content_kind: ExecutionFileContentKind::from_metadata(path, content_type.as_deref()),
            content_type,
            stream_state,
            offset: 0,
            size_bytes: observed_size,
            bytes: Vec::new(),
        };
        let source = preparation
            .finish(Source {
                file: Mutex::new(file),
                metadata,
            })?
            .try_into_value()?
            .try_into_shared()?;
        computation.check()?;
        Ok(Self {
            source,
            offset: 0,
            pending: None,
        })
    }

    pub(crate) fn metadata(&self) -> &ExecutionFileContent {
        &self.source.metadata
    }

    pub(crate) fn project_metadata<U: MemorySize>(
        &self,
        maximum: NonZeroUsize,
        map: impl FnOnce(&ExecutionFileContent, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let computation = self.source.computation()?;
        let projected =
            computation.project_shared(&self.source, maximum, |source, allowance| {
                map(&source.metadata, allowance)
            })?;
        Ok(projected)
    }

    pub(crate) async fn next_chunk(
        &mut self,
        maximum: NonZeroUsize,
    ) -> Result<Option<QueryValue<ExecutionFileContent>>> {
        if let Some(pending) = &self.pending {
            if pending.maximum != maximum {
                return Err(harmos::Error::InvalidInput {
                    field: "native stream chunk",
                    reason: "pending read has another maximum",
                }
                .into());
            }
        } else {
            if self.offset == self.source.metadata.size_bytes {
                return Ok(None);
            }
            let computation = self.source.computation()?;
            let deadline = computation.deadline();
            let scratch = computation.scratch_bytes();
            let source = self.source.clone();
            let offset = self.offset;
            let length = (source.metadata.size_bytes - offset).min(maximum.get() as u64) as usize;
            let allowance = super::content_memory(&source.metadata.relative_path, length)?;
            let read = move || {
                computation
                    .project_shared(&source, allowance, |source, _| {
                        // Read Directly Into The Final Buffer; No Hidden Async File Buffer.
                        let mut bytes = content_bytes(length).map_err(harmos_error)?;
                        let mut file = source.file.lock().map_err(|_| {
                            harmos::Error::backend(
                                "lock native output source",
                                harmos::BackendFailure::Corrupt,
                                std::io::Error::other("native stream file lock poisoned"),
                            )
                        })?;
                        file.seek(std::io::SeekFrom::Start(offset))
                            .map_err(|failure| harmos_error(error(failure)))?;
                        file.read_exact(&mut bytes)
                            .map_err(|failure| harmos_error(error(failure)))?;
                        drop(file);
                        Ok(ExecutionFileContent {
                            relative_path: text(&source.metadata.relative_path)
                                .map_err(harmos_error)?,
                            content_type: source
                                .metadata
                                .content_type
                                .as_deref()
                                .map(text)
                                .transpose()
                                .map_err(harmos_error)?,
                            content_kind: source.metadata.content_kind,
                            stream_state: source.metadata.stream_state,
                            offset,
                            size_bytes: source.metadata.size_bytes,
                            bytes,
                        })
                    })
                    .map_err(Error::from)
            };
            let working = std::mem::size_of_val(&read)
                .checked_add(std::mem::size_of::<Self>())
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Pending>()))
                .ok_or_else(exhausted)?;
            if working > scratch {
                return Err(exhausted());
            }
            let task = tokio::task::spawn_blocking(read);
            self.pending = Some(Pending {
                maximum,
                deadline,
                task,
            });
        }

        let pending = self
            .pending
            .as_mut()
            .expect("accepted native read retained");
        let result = tokio::time::timeout_at(pending.deadline, &mut pending.task)
            .await
            .map_err(|_| harmos::Error::ResourceExhausted {
                resource: "query computation time",
            })?;
        self.pending = None;
        let content = result.map_err(|source| Error::Join { source })??;
        self.offset = content
            .offset
            .checked_add(content.bytes.len() as u64)
            .ok_or_else(|| error("native stream offset overflows"))?;
        Ok(Some(content))
    }
}

impl MemorySize for Source {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.metadata.relative_path.capacity())
            .saturating_add(
                self.metadata
                    .content_type
                    .as_ref()
                    .map_or(0, String::capacity),
            )
            .saturating_add(self.metadata.bytes.capacity())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn text(source: &str) -> Result<String> {
    let mut target = String::new();
    target
        .try_reserve_exact(source.len())
        .map_err(|_| exhausted())?;
    if target.capacity() > source.len() {
        return Err(exhausted());
    }
    target.push_str(source);
    Ok(target)
}

fn exhausted() -> Error {
    harmos::Error::ResourceExhausted {
        resource: "native stream memory",
    }
    .into()
}

fn harmos_error(error: Error) -> harmos::Error {
    match error {
        Error::Harmos(source) => source,
        other => harmos::Error::backend(
            "read original native output",
            harmos::BackendFailure::Corrupt,
            other,
        ),
    }
}

/* === Tests === */
