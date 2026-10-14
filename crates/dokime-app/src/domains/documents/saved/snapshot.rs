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
use std::{num::NonZeroUsize, sync::Mutex};

// Third Party Imports
use harmos::{MemorySize, storage::QueryValue};

// Local Imports
use super::{SavedOutputFile, exhausted, invalid, slots, source, window};
use crate::{Result, execution_runtime::outputs::ExecutionFileContent};
use dokime::execution::saved::ExecutionOutput;

/* === Constants === */
const CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct PendingSnapshot {
    maximum: NonZeroUsize,
    require_complete: bool,
    target: usize,
    buffer: QueryValue<Buffer>,
    chunk: Option<PendingChunk>,
}

struct PendingChunk {
    offset: usize,
    bytes: QueryValue<Vec<u8>>,
}

struct Buffer {
    content: Mutex<ExecutionFileContent>,
    memory: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl SavedOutputFile {
    pub(crate) async fn snapshot(
        &mut self,
        maximum: NonZeroUsize,
        require_complete: bool,
    ) -> Result<QueryValue<ExecutionFileContent>> {
        if self.window.is_some() {
            return Err(invalid("saved output has an unsettled window projection"));
        }
        if let Some(pending) = &self.snapshot {
            if pending.maximum != maximum || pending.require_complete != require_complete {
                return Err(invalid("saved output retry changed its original snapshot"));
            }
        } else {
            if self.reader.offset() != 0 {
                return Err(invalid("saved output snapshot must start at zero"));
            }
            let length = self.descriptor().file.length;
            if require_complete && length > maximum.get() as u64 {
                return Err(crate::Error::ExecutionFileTooLarge);
            }
            let target = usize::try_from(length.min(maximum.get() as u64))
                .map_err(|_| invalid("saved output snapshot length overflows"))?;
            let index = self.index;
            let memory = buffer_memory(self.descriptor(), target)?;
            let buffer = self.outputs.original.project(
                &self.outputs.revision,
                memory,
                |revision, allowance| {
                    Buffer::new(&source(revision)?.outputs[index], target, allowance)
                },
            )?;
            self.snapshot = Some(PendingSnapshot {
                maximum,
                require_complete,
                target,
                buffer,
                chunk: None,
            });
        }

        loop {
            let pending = self.snapshot.as_ref().expect("selected snapshot");
            let copied = pending.buffer.length()?;
            if pending.chunk.is_none() && copied == pending.target {
                break;
            }
            if pending.chunk.is_none() {
                let maximum =
                    NonZeroUsize::new((pending.target - copied).min(CHUNK_BYTES)).unwrap();
                let chunk = self
                    .reader
                    .next_chunk(maximum)
                    .await?
                    .ok_or_else(|| invalid("saved output ended before selected snapshot"))?;
                self.snapshot.as_mut().expect("selected snapshot").chunk = Some(PendingChunk {
                    offset: copied,
                    bytes: chunk.into_query_value(),
                });
            }
            let pending = self.snapshot.as_ref().expect("selected snapshot");
            let chunk = pending.chunk.as_ref().expect("retained snapshot chunk");
            let copied = self.outputs.original.project(
                &chunk.bytes,
                NonZeroUsize::new(std::mem::size_of::<u64>()).unwrap(),
                |bytes, _| {
                    pending
                        .buffer
                        .append(chunk.offset, bytes)
                        .map(|copied| copied as u64)
                },
            )?;
            if *copied > pending.target as u64 {
                return Err(invalid(
                    "saved output snapshot exceeded its original target",
                ));
            }
            self.snapshot.as_mut().expect("selected snapshot").chunk = None;
        }

        // Verify An Empty File Through The Original Reader Too.
        if self.descriptor().file.length == 0
            && self
                .reader
                .next_chunk(NonZeroUsize::new(1).unwrap())
                .await?
                .is_some()
        {
            return Err(invalid("empty saved output returned a chunk"));
        }
        let pending = self.snapshot.as_ref().expect("selected snapshot");
        let memory = self.window_memory(0, pending.maximum)?;
        let output = &source(&self.outputs.revision)?.outputs[self.index];
        let content =
            self.outputs
                .original
                .project(&pending.buffer, memory, |buffer, allowance| {
                    let content = buffer
                        .content
                        .lock()
                        .map_err(|_| exhausted("saved output snapshot buffer is poisoned"))?;
                    window(output, 0, &content.bytes, allowance)
                })?;
        self.snapshot = None;
        Ok(content)
    }
}

impl MemorySize for Buffer {
    fn memory_size(&self) -> usize {
        self.memory
    }
}

impl Buffer {
    fn new(
        output: &ExecutionOutput,
        target: usize,
        allowance: NonZeroUsize,
    ) -> harmos::Result<Self> {
        let memory = buffer_memory(output, target)?;
        if memory > allowance {
            return Err(exhausted(
                "saved output snapshot exceeds projection admission",
            ));
        }
        let mut content = window(output, 0, &[], allowance)?;
        content.bytes = slots(target)?;
        Ok(Self {
            content: Mutex::new(content),
            memory: memory.get(),
        })
    }

    fn length(&self) -> harmos::Result<usize> {
        let content = self
            .content
            .lock()
            .map_err(|_| exhausted("saved output snapshot buffer is poisoned"))?;
        Ok(content.bytes.len())
    }

    fn append(&self, offset: usize, bytes: &[u8]) -> harmos::Result<usize> {
        let mut content = self
            .content
            .lock()
            .map_err(|_| exhausted("saved output snapshot buffer is poisoned"))?;
        let end = offset
            .checked_add(bytes.len())
            .ok_or_else(|| exhausted("saved output snapshot offset overflows"))?;
        if end > content.bytes.capacity() {
            return Err(exhausted(
                "saved output snapshot exceeds its fixed capacity",
            ));
        }
        if content.bytes.len() == end {
            if content.bytes.get(offset..end) != Some(bytes) {
                return Err(super::refused(
                    "saved output snapshot retry changed its chunk",
                ));
            }
            return Ok(end);
        }
        if content.bytes.len() != offset {
            return Err(super::refused(
                "saved output snapshot chunk is out of order",
            ));
        }
        content.bytes.extend_from_slice(bytes);
        Ok(end)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn buffer_memory(output: &ExecutionOutput, target: usize) -> harmos::Result<NonZeroUsize> {
    std::mem::size_of::<Buffer>()
        .checked_add(output.relative_path.len())
        .and_then(|bytes| bytes.checked_add(output.file.metadata.media_type.len()))
        .and_then(|bytes| bytes.checked_add(target))
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| exhausted("saved output snapshot memory overflows"))
}

/* === Tests === */
